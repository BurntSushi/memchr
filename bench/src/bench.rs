extern crate core;
#[macro_use]
extern crate criterion;
#[cfg(target_arch = "x86_64")]
extern crate libc;
extern crate memchr;

use criterion::{Bencher, Benchmark, Criterion, Throughput};

use imp::{
    memchr1_count, memchr2_count, memchr3_count,
    memrchr1_count, memrchr2_count, memrchr3_count,
    fallback1_count, fallback2_count, fallback3_count,
    naive1_count, naive2_count, naive3_count,
};
use inputs::{
    Input, Search1, Search2, Search3,
    HUGE, SMALL, TINY, EMPTY,
};

#[cfg(target_arch = "x86_64")]
#[path = "../../src/c.rs"]
mod c;
#[path = "../../src/fallback.rs"]
#[allow(dead_code)]
mod fallback;
mod imp;
mod inputs;
#[path = "../../src/naive.rs"]
mod naive;

fn all(c: &mut Criterion) {
    define_input1(c, "memchr1/rust/huge", HUGE, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                memchr1_count(search.byte1.byte, search.corpus),
            );
        });
    });
    define_input1(c, "memchr1/rust/small", SMALL, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                memchr1_count(search.byte1.byte, search.corpus),
            );
        });
    });
    define_input1(c, "memchr1/rust/tiny", TINY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                memchr1_count(search.byte1.byte, search.corpus),
            );
        });
    });
    define_input1(c, "memchr1/rust/empty", EMPTY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                memchr1_count(search.byte1.byte, search.corpus),
            );
        });
    });

    #[cfg(target_arch = "x86_64")]
    {
        define_input1(c, "memchr1/libc/huge", HUGE, move |search, b| {
            b.iter(|| {
                assert_eq!(
                    search.byte1.count,
                    imp::memchr1_libc_count(search.byte1.byte, search.corpus),
                );
            });
        });
        define_input1(c, "memchr1/libc/small", SMALL, move |search, b| {
            b.iter(|| {
                assert_eq!(
                    search.byte1.count,
                    imp::memchr1_libc_count(search.byte1.byte, search.corpus),
                );
            });
        });
        define_input1(c, "memchr1/libc/tiny", TINY, move |search, b| {
            b.iter(|| {
                assert_eq!(
                    search.byte1.count,
                    imp::memchr1_libc_count(search.byte1.byte, search.corpus),
                );
            });
        });
        define_input1(c, "memchr1/libc/empty", EMPTY, move |search, b| {
            b.iter(|| {
                assert_eq!(
                    search.byte1.count,
                    imp::memchr1_libc_count(search.byte1.byte, search.corpus),
                );
            });
        });
    }

    define_input1(c, "memchr1/fallback/huge", HUGE, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                fallback1_count(search.byte1.byte, search.corpus),
            );
        });
    });
    define_input1(c, "memchr1/fallback/small", SMALL, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                fallback1_count(search.byte1.byte, search.corpus),
            );
        });
    });
    define_input1(c, "memchr1/fallback/tiny", TINY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                fallback1_count(search.byte1.byte, search.corpus),
            );
        });
    });
    define_input1(c, "memchr1/fallback/empty", EMPTY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                fallback1_count(search.byte1.byte, search.corpus),
            );
        });
    });

    define_input1(c, "memchr1/naive/huge", HUGE, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                naive1_count(search.byte1.byte, search.corpus),
            );
        });
    });
    define_input1(c, "memchr1/naive/small", SMALL, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                naive1_count(search.byte1.byte, search.corpus),
            );
        });
    });
    define_input1(c, "memchr1/naive/tiny", TINY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                naive1_count(search.byte1.byte, search.corpus),
            );
        });
    });
    define_input1(c, "memchr1/naive/empty", EMPTY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                naive1_count(search.byte1.byte, search.corpus),
            );
        });
    });

    define_input2(c, "memchr2/rust/huge", HUGE, move |search, b| {
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
    define_input2(c, "memchr2/rust/small", SMALL, move |search, b| {
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
    define_input2(c, "memchr2/rust/tiny", TINY, move |search, b| {
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
    define_input2(c, "memchr2/rust/empty", EMPTY, move |search, b| {
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

    define_input2(c, "memchr2/fallback/huge", HUGE, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count,
                fallback2_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.corpus,
                )
            );
        });
    });
    define_input2(c, "memchr2/fallback/small", SMALL, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count,
                fallback2_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.corpus,
                )
            );
        });
    });
    define_input2(c, "memchr2/fallback/tiny", TINY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count,
                fallback2_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.corpus,
                )
            );
        });
    });
    define_input2(c, "memchr2/fallback/empty", EMPTY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count,
                fallback2_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.corpus,
                )
            );
        });
    });

    define_input2(c, "memchr2/naive/huge", HUGE, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count,
                naive2_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.corpus,
                )
            );
        });
    });
    define_input2(c, "memchr2/naive/small", SMALL, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count,
                naive2_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.corpus,
                )
            );
        });
    });
    define_input2(c, "memchr2/naive/tiny", TINY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count,
                naive2_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.corpus,
                )
            );
        });
    });
    define_input2(c, "memchr2/naive/empty", EMPTY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count,
                naive2_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.corpus,
                )
            );
        });
    });

    define_input3(c, "memchr3/rust/huge", HUGE, move |search, b| {
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
    define_input3(c, "memchr3/rust/small", SMALL, move |search, b| {
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
    define_input3(c, "memchr3/rust/tiny", TINY, move |search, b| {
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
    define_input3(c, "memchr3/rust/empty", EMPTY, move |search, b| {
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

    define_input3(c, "memchr3/fallback/huge", HUGE, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count + search.byte3.count,
                fallback3_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.byte3.byte,
                    search.corpus,
                )
            );
        });
    });
    define_input3(c, "memchr3/fallback/small", SMALL, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count + search.byte3.count,
                fallback3_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.byte3.byte,
                    search.corpus,
                )
            );
        });
    });
    define_input3(c, "memchr3/fallback/tiny", TINY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count + search.byte3.count,
                fallback3_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.byte3.byte,
                    search.corpus,
                )
            );
        });
    });
    define_input3(c, "memchr3/fallback/empty", EMPTY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count + search.byte3.count,
                fallback3_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.byte3.byte,
                    search.corpus,
                )
            );
        });
    });

    define_input3(c, "memchr3/naive/huge", HUGE, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count + search.byte3.count,
                naive3_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.byte3.byte,
                    search.corpus,
                )
            );
        });
    });
    define_input3(c, "memchr3/naive/small", SMALL, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count + search.byte3.count,
                naive3_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.byte3.byte,
                    search.corpus,
                )
            );
        });
    });
    define_input3(c, "memchr3/naive/tiny", TINY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count + search.byte3.count,
                naive3_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.byte3.byte,
                    search.corpus,
                )
            );
        });
    });
    define_input3(c, "memchr3/naive/empty", EMPTY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count + search.byte3.count,
                naive3_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.byte3.byte,
                    search.corpus,
                )
            );
        });
    });

    define_input1(c, "memrchr1/rust/huge", HUGE, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                memrchr1_count(search.byte1.byte, search.corpus)
            );
        });
    });
    define_input1(c, "memrchr1/rust/small", SMALL, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                memrchr1_count(search.byte1.byte, search.corpus)
            );
        });
    });
    define_input1(c, "memrchr1/rust/tiny", TINY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                memrchr1_count(search.byte1.byte, search.corpus)
            );
        });
    });
    define_input1(c, "memrchr1/rust/empty", EMPTY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                memrchr1_count(search.byte1.byte, search.corpus)
            );
        });
    });

    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    {
        define_input1(c, "memrchr1/libc/huge", HUGE, move |search, b| {
            b.iter(|| {
                assert_eq!(
                    search.byte1.count,
                    imp::memrchr1_libc_count(search.byte1.byte, search.corpus)
                );
            });
        });
        define_input1(c, "memrchr1/libc/small", SMALL, move |search, b| {
            b.iter(|| {
                assert_eq!(
                    search.byte1.count,
                    imp::memrchr1_libc_count(search.byte1.byte, search.corpus)
                );
            });
        });
        define_input1(c, "memrchr1/libc/tiny", TINY, move |search, b| {
            b.iter(|| {
                assert_eq!(
                    search.byte1.count,
                    imp::memrchr1_libc_count(search.byte1.byte, search.corpus)
                );
            });
        });
        define_input1(c, "memrchr1/libc/empty", EMPTY, move |search, b| {
            b.iter(|| {
                assert_eq!(
                    search.byte1.count,
                    imp::memrchr1_libc_count(search.byte1.byte, search.corpus)
                );
            });
        });
    }

    define_input2(c, "memrchr2/rust/huge", HUGE, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count,
                memrchr2_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.corpus,
                )
            );
        });
    });
    define_input2(c, "memrchr2/rust/small", SMALL, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count,
                memrchr2_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.corpus,
                )
            );
        });
    });
    define_input2(c, "memrchr2/rust/tiny", TINY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count,
                memrchr2_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.corpus,
                )
            );
        });
    });
    define_input2(c, "memrchr2/rust/empty", EMPTY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count,
                memrchr2_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.corpus,
                )
            );
        });
    });

    define_input3(c, "memrchr3/rust/huge", HUGE, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count + search.byte3.count,
                memrchr3_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.byte3.byte,
                    search.corpus,
                )
            );
        });
    });
    define_input3(c, "memrchr3/rust/small", SMALL, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count + search.byte3.count,
                memrchr3_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.byte3.byte,
                    search.corpus,
                )
            );
        });
    });
    define_input3(c, "memrchr3/rust/tiny", TINY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count + search.byte3.count,
                memrchr3_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.byte3.byte,
                    search.corpus,
                )
            );
        });
    });
    define_input3(c, "memrchr3/rust/empty", EMPTY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count + search.byte3.count,
                memrchr3_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.byte3.byte,
                    search.corpus,
                )
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
