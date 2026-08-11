use std::io::Write;

use memchr::arch::all::packedpair::HeuristicFrequencyRank;

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
        ("memchr-onlycount", "count-bytes") => memchr_only_count(&b)?,
        ("memchr-fallback", "count-bytes") => memchr_fallback_count(&b)?,
        ("memchr-fallback-onlycount", "count-bytes") => {
            memchr_fallback_only_count(&b)?
        }
        ("memchr-naive", "count-bytes") => memchr_naive_count(&b)?,
        ("memchr2", "count-bytes") => memchr2_count(&b)?,
        ("memchr2-fallback", "count-bytes") => memchr2_fallback_count(&b)?,
        ("memchr2-naive", "count-bytes") => memchr2_naive_count(&b)?,
        ("memchr3", "count-bytes") => memchr3_count(&b)?,
        ("memchr3-fallback", "count-bytes") => memchr3_fallback_count(&b)?,
        ("memchr3-naive", "count-bytes") => memchr3_naive_count(&b)?,
        ("memrchr", "count-bytes") => memrchr_count(&b)?,
        ("memrchr2", "count-bytes") => memrchr2_count(&b)?,
        ("memrchr3", "count-bytes") => memrchr3_count(&b)?,
        ("memmem-prebuilt", "count") => memmem_prebuilt_count(&b)?,
        ("memmem-oneshot", "count") => memmem_oneshot_count(&b)?,
        ("memmem-binary", "count") => memmem_binary_count(&b)?,
        ("memmem-twoway", "count") => memmem_twoway_count(&b)?,
        ("memmem-rabinkarp", "count") => memmem_rabinkarp_count(&b)?,
        ("memmem-shiftor", "count") => memmem_shiftor_count(&b)?,
        ("memmem-prebuilt", "needles-in-needles") => {
            memmem_prebuilt_needles(&b)?
        }
        ("memmem-prebuilt", "needles-in-haystack") => {
            memmem_prebuilt_haystack(&b)?
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

fn memchr_oneshot_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let needle = b.one_needle_byte()?;
    shared::run(b, || {
        Ok(shared::count_memchr(haystack, needle, |h, n1| {
            memchr::memchr(n1, h)
        }))
    })
}

fn memchr_prebuilt_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let needle = b.one_needle_byte()?;
    shared::run(b, || Ok(memchr::memchr_iter(needle, haystack).count_slow()))
}

fn memchr_only_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let needle = b.one_needle_byte()?;
    shared::run(b, || Ok(memchr::memchr_iter(needle, haystack).count()))
}

fn memchr_fallback_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let needle = b.one_needle_byte()?;
    let finder = memchr::arch::all::memchr::One::new(needle);
    shared::run(b, || Ok(finder.iter(haystack).count_slow()))
}

fn memchr_fallback_only_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let needle = b.one_needle_byte()?;
    let finder = memchr::arch::all::memchr::One::new(needle);
    shared::run(b, || Ok(finder.iter(haystack).count()))
}

fn memchr_naive_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let needle = b.one_needle_byte()?;
    shared::run(b, || {
        Ok(shared::count_memchr(haystack, needle, |h, n1| {
            h.iter().position(|&b| b == n1)
        }))
    })
}

fn memchr2_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let (n1, n2) = b.two_needle_bytes()?;
    shared::run(b, || Ok(memchr::memchr2_iter(n1, n2, haystack).count_slow()))
}

fn memchr2_fallback_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let (n1, n2) = b.two_needle_bytes()?;
    let finder = memchr::arch::all::memchr::Two::new(n1, n2);
    shared::run(b, || Ok(finder.iter(haystack).count_slow()))
}

fn memchr2_naive_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let (n1, n2) = b.two_needle_bytes()?;
    shared::run(b, || {
        Ok(shared::count_memchr2(haystack, n1, n2, |h, n1, n2| {
            h.iter().position(|&b| b == n1 || b == n2)
        }))
    })
}

fn memchr3_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let (n1, n2, n3) = b.three_needle_bytes()?;
    shared::run(b, || {
        Ok(memchr::memchr3_iter(n1, n2, n3, haystack).count_slow())
    })
}

fn memchr3_fallback_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let (n1, n2, n3) = b.three_needle_bytes()?;
    let finder = memchr::arch::all::memchr::Three::new(n1, n2, n3);
    shared::run(b, || Ok(finder.iter(haystack).count_slow()))
}

fn memchr3_naive_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let (n1, n2, n3) = b.three_needle_bytes()?;
    shared::run(b, || {
        Ok(shared::count_memchr3(haystack, n1, n2, n3, |h, n1, n2, n3| {
            h.iter().position(|&b| b == n1 || b == n2 || b == n3)
        }))
    })
}

fn memrchr_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let needle = b.one_needle_byte()?;
    shared::run(b, || {
        Ok(memchr::memchr_iter(needle, haystack).rev().count_slow())
    })
}

fn memrchr2_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let (n1, n2) = b.two_needle_bytes()?;
    shared::run(b, || {
        Ok(memchr::memchr2_iter(n1, n2, haystack).rev().count_slow())
    })
}

