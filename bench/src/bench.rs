use criterion::{
    criterion_group, criterion_main, Bencher, Criterion, Throughput,
};

mod data;
mod memchr;
mod memmem;

fn all(c: &mut Criterion) {
    memchr::all(c);
    memmem::all(c);
}

/// A convenience function for defining a Criterion benchmark using our own
/// conventions and a common config.
///
/// Note that we accept `bench` as a boxed closure to avoid the costs
/// of monomorphization. Particularly with the memchr benchmarks, this
/// function getting monomorphized (which also monomorphizes via Criterion's
/// `bench_function`) bloats compile times dramatically (by an order of
/// magnitude). This is okay to do since `bench` isn't the actual thing we
/// measure. The measurement comes from running `Bencher::iter` from within`
/// bench. So the dynamic dispatch is okay here.
fn define(
    c: &mut Criterion,
    name: &str,
    corpus: &[u8],
    bench: Box<dyn FnMut(&mut Bencher<'_>) + 'static>,
) {
    // I don't really "get" the whole Criterion benchmark group thing. I just
    // want a flat namespace to define all benchmarks. The only thing that
    // matters to me is that we can group benchmarks arbitrarily using the
    // name only. So we play Criterion's game by splitting our benchmark name
    // on the first flash.
    //
    // N.B. We don't include the slash, since Criterion automatically adds it.
    let mut it = name.splitn(2, "/");
    let (group_name, bench_name) = (it.next().unwrap(), it.next().unwrap());
    c.benchmark_group(group_name)
        .throughput(Throughput::Bytes(corpus.len() as u64))
        .sample_size(10)
        .warm_up_time(std::time::Duration::from_millis(500))
        .measurement_time(std::time::Duration::from_secs(2))
        .bench_function(bench_name, bench);
}

criterion_group!(does_not_matter, all);
criterion_main!(does_not_matter);
