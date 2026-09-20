use std::io::Write;

use memchr_n::{Backend, MemchrN};

use shared::{Benchmark, Sample};

fn main() -> anyhow::Result<()> {
    let mut args = vec![];
    for osarg in std::env::args_os().skip(1) {
        let Ok(arg) = osarg.into_string() else {
            anyhow::bail!("all arguments must be valid UTF-8")
        };
        args.push(arg);
    }
    anyhow::ensure!(
        !args.is_empty(),
        "Usage: runner [--quiet] (<engine-name> | --version)"
    );
    if args.iter().any(|a| a == "--version") {
        writeln!(std::io::stdout(), env!("CARGO_PKG_VERSION"))?;
        return Ok(());
    }
    let quiet = args.iter().any(|a| a == "--quiet");
    let engine = &**args.last().unwrap();
    let b = Benchmark::from_stdin()?;
    let samples = match (&*engine, &*b.model) {
        ("memchr-oneshot", "count-bytes") => memchr_oneshot_count(&b)?,
        ("memchr-prebuilt", "count-bytes") => memchr_prebuilt_count(&b)?,
        ("memchr-fallback", "count-bytes") => memchr_fallback_count(&b)?,
        ("memchr-onlycount", "count-bytes") => memchr_onlycount(&b)?,
        ("memchr2", "count-bytes") => memchr2_count(&b)?,
        ("memchr2-onlycount", "count-bytes") => memchr2_only_count(&b)?,
        ("memchr2-fallback", "count-bytes") => memchr2_fallback_count(&b)?,
        ("memchr3", "count-bytes") => memchr3_count(&b)?,
        ("memchr3-onlycount", "count-bytes") => memchr3_only_count(&b)?,
        ("memchr3-fallback", "count-bytes") => memchr3_fallback_count(&b)?,
        ("byteset", "count-bytes") => byteset_count(&b, Backend::Auto)?,
        ("byteset-onlycount", "count-bytes") => {
            byteset_onlycount(&b, Backend::Auto)?
        }
        ("range", "count-bytes") => range_count(&b, Backend::Auto)?,
        ("range-onlycount", "count-bytes") => {
            range_onlycount(&b, Backend::Auto)?
        }
        (engine, model) => {
            anyhow::bail!("unrecognized engine '{engine}' and model '{model}'")
        }
    };
    if !quiet {
        let mut stdout = std::io::stdout().lock();
        for s in samples.iter() {
            writeln!(stdout, "{},{}", s.duration.as_nanos(), s.count)?;
        }
    }
    Ok(())
}

/// Rebuilds the searcher on every call, which is how `memchr::memchr` and the
/// other oneshot engines behave.
fn memchr_oneshot_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let needle = b.one_needle_byte()?;
    shared::run(b, || {
        Ok(shared::count_memchr(haystack, needle, |h, n1| {
            MemchrN::new(&[n1]).find(h)
        }))
    })
}

fn memchr_prebuilt_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let needle = b.one_needle_byte()?;
    shared::run(b, || {
        let finder = MemchrN::new(&[needle]);
        Ok(finder.iter(haystack).count_slow())
    })
}

fn memchr_fallback_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let needle = b.one_needle_byte()?;
    let finder = MemchrN::new_with_backend(&[needle], Backend::Swar);
    shared::run(b, || Ok(finder.iter(haystack).count_slow()))
}

/// Uses the specialized counting kernel, which tallies matches without
/// reporting where they are. This is the counterpart of the
/// `rust/memchr/memchr/onlycount` and `rust/bytecount/*` engines.
fn memchr_onlycount(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let needle = b.one_needle_byte()?;
    shared::run(b, || {
        let finder = MemchrN::new(&[needle]);
        Ok(finder.iter(haystack).count())
    })
}

fn memchr2_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let (n1, n2) = b.two_needle_bytes()?;
    shared::run(b, || {
        let finder = MemchrN::new(&[n1, n2]);
        Ok(finder.iter(haystack).count_slow())
    })
}

fn memchr2_only_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let (n1, n2) = b.two_needle_bytes()?;
    shared::run(b, || {
        let finder = MemchrN::new(&[n1, n2]);
        Ok(finder.iter(haystack).count())
    })
}

