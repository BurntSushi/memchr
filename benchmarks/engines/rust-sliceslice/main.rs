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
    let mut stdout = std::io::stdout().lock();
    for s in samples.iter() {
        writeln!(stdout, "{},{}", s.duration.as_nanos(), s.count)?;
    }
    Ok(())
}

// The sliceslice crate perplexingly only offers "does the needle occur at
// least once in the haystack" APIs, and does not actually report the offset
// at which the needle matches. It's therefore questionable to even bother
// benchmarking it, but alas, it is quite fast and has been heavily optimized.
// But this does mean we can only use sliceslice in benchmarks with a count
// equal to 0 or 1. (We could define a distinct "does any match occur" model,
// but it does not seem worth it.)

fn memmem_prebuilt_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    #[cfg(target_arch = "x86_64")]
    {
        use sliceslice::x86::DynamicAvx2Searcher;
        anyhow::ensure!(
            is_x86_feature_detected!("avx2"),
            "AVX2 not available"
        );
        let haystack = &b.haystack;
        let needle = b.one_needle()?;
        // SAFETY: We just checked that avx2 is available.
        unsafe {
            let finder = DynamicAvx2Searcher::new(needle);
            shared::run(b, || {
                Ok(if finder.search_in(haystack) { 1 } else { 0 })
            })
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        use sliceslice::aarch64::NeonSearcher;
        anyhow::ensure!(
            std::arch::is_aarch64_feature_detected!("neon"),
            "NEON not available"
        );
        let haystack = &b.haystack;
        let needle = b.one_needle()?;
        // SAFETY: We just checked that avx2 is available.
        unsafe {
            let finder = NeonSearcher::new(needle);
            shared::run(b, || {
                Ok(if finder.search_in(haystack) { 1 } else { 0 })
            })
        }
    }
}

fn memmem_prebuilt_needles(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    #[cfg(target_arch = "x86_64")]
    {
        use sliceslice::x86::DynamicAvx2Searcher;

        anyhow::ensure!(
            is_x86_feature_detected!("avx2"),
            "AVX2 not available"
        );
        let finders = b
            .needles
            .iter()
            // SAFETY: We just checked that avx2 is available.
            .map(|n| unsafe { DynamicAvx2Searcher::new(n) })
            .collect::<Vec<_>>();
        shared::run(b, || {
            let mut count = 0;
            for (i, finder) in finders.iter().enumerate() {
                for haystack in b.needles[i..].iter() {
                    // SAFETY: We just checked that avx2 is available.
                    unsafe {
                        if finder.search_in(haystack) {
                            count += 1;
                        }
                    }
                }
            }
            Ok(count)
        })
    }
    #[cfg(target_arch = "aarch64")]
    {
        use sliceslice::aarch64::NeonSearcher;

        anyhow::ensure!(
            std::arch::is_aarch64_feature_detected!("neon"),
            "NEON not available"
        );
        let finders = b
            .needles
            .iter()
            // SAFETY: We just checked that avx2 is available.
            .map(|n| unsafe { NeonSearcher::new(n) })
            .collect::<Vec<_>>();
        shared::run(b, || {
            let mut count = 0;
            for (i, finder) in finders.iter().enumerate() {
                for haystack in b.needles[i..].iter() {
                    // SAFETY: We just checked that avx2 is available.
                    unsafe {
                        if finder.search_in(haystack) {
                            count += 1;
                        }
                    }
                }
            }
            Ok(count)
        })
    }
}

fn memmem_prebuilt_haystack(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    #[cfg(target_arch = "x86_64")]
    {
        use sliceslice::x86::DynamicAvx2Searcher;

        anyhow::ensure!(
            is_x86_feature_detected!("avx2"),
            "AVX2 not available"
        );
        let haystack = &b.haystack;
        let finders = b
            .needles
            .iter()
            // SAFETY: We just checked that avx2 is available.
            .map(|n| unsafe { DynamicAvx2Searcher::new(n) })
            .collect::<Vec<_>>();
        shared::run(b, || {
            let mut count = 0;
            for finder in finders.iter() {
                // SAFETY: We just checked that avx2 is available.
                unsafe {
                    if finder.search_in(haystack) {
                        count += 1;
                    }
                }
            }
            Ok(count)
        })
    }
    #[cfg(target_arch = "aarch64")]
    {
        use sliceslice::aarch64::NeonSearcher;

        anyhow::ensure!(
            std::arch::is_aarch64_feature_detected!("neon"),
            "NEON not available"
        );
        let haystack = &b.haystack;
        let finders = b
            .needles
            .iter()
            // SAFETY: We just checked that avx2 is available.
            .map(|n| unsafe { NeonSearcher::new(n) })
            .collect::<Vec<_>>();
        shared::run(b, || {
            let mut count = 0;
            for finder in finders.iter() {
                // SAFETY: We just checked that avx2 is available.
                unsafe {
                    if finder.search_in(haystack) {
                        count += 1;
                    }
                }
            }
            Ok(count)
        })
    }
}

fn memmem_oneshot_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    #[cfg(target_arch = "x86_64")]
    {
        use sliceslice::x86::DynamicAvx2Searcher;

        anyhow::ensure!(
            is_x86_feature_detected!("avx2"),
            "AVX2 not available"
        );
        let haystack = &b.haystack;
        let needle = b.one_needle()?;
        shared::run(b, || {
            // SAFETY: We just checked that avx2 is available.
            Ok(unsafe {
                if DynamicAvx2Searcher::new(needle).search_in(haystack) {
                    1
                } else {
                    0
                }
            })
        })
    }
    #[cfg(target_arch = "aarch64")]
    {
        use sliceslice::aarch64::NeonSearcher;

        anyhow::ensure!(
            std::arch::is_aarch64_feature_detected!("neon"),
            "NEON not available"
        );
        let haystack = &b.haystack;
        let needle = b.one_needle()?;
        shared::run(b, || {
            // SAFETY: We just checked that avx2 is available.
            Ok(unsafe {
                if NeonSearcher::new(needle).search_in(haystack) {
                    1
                } else {
                    0
                }
            })
        })
    }
}
