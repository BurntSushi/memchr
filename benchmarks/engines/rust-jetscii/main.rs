use std::io::Write;

use shared::{Benchmark, Sample};

fn main() -> anyhow::Result<()> {
    let Some(arg) = std::env::args_os().nth(1) else {
        anyhow::bail!("Usage: runner (<engine-name> | --version)")
    };
    let Ok(arg) = arg.into_string() else {
        anyhow::bail!("argument given is not valid UTF-8")
    };
    if arg == "--version" {
        writeln!(std::io::stdout(), env!("CARGO_PKG_VERSION"))?;
        return Ok(());
    }
    let engine = arg;
    let b = Benchmark::from_stdin()?;
    let samples = match (&*engine, &*b.model) {
        ("memmem-prebuilt", "count") => memmem_prebuilt_count(&b)?,
        ("memmem-oneshot", "count") => memmem_oneshot_count(&b)?,
        ("memchr", "count-bytes") => memchr_count(&b)?,
        ("memchr-onlycount", "count-bytes") => memchr_only_count(&b)?,
        ("memchr3", "count-bytes") => memchr3_count(&b)?,
        ("memchr3-onlycount", "count-bytes") => memchr3_only_count(&b)?,
        (engine, model) => {
            anyhow::bail!("unrecognized engine '{engine}' and model '{model}'")
        }
    };
    let mut stdout = std::io::stdout().lock();
    for s in samples.iter() {
        writeln!(stdout, "{},{}", s.duration.as_nanos(), s.count)?;
    }
    Ok(())
}

fn memmem_prebuilt_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let needle = b.one_needle()?;
    let finder = jetscii::ByteSubstring::new(needle);
    shared::run(b, || {
        Ok(shared::count_memmem(haystack, needle, |h, _| finder.find(h)))
    })
}

fn memmem_oneshot_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let needle = b.one_needle()?;
    shared::run(b, || {
        Ok(shared::count_memmem(haystack, needle, |h, n| {
            jetscii::ByteSubstring::new(n).find(h)
        }))
    })
}

fn memchr_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let needle = b.one_needle_byte()?;
    let finder = jetscii::bytes!(needle);
    shared::run(b, || Ok(finder.iter(haystack).count_slow()))
}

fn memchr_only_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let needle = b.one_needle_byte()?;
    let finder = jetscii::bytes!(needle);
    shared::run(b, || Ok(finder.iter(haystack).count()))
}

fn memchr3_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let (n1, n2, n3) = b.three_needle_bytes()?;
    let finder = jetscii::bytes!(n1, n2, n3);
    shared::run(b, || Ok(finder.iter(haystack).count_slow()))
}

fn memchr3_only_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let (n1, n2, n3) = b.three_needle_bytes()?;
    let finder = jetscii::bytes!(n1, n2, n3);
    shared::run(b, || Ok(finder.iter(haystack).count()))
}

trait IteratorExt: Iterator {
    /// Like `Iterator::count`, but guarantees that it gets the count by
    /// iterating over each element without taking any specialized shortcuts.
    ///
    /// We do this because the jetscii crate may specialize `count` in certain
    /// circumstances, and we'd generally like to measure how long it takes
    /// to find all occurrences of a needle and not just the number of them.
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
