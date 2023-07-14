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
        writeln!(std::io::stdout(), env!("RUSTC_VERSION"))?;
        return Ok(());
    }
    let engine = arg;
    let b = Benchmark::from_stdin()?;
    let samples = match (&*engine, &*b.model) {
        ("memmem-oneshot", "count") => memmem_oneshot_count(&b)?,
        ("memmem-prebuilt", "count") => memmem_prebuilt_count(&b)?,
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

fn memmem_oneshot_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = std::str::from_utf8(&b.haystack)?;
    let needle = std::str::from_utf8(b.one_needle()?)?;
    shared::run(b, || {
        Ok(shared::count_memmem_str(haystack, needle, |h, n| {
            h.match_indices(n).map(|(i, _)| i).next()
        }))
    })
}

/// A prebuilt version of the std searcher. This isn't quite as "prebuilt" as
/// is possible, since each measurement doesn't just include the creation of
/// the iterator but also the searcher. Where as the memchr crate can do this
/// where the searcher is created once and the measurement only includes the
/// creation of the iterator.
///
/// So probably we "should" have created some other kind of engine called
/// "partial prebuilt" but it makes things even more confusing. At some point,
/// this is just the way the cookie crumbles.
///
/// Remember, it's not that oneshot and prebuilt aren't comparable.
/// It's absolutely fair to say, for example, "Rust's standard library
/// substring search is slow because it doesn't let one amortize the cost of
/// construction." But it's *also* interesting to make these assumptions a
/// little more explicit in our measurement model. But we do this within
/// reason.
fn memmem_prebuilt_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = std::str::from_utf8(&b.haystack)?;
    let needle = std::str::from_utf8(b.one_needle()?)?;
    shared::run(b, || Ok(haystack.matches(needle).count()))
}
