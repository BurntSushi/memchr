use criterion::{
    criterion_group, criterion_main, Bencher, Benchmark, Criterion, Throughput,
};

mod data;
mod memchr;

fn all(c: &mut Criterion) {
    memchr::all(c);
}

fn define(
    c: &mut Criterion,
    group_name: &str,
    bench_name: &str,
    corpus: &[u8],
    bench: impl FnMut(&mut Bencher<'_>) + 'static,
) {
    let tput = Throughput::Bytes(corpus.len() as u64);
    // let benchmark = Benchmark::new(bench_name, bench).throughput(tput);

    let benchmark = Benchmark::new(bench_name, bench)
        .throughput(tput)
        .sample_size(30)
        .warm_up_time(std::time::Duration::from_millis(500))
        .measurement_time(std::time::Duration::from_secs(2));
    c.bench(group_name, benchmark);
}

criterion_group!(does_not_matter, all);
criterion_main!(does_not_matter);
