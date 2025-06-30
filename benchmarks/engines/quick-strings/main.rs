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
        ("memmem-oneshot", "count") => memmem_oneshot_count(&b)?,
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
        Ok(if quick_strings::contains(haystack, needle) { 1 } else { 0 })
    })
}
