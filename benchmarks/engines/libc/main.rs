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
        // FIXME: How do we get libc version?
        writeln!(std::io::stdout(), "unknown")?;
        return Ok(());
    }
    let engine = arg;
    let b = Benchmark::from_stdin()?;
    let samples = match (&*engine, &*b.model) {
        ("memmem-oneshot", "count") => memmem_oneshot_count(&b)?,
        ("memchr-oneshot", "count-bytes") => memchr_oneshot_count(&b)?,
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

fn memchr_oneshot_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let needle = b.one_needle_byte()?;
    shared::run(b, || Ok(shared::count_memchr(haystack, needle, libc_memchr)))
}

fn memmem_oneshot_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let needle = b.one_needle()?;
    shared::run(b, || Ok(shared::count_memmem(haystack, needle, libc_memmem)))
}

/// A safe wrapper around libc's `memchr` function. In particular, this
/// converts memchr's pointer return to an index offset into `haystack`.
fn libc_memchr(haystack: &[u8], needle: u8) -> Option<usize> {
    // SAFETY: This is safe to call since all pointers are valid.
    let p = unsafe {
        libc::memchr(
            haystack.as_ptr().cast(),
            needle as libc::c_int,
            haystack.len(),
        )
    };
    if p.is_null() {
        None
    } else {
        Some(p as usize - (haystack.as_ptr() as usize))
    }
}

/// A safe wrapper around libc's `memmem` function. In particular, this
/// converts memmem's pointer return to an index offset into `haystack`.
fn libc_memmem(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    // SAFETY: We know that both our haystack and needle pointers are valid and
    // non-null, and we also know that the lengths of each corresponds to the
    // number of bytes at that memory region.
    let p = unsafe {
        libc::memmem(
            haystack.as_ptr().cast(),
            haystack.len(),
            needle.as_ptr().cast(),
            needle.len(),
        )
    };
    if p.is_null() {
        None
    } else {
        let start = (p as isize) - (haystack.as_ptr() as isize);
        Some(start as usize)
    }
}