fn memrchr3_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let (n1, n2, n3) = b.three_needle_bytes()?;
    shared::run(b, || {
        Ok(memchr::memchr3_iter(n1, n2, n3, haystack).rev().count_slow())
    })
}

fn memmem_prebuilt_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let needle = b.one_needle()?;
    let finder = memchr::memmem::Finder::new(needle);
    shared::run(b, || Ok(finder.find_iter(haystack).count_slow()))
}

fn memmem_prebuilt_needles(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let finders = b
        .needles
        .iter()
        .map(|n| memchr::memmem::Finder::new(n))
        .collect::<Vec<_>>();
    shared::run(b, || {
        let mut count = 0;
        for (i, finder) in finders.iter().enumerate() {
            for haystack in b.needles[i..].iter() {
                if finder.find(haystack).is_some() {
                    count += 1;
                }
            }
        }
        Ok(count)
    })
}

fn memmem_prebuilt_haystack(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let finders = b
        .needles
        .iter()
        .map(|n| memchr::memmem::Finder::new(n))
        .collect::<Vec<_>>();
    shared::run(b, || {
        let mut count = 0;
        for finder in finders.iter() {
            if finder.find(haystack).is_some() {
                count += 1;
            }
        }
        Ok(count)
    })
}

fn memmem_oneshot_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let needle = b.one_needle()?;
    let memmem = memchr::memmem::find;
    shared::run(b, || Ok(shared::count_memmem(haystack, needle, memmem)))
}

fn memmem_binary_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let needle = b.one_needle()?;
    let finder = memchr::memmem::FinderBuilder::new()
        .build_forward_with_ranker(Binary, needle);
    shared::run(b, || Ok(finder.find_iter(haystack).count_slow()))
}

fn memmem_twoway_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let needle = b.one_needle()?;
    let finder = memchr::arch::all::twoway::Finder::new(needle);
    shared::run(b, || {
        Ok(shared::count_memmem(haystack, needle, |h, n| finder.find(h, n)))
    })
}

fn memmem_rabinkarp_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let needle = b.one_needle()?;
    let finder = memchr::arch::all::rabinkarp::Finder::new(needle);
    shared::run(b, || {
        Ok(shared::count_memmem(haystack, needle, |h, n| finder.find(h, n)))
    })
}

fn memmem_shiftor_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let needle = b.one_needle()?;
    let Some(finder) = memchr::arch::all::shiftor::Finder::new(needle) else {
        anyhow::bail!("could not build Shift-Or searcher for this needle")
    };
    shared::run(b, || {
        Ok(shared::count_memmem(haystack, needle, |h, _| finder.find(h)))
    })
}

/// A byte-frequency table that is good for scanning binary executables.
struct Binary;

impl HeuristicFrequencyRank for Binary {
    fn rank(&self, byte: u8) -> u8 {
        const TABLE: [u8; 256] = [
            255, 128, 61, 43, 50, 41, 27, 28, 57, 15, 21, 13, 24, 17, 17, 89,
            58, 16, 11, 7, 14, 23, 7, 6, 24, 9, 6, 5, 9, 4, 7, 16, 68, 11, 9,
            6, 88, 7, 4, 4, 23, 9, 4, 8, 8, 5, 10, 4, 30, 11, 9, 24, 11, 5, 5,
            5, 19, 11, 6, 17, 9, 9, 6, 8, 48, 58, 11, 14, 53, 40, 9, 9, 254,
            35, 3, 6, 52, 23, 6, 6, 27, 4, 7, 11, 14, 13, 10, 11, 11, 5, 2,
            10, 16, 12, 6, 19, 19, 20, 5, 14, 16, 31, 19, 7, 14, 20, 4, 4, 19,
            8, 18, 20, 24, 1, 25, 19, 58, 29, 10, 5, 15, 20, 2, 2, 9, 4, 3, 5,
            51, 11, 4, 53, 23, 39, 6, 4, 13, 81, 4, 186, 5, 67, 3, 2, 15, 0,
            0, 1, 3, 2, 0, 0, 5, 0, 0, 0, 2, 0, 0, 0, 12, 2, 1, 1, 3, 1, 1, 1,
            6, 1, 2, 1, 3, 1, 1, 2, 9, 1, 1, 0, 2, 2, 4, 4, 11, 6, 7, 3, 6, 9,
            4, 5, 46, 18, 8, 18, 17, 3, 8, 20, 16, 10, 3, 7, 175, 4, 6, 7, 13,
            3, 7, 3, 3, 1, 3, 3, 10, 3, 1, 5, 2, 0, 1, 2, 16, 3, 5, 1, 6, 1,
            1, 2, 58, 20, 3, 14, 12, 2, 1, 3, 16, 3, 5, 8, 3, 1, 8, 6, 17, 6,
            5, 3, 8, 6, 13, 175,
        ];
        TABLE[usize::from(byte)]
    }
}

trait IteratorExt: Iterator {
    /// Like `Iterator::count`, but guarantees that it gets the count by
    /// iterating over each element without taking any specialized shortcuts.
    ///
    /// We do this because the memchr crate may specialize `count` in certain
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
