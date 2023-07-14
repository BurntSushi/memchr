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
    shared::run(b, || Ok(bytecount::count(haystack, needle)))
}
