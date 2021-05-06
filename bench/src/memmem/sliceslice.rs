/*
These benchmarks were lifted almost verbtaim out of the sliceslice crate. The
reason why we have these benchmarks is because they were the primary thing that
motivated me to write this particular memmem implementation. In particular, my
existing substring search implementation in the bstr crate did quite poorly
on these particular benchmarks. Moreover, while the benchmark setup is a little
weird, these benchmarks do reflect cases that I think are somewhat common:

N.B. In the sliceslice crate, the benchmarks are called "short" and "long."
Here, we call them sliceslice-words/words and sliceslice-i386/words,
respectively. The name change was made to be consistent with the naming
convention used for other benchmarks.

* In the sliceslice-words/words case, the benchmark is primarily about
  searching very short haystacks using common English words.
* In the sliceslice-words/i386 case, the benchmark is primarily about searching
  a longer haystack with common English words.

The main thing that's "weird" about these benchmarks is that each iteration
involves a lot of work. All of the other benchmarks in this crate focus on one
specific needle with one specific haystack, and each iteration is a single
search or iteration. But in these benchmarks, each iteration involves searching
with many needles against potentially many haystacks. Nevertheless, these have
proven useful targets for optimization.
*/
use criterion::{black_box, Criterion};
use memchr::memmem;

use crate::{data::*, define};

pub fn all(c: &mut Criterion) {
    search_short_haystack(c);
    search_long_haystack(c);
    search_i386_haystack(c);
}

fn search_short_haystack(c: &mut Criterion) {
    let mut words = SLICESLICE_WORDS.lines().collect::<Vec<_>>();
    words.sort_unstable_by_key(|word| word.len());
    let words: Vec<&str> = words.iter().map(|&s| s).collect();

    let needles = words.clone();
    define(
        c,
        "memmem/krate/prebuilt/sliceslice-words/words",
        &[],
        Box::new(move |b| {
            let searchers = needles
                .iter()
                .map(|needle| memmem::Finder::new(needle.as_bytes()))
                .collect::<Vec<_>>();
            b.iter(|| {
                for (i, searcher) in searchers.iter().enumerate() {
                    for haystack in &needles[i..] {
                        black_box(
                            searcher.find(haystack.as_bytes()).is_some(),
                        );
                    }
                }
            });
        }),
    );

    let needles = words.clone();
    define(
        c,
        "memmem/krate_nopre/prebuilt/sliceslice-words/words",
        &[],
        Box::new(move |b| {
            let searchers = needles
                .iter()
                .map(|needle| {
                    memmem::FinderBuilder::new()
                        .prefilter(memmem::Prefilter::None)
                        .build_forward(needle)
                })
                .collect::<Vec<_>>();
            b.iter(|| {
                for (i, searcher) in searchers.iter().enumerate() {
                    for haystack in &needles[i..] {
                        black_box(
                            searcher.find(haystack.as_bytes()).is_some(),
                        );
                    }
                }
            });
        }),
    );

    let needles = words.clone();
    define(
        c,
        "memmem/stud/prebuilt/sliceslice-words/words",
        &[],
        Box::new(move |b| {
            b.iter(|| {
                for (i, needle) in needles.iter().enumerate() {
                    for haystack in &needles[i..] {
                        black_box(haystack.contains(needle));
                    }
                }
            });
        }),
    );

    #[cfg(target_arch = "x86_64")]
    {
        use sliceslice::x86::DynamicAvx2Searcher;

        let needles = words.clone();
        define(
            c,
            "memmem/sliceslice/prebuilt/sliceslice-words/words",
            &[],
            Box::new(move |b| {
                let searchers = needles
                    .iter()
                    .map(|&needle| unsafe {
                        DynamicAvx2Searcher::new(needle.as_bytes())
                    })
                    .collect::<Vec<_>>();

                b.iter(|| {
                    for (i, searcher) in searchers.iter().enumerate() {
                        for haystack in &needles[i..] {
                            black_box(unsafe {
                                searcher.search_in(haystack.as_bytes())
                            });
                        }
                    }
                });
            }),
        );
    }
}