fn memchr2_fallback_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let (n1, n2) = b.two_needle_bytes()?;
    let finder = MemchrN::new_with_backend(&[n1, n2], Backend::Swar);
    shared::run(b, || Ok(finder.iter(haystack).count_slow()))
}

fn memchr3_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let (n1, n2, n3) = b.three_needle_bytes()?;
    shared::run(b, || {
        let finder = MemchrN::new(&[n1, n2, n3]);
        Ok(finder.iter(haystack).count_slow())
    })
}

fn memchr3_only_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let (n1, n2, n3) = b.three_needle_bytes()?;
    shared::run(b, || {
        let finder = MemchrN::new(&[n1, n2, n3]);
        Ok(finder.iter(haystack).count())
    })
}

fn memchr3_fallback_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let (n1, n2, n3) = b.three_needle_bytes()?;
    let finder = MemchrN::new_with_backend(&[n1, n2, n3], Backend::Swar);
    shared::run(b, || Ok(finder.iter(haystack).count_slow()))
}

/// Searches for any byte in a set of arbitrary size.
///
/// Which kernel this runs is up to `memchr_n`: it picks between a shuffle over
/// one nibble, a pair of nibble lookups and a bitset probe based on the set,
/// so the benchmark definition's needles are what select the kernel.
fn byteset_count(
    b: &Benchmark,
    backend: Backend,
) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let bytes = needle_bytes(b)?;
    let finder = MemchrN::new_with_backend(&bytes, backend);
    shared::run(b, || Ok(finder.iter(haystack).count_slow()))
}

fn byteset_onlycount(
    b: &Benchmark,
    backend: Backend,
) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let bytes = needle_bytes(b)?;
    let finder = MemchrN::new_with_backend(&bytes, backend);
    shared::run(b, || Ok(finder.iter(haystack).count()))
}

fn range_count(
    b: &Benchmark,
    backend: Backend,
) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let (first, last) = needle_range(b)?;
    let finder = MemchrN::from_range_with_backend(first..=last, backend);
    shared::run(b, || Ok(finder.iter(haystack).count_slow()))
}

fn range_onlycount(
    b: &Benchmark,
    backend: Backend,
) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let (first, last) = needle_range(b)?;
    let finder = MemchrN::from_range_with_backend(first..=last, backend);
    shared::run(b, || Ok(finder.iter(haystack).count()))
}

/// The set of every needle in the benchmark, each of which must be one byte.
fn needle_bytes(b: &Benchmark) -> anyhow::Result<Vec<u8>> {
    anyhow::ensure!(!b.needles.is_empty(), "benchmark has no needles");
    let mut bytes = Vec::with_capacity(b.needles.len());
    for needle in b.needles.iter() {
        anyhow::ensure!(
            needle.len() == 1,
            "every needle must have length 1 (in bytes) but one has length {}",
            needle.len(),
        );
        bytes.push(needle[0]);
    }
    Ok(bytes)
}

/// The inclusive range spelled by the benchmark's single two byte needle, as
/// `<first><last>`.
fn needle_range(b: &Benchmark) -> anyhow::Result<(u8, u8)> {
    let needle = b.one_needle()?;
    let &[first, last] = needle else {
        anyhow::bail!(
            "a range needle must have length 2 (in bytes) but it has length {}",
            needle.len(),
        )
    };
    anyhow::ensure!(
        first <= last,
        "range needle {first:#04x}..={last:#04x} is empty",
    );
    Ok((first, last))
}

trait IteratorExt: Iterator {
    /// Like `Iterator::count`, but guarantees that it gets the count by
    /// iterating over each element without taking any specialized shortcuts.
    ///
    /// We do this because `memchr_n` specializes `count` to a kernel that only
    /// tallies matches, and we'd generally like to measure how long it takes to
    /// find all occurrences of a needle and not just the number of them.
    fn count_slow(mut self) -> usize
    where
        Self: Sized,
    {
        let mut count = 0;
        while let Some(_) = self.next() {
            count += 1;
        }
        count
    }
}

impl<I: Iterator> IteratorExt for I {}
