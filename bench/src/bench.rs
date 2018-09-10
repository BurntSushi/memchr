#![allow(dead_code)]

#[macro_use]
extern crate criterion;
extern crate memchr;

use criterion::{Bencher, Benchmark, Criterion, Throughput};

use imp::{
    memchr1_count, memchr2_count, memchr3_count,
    memrchr1_count,
    naive1_count,
};
use inputs::{
    Input, Search1, Search2, Search3,
    HUGE, SMALL, TINY, EMPTY,
};

mod imp;
mod inputs;

fn all(c: &mut Criterion) {
    define_input1(c, "memchr1/huge", HUGE, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                memchr1_count(search.byte1.byte, search.corpus),
            );
        });
    });
    define_input1(c, "memchr1/small", SMALL, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                memchr1_count(search.byte1.byte, search.corpus),
            );
        });
    });
    define_input1(c, "memchr1/tiny", TINY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                memchr1_count(search.byte1.byte, search.corpus),
            );
        });
    });
    define_input1(c, "memchr1/empty", EMPTY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                memchr1_count(search.byte1.byte, search.corpus),
            );
        });
    });

    define_input2(c, "memchr2/huge", HUGE, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count,
                memchr2_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.corpus,
                )
            );
        });
    });
    define_input2(c, "memchr2/small", SMALL, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count,
                memchr2_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.corpus,
                )
            );
        });
    });
    define_input2(c, "memchr2/tiny", TINY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count,
                memchr2_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.corpus,
                )
            );
        });
    });
    define_input2(c, "memchr2/empty", EMPTY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count,
                memchr2_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.corpus,
                )
            );
        });
    });

    define_input3(c, "memchr3/huge", HUGE, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count + search.byte3.count,
                memchr3_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.byte3.byte,
                    search.corpus,
                )
            );
        });
    });
    define_input3(c, "memchr3/small", SMALL, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count + search.byte3.count,
                memchr3_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.byte3.byte,
                    search.corpus,
                )
            );
        });
    });
    define_input3(c, "memchr3/tiny", TINY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count + search.byte3.count,
                memchr3_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.byte3.byte,
                    search.corpus,
                )
            );
        });
    });
    define_input3(c, "memchr3/empty", EMPTY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count + search.byte3.count,
                memchr3_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.byte3.byte,
                    search.corpus,
                )
            );
        });
    });

    define_input1(c, "memrchr1/huge", HUGE, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                memrchr1_count(search.byte1.byte, search.corpus)
            );
        });
    });
    define_input1(c, "memrchr1/small", SMALL, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                memrchr1_count(search.byte1.byte, search.corpus)
            );
        });
    });
    define_input1(c, "memrchr1/tiny", TINY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                memrchr1_count(search.byte1.byte, search.corpus)
            );
        });
    });
    define_input1(c, "memrchr1/empty", EMPTY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                memrchr1_count(search.byte1.byte, search.corpus)
            );
        });
    });

    define_input1(c, "naive1/huge", HUGE, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                naive1_count(search.byte1.byte, search.corpus),
            );
        });
    });
    define_input1(c, "naive1/small", SMALL, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                naive1_count(search.byte1.byte, search.corpus),
            );
        });
    });
    define_input1(c, "naive1/tiny", TINY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                naive1_count(search.byte1.byte, search.corpus),
            );
        });
    });
    define_input1(c, "naive1/empty", EMPTY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                naive1_count(search.byte1.byte, search.corpus),
            );
        });
    });
}

fn define_input1<'i>(
    c: &mut Criterion,
    group: &str,
    input: Input,
    bench: impl FnMut(Search1, &mut Bencher) + Clone + 'static,
) {
    if let Some(search) = input.never1() {
        let mut bench = bench.clone();
        define(c, group, "never", input.corpus, move |b| {
            bench(search, b)
        });
    }
    if let Some(search) = input.rare1() {
        let mut bench = bench.clone();
        define(c, group, "rare", input.corpus, move |b| {
            bench(search, b)
        });
    }
    if let Some(search) = input.uncommon1() {
        let mut bench = bench.clone();
        define(c, group, "uncommon", input.corpus, move |b| {
            bench(search, b)
        });
    }
    if let Some(search) = input.common1() {
        let mut bench = bench.clone();
        define(c, group, "common", input.corpus, move |b| {
            bench(search, b)
        });
    }
    if let Some(search) = input.verycommon1() {
        let mut bench = bench.clone();
        define(c, group, "verycommon", input.corpus, move |b| {
            bench(search, b)
        });
    }
    if let Some(search) = input.supercommon1() {
        let mut bench = bench.clone();
        define(c, group, "supercommon", input.corpus, move |b| {
            bench(search, b)
        });
    }
}

fn define_input2<'i>(
    c: &mut Criterion,
    group: &str,
    input: Input,
    bench: impl FnMut(Search2, &mut Bencher) + Clone + 'static,
) {
    if let Some(search) = input.never2() {
        let mut bench = bench.clone();
        define(c, group, "never", input.corpus, move |b| {
            bench(search, b)
        });
    }
    if let Some(search) = input.rare2() {
        let mut bench = bench.clone();
        define(c, group, "rare", input.corpus, move |b| {
            bench(search, b)
        });
    }
    if let Some(search) = input.uncommon2() {
        let mut bench = bench.clone();
        define(c, group, "uncommon", input.corpus, move |b| {
            bench(search, b)
        });
    }
    if let Some(search) = input.common2() {
        let mut bench = bench.clone();
        define(c, group, "common", input.corpus, move |b| {
            bench(search, b)
        });
    }
    if let Some(search) = input.verycommon2() {
        let mut bench = bench.clone();
        define(c, group, "verycommon", input.corpus, move |b| {
            bench(search, b)
        });
    }
    if let Some(search) = input.supercommon2() {
        let mut bench = bench.clone();
        define(c, group, "supercommon", input.corpus, move |b| {
            bench(search, b)
        });
    }
}

fn define_input3<'i>(
    c: &mut Criterion,
    group: &str,
    input: Input,
    bench: impl FnMut(Search3, &mut Bencher) + Clone + 'static,
) {
    if let Some(search) = input.never3() {
        let mut bench = bench.clone();
        define(c, group, "never", input.corpus, move |b| {
            bench(search, b)
        });
    }
    if let Some(search) = input.rare3() {
        let mut bench = bench.clone();
        define(c, group, "rare", input.corpus, move |b| {
            bench(search, b)
        });
    }
    if let Some(search) = input.uncommon3() {
        let mut bench = bench.clone();
        define(c, group, "uncommon", input.corpus, move |b| {
            bench(search, b)
        });
    }
    if let Some(search) = input.common3() {
        let mut bench = bench.clone();
        define(c, group, "common", input.corpus, move |b| {
            bench(search, b)
        });
    }
    if let Some(search) = input.verycommon3() {
        let mut bench = bench.clone();
        define(c, group, "verycommon", input.corpus, move |b| {
            bench(search, b)
        });
    }
    if let Some(search) = input.supercommon3() {
        let mut bench = bench.clone();
        define(c, group, "supercommon", input.corpus, move |b| {
            bench(search, b)
        });
    }
}

fn define(
    c: &mut Criterion,
    group_name: &str,
    bench_name: &str,
    corpus: &[u8],
    bench: impl FnMut(&mut Bencher) + 'static,
) {
    let tput = Throughput::Bytes(corpus.len() as u32);
    let benchmark = Benchmark::new(bench_name, bench).throughput(tput);
    c.bench(group_name, benchmark);
}

criterion_group!(does_not_matter, all);
criterion_main!(does_not_matter);