fn search_long_haystack(c: &mut Criterion) {
    let words: Vec<&str> = SLICESLICE_WORDS.lines().collect();
    let haystack = SLICESLICE_HAYSTACK;
    let needles = words.clone();
    define(
        c,
        "memmem/krate/prebuilt/sliceslice-haystack/words",
        &[],
        Box::new(move |b| {
            let searchers = needles
                .iter()
                .map(|needle| memmem::Finder::new(needle.as_bytes()))
                .collect::<Vec<_>>();
            b.iter(|| {
                for searcher in searchers.iter() {
                    black_box(searcher.find(haystack.as_bytes()).is_some());
                }
            });
        }),
    );

    let needles = words.clone();
    define(
        c,
        "memmem/krate_nopre/prebuilt/sliceslice-haystack/words",
        &[],
        Box::new(move |b| {
            let searchers = needles
                .iter()
                .map(|needle| {
                    memmem::FinderBuilder::new()
                        .prefilter(memmem::Prefilter::None)
                        .build_forward(needle)
                })
                .collect::<Vec<_>>();
            b.iter(|| {
                for searcher in searchers.iter() {
                    black_box(searcher.find(haystack.as_bytes()).is_some());
                }
            });
        }),
    );

    let needles = words.clone();
    define(
        c,
        "memmem/stud/prebuilt/sliceslice-haystack/words",
        &[],
        Box::new(move |b| {
            b.iter(|| {
                for needle in needles.iter() {
                    black_box(haystack.contains(needle));
                }
            });
        }),
    );

    #[cfg(target_arch = "x86_64")]
    {
        use sliceslice::x86::DynamicAvx2Searcher;

        let needles = words.clone();
        define(
            c,
            "memmem/sliceslice/prebuilt/sliceslice-haystack/words",
            &[],
            Box::new(move |b| {
                let searchers = needles
                    .iter()
                    .map(|needle| unsafe {
                        DynamicAvx2Searcher::new(needle.as_bytes())
                    })
                    .collect::<Vec<_>>();

                b.iter(|| {
                    for searcher in &searchers {
                        black_box(unsafe {
                            searcher.search_in(haystack.as_bytes())
                        });
                    }
                });
            }),
        );
    }
}

fn search_i386_haystack(c: &mut Criterion) {
    let words: Vec<&str> = SLICESLICE_WORDS.lines().collect();
    let haystack = SLICESLICE_I386;
    let needles = words.clone();
    define(
        c,
        "memmem/krate/prebuilt/sliceslice-i386/words",
        &[],
        Box::new(move |b| {
            let searchers = needles
                .iter()
                .map(|needle| memmem::Finder::new(needle.as_bytes()))
                .collect::<Vec<_>>();
            b.iter(|| {
                for searcher in searchers.iter() {
                    black_box(searcher.find(haystack.as_bytes()).is_some());
                }
            });
        }),
    );

    let needles = words.clone();
    define(
        c,
        "memmem/krate_nopre/prebuilt/sliceslice-i386/words",
        &[],
        Box::new(move |b| {
            let searchers = needles
                .iter()
                .map(|needle| {
                    memmem::FinderBuilder::new()
                        .prefilter(memmem::Prefilter::None)
                        .build_forward(needle)
                })
                .collect::<Vec<_>>();
            b.iter(|| {
                for searcher in searchers.iter() {
                    black_box(searcher.find(haystack.as_bytes()).is_some());
                }
            });
        }),
    );

    let needles = words.clone();
    define(
        c,
        "memmem/stud/prebuilt/sliceslice-i386/words",
        &[],
        Box::new(move |b| {
            b.iter(|| {
                for needle in needles.iter() {
                    black_box(haystack.contains(needle));
                }
            });
        }),
    );

    #[cfg(target_arch = "x86_64")]
    {
        use sliceslice::x86::DynamicAvx2Searcher;

        let needles = words.clone();
        define(
            c,
            "memmem/sliceslice/prebuilt/sliceslice-i386/words",
            &[],
            Box::new(move |b| {
                let searchers = needles
                    .iter()
                    .map(|needle| unsafe {
                        DynamicAvx2Searcher::new(needle.as_bytes())
                    })
                    .collect::<Vec<_>>();

                b.iter(|| {
                    for searcher in &searchers {
                        black_box(unsafe {
                            searcher.search_in(haystack.as_bytes())
                        });
                    }
                });
            }),
        );
    }
}
