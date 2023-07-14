use std::io::Write;

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
        ("memchr-naive", "count-bytes") => memchr_naive_count(&b)?,
        ("memchr2", "count-bytes") => memchr2_count(&b)?,
        ("memchr3", "count-bytes") => memchr3_count(&b)?,
        ("memrchr", "count-bytes") => memrchr_count(&b)?,
        ("memrchr2", "count-bytes") => memrchr2_count(&b)?,
        ("memrchr3", "count-bytes") => memrchr3_count(&b)?,
        ("memmem-prebuilt", "count") => memmem_prebuilt_count(&b)?,
        ("memmem-oneshot", "count") => memmem_oneshot_count(&b)?,
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
    shared::run(b, || Ok(memchr::memchr_iter(needle, haystack).count()))
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
    shared::run(b, || Ok(memchr::memchr2_iter(n1, n2, haystack).count()))
}

fn memchr3_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let (n1, n2, n3) = b.three_needle_bytes()?;
    shared::run(b, || Ok(memchr::memchr3_iter(n1, n2, n3, haystack).count()))
}

fn memrchr_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let needle = b.one_needle_byte()?;
    shared::run(b, || Ok(memchr::memchr_iter(needle, haystack).rev().count()))
}

fn memrchr2_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let (n1, n2) = b.two_needle_bytes()?;
    shared::run(b, || Ok(memchr::memchr2_iter(n1, n2, haystack).rev().count()))
}

fn memrchr3_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let (n1, n2, n3) = b.three_needle_bytes()?;
    shared::run(b, || {
        Ok(memchr::memchr3_iter(n1, n2, n3, haystack).rev().count())
    })
}

fn memmem_prebuilt_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let needle = b.one_needle()?;
    let finder = memchr::memmem::Finder::new(needle);
    shared::run(b, || Ok(finder.find_iter(haystack).count()))
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
