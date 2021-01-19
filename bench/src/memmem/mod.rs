/*
This module defines benchmarks for the memmem family of functions.
Benchmarking a substring algorithm is particularly difficult, especially
when implementations (like this one, and others) use heuristics to speed up
common cases, typically at the expense of less common cases. The job of this
benchmark suite is to not only highlight the fast common cases, but to also put
a spotlight on the less common or pathological cases. While some things are
generally expected to be slower because of these heuristics, the benchmarks
help us make sure they we don't let things get too slow.

The naming scheme is as follows:

  memr?mem/{impl}/{config}/{corpus}/{needle}

Where {...} is a variable. Variables should never contain slashes. They are as
follows:

  impl
    A brief name describing the implementation under test. Possible values:

    krate
      The implementation provided by this crate.
    krate-nopre
      The implementation provided by this crate without prefilters enabled.
    bstr
      The implementation provided by the bstr crate.
      N.B. This is only applicable at time of writing, since bstr will
      eventually just use this crate.
    regex
      The implementation of substring search provided by the regex crate.
      N.B. This is only applicable at time of writing, since regex will
      eventually just use this crate.
    stud
      The implementation of substring search provided by the standard
      library. This implementation only works on valid UTF-8 by virtue of
      how its API is exposed.
    twoway
      The implementation of substring search provided by the twoway crate.
    sliceslice
      The implementation of substring search provided by the sliceslice crate.
    libc
      The implementation of memmem in your friendly neighborhood libc.

    Note that there is also a 'memmem' crate, but it is unmaintained and
    appears to just be a snapshot of std's implementation at a particular
    point in time (but exposed in a way to permit it to search arbitrary
    bytes).

  config
    This should be a brief description of the configuration of the search. Not
    all implementations can be benchmarked in all configurations. It depends on
    the API they expose. Possible values:

    oneshot
      Executes a single search without pre-building a searcher. That
      this measurement includes the time it takes to initialize a
      searcher.
    prebuilt
      Executes a single search without measuring the time it takes to
      build a searcher.
    iter-oneshot
      Counts the total number of matches. This measures the time it takes to
      build the searcher.
    iter-prebuilt
      Counts the total number of matches. This does not measure the time it
      takes to build the searcher.

  corpus
    A brief name describing the corpus or haystack used in the benchmark. In
    general, we vary this with regard to size and language. Possible values:

    subtitles-{en,ru,zh}
      Text from the OpenSubtitles project, in one of English, Russian or
      Chinese. This is the primary input meant to represent most kinds of
      haystacks.
    pathological-{...}
      A haystack that has been specifically constructed to exploit a
      pathological case in or more substring search implementations.
    sliceslice-words
      The haystack is varied across words in an English dictionary. Using
      this corpus means the benchmark is measuring performance on very small
      haystacks. This was taken from the sliceslice crate benchmarks.
    sliceslice-i386
      The haystack is an Intel 80386 reference manual.
      This was also taken from the sliceslice crate benchmarks.

  needle
    A brief name describing the needle used. Unlike other variables, there
    isn't a strong controlled vocabularly for this parameter. The needle
    variable is meant to be largely self explanatory. For example, a needle
    named "rare" probably means that the number of occurrences of the needle
    is expected to be particularly low.
*/

use criterion::Criterion;

use crate::{define, memmem::inputs::INPUTS};

mod imp;
mod inputs;
mod sliceslice;

pub fn all(c: &mut Criterion) {
    oneshot(c);
    prebuilt(c);
    oneshot_iter(c);
    prebuilt_iter(c);
    sliceslice::all(c);
}

fn oneshot(c: &mut Criterion) {
    macro_rules! def_impl {
        ($inp:expr, $q:expr, $freq:expr, $impl:ident) => {
            let config = "oneshot";
            let available = imp::$impl::available($q.needle);
            // We only define non-iter benchmarks when the count is <=1. Such
            // queries are usually constructed to only appear at the end.
            // Otherwise, for more common queries, the benchmark would be
            // approximately duplicative with benchmarks on shorter haystacks
            // for the implementations we benchmark.
            if $q.count <= 1 && available.contains(&config) {
                let expected = $q.count > 0;
                macro_rules! define {
                    ($dir:expr, $find:expr) => {
                        let name = format!(
                            "{dir}/{imp}/{config}/{inp}/{freq}-{q}",
                            dir = $dir,
                            imp = stringify!($impl),
                            config = config,
                            inp = $inp.name,
                            freq = $freq,
                            q = $q.name,
                        );
                        define(
                            c,
                            &name,
                            $inp.corpus.as_bytes(),
                            Box::new(move |b| {
                                b.iter(|| {
                                    assert_eq!(
                                        expected,
                                        $find($inp.corpus, $q.needle)
                                    );
                                });
                            }),
                        );
                    };
                }
                define!("memmem", imp::$impl::fwd::oneshot);
                if available.contains(&"reverse") {
                    define!("memrmem", imp::$impl::rev::oneshot);
                }
            }
        };
    }
    macro_rules! def_all_impls {
        ($inp:expr, $q:expr, $freq:expr) => {
            def_impl!($inp, $q, $freq, krate);
            def_impl!($inp, $q, $freq, krate_nopre);
            def_impl!($inp, $q, $freq, bstr);
            def_impl!($inp, $q, $freq, regex);
            def_impl!($inp, $q, $freq, stud);
            def_impl!($inp, $q, $freq, twoway);
            def_impl!($inp, $q, $freq, sliceslice);
            def_impl!($inp, $q, $freq, libc);
        };
    }
    for inp in INPUTS {
        for q in inp.never {
            def_all_impls!(inp, q, "never");
        }
        for q in inp.rare {
            def_all_impls!(inp, q, "rare");
        }
        for q in inp.common {
            def_all_impls!(inp, q, "common");
        }
    }
}

fn prebuilt(c: &mut Criterion) {
    macro_rules! def_impl {
        ($inp:expr, $q:expr, $freq:expr, $impl:ident) => {
            let config = "prebuilt";
            let available = imp::$impl::available($q.needle);
            // We only define non-iter benchmarks when the count is <=1. Such
            // queries are usually constructed to only appear at the end.
            // Otherwise, for more common queries, the benchmark would be
            // approximately duplicative with benchmarks on shorter haystacks
            // for the implementations we benchmark.
            if $q.count <= 1 && available.contains(&config) {
                let expected = $q.count > 0;
                macro_rules! define {
                    ($dir:expr, $new_finder:expr) => {
                        let name = format!(
                            "{dir}/{imp}/{config}/{inp}/{freq}-{q}",
                            dir = $dir,
                            imp = stringify!($impl),
                            config = config,
                            inp = $inp.name,
                            freq = $freq,
                            q = $q.name,
                        );
                        define(
                            c,
                            &name,
                            $inp.corpus.as_bytes(),
                            Box::new(move |b| {
                                let find = $new_finder($q.needle);
                                b.iter(|| {
                                    assert_eq!(expected, find($inp.corpus));
                                });
                            }),
                        );
                    };
                }
                define!("memmem", imp::$impl::fwd::prebuilt);
                if available.contains(&"reverse") {
                    define!("memrmem", imp::$impl::rev::prebuilt);
                }
            }
        };
    }
    macro_rules! def_all_impls {
        ($inp:expr, $q:expr, $freq:expr) => {
            def_impl!($inp, $q, $freq, krate);
            def_impl!($inp, $q, $freq, krate_nopre);
            def_impl!($inp, $q, $freq, bstr);
            def_impl!($inp, $q, $freq, regex);
            def_impl!($inp, $q, $freq, stud);
            def_impl!($inp, $q, $freq, twoway);
            def_impl!($inp, $q, $freq, sliceslice);
            def_impl!($inp, $q, $freq, libc);
        };
    }
    for inp in INPUTS {
        for q in inp.never {
            def_all_impls!(inp, q, "never");
        }
        for q in inp.rare {
            def_all_impls!(inp, q, "rare");
        }
        for q in inp.common {
            def_all_impls!(inp, q, "common");
        }
    }
}

fn oneshot_iter(c: &mut Criterion) {
    macro_rules! def_impl {
        ($inp:expr, $q:expr, $freq:expr, $impl:ident) => {
            let config = "oneshotiter";
            let available = imp::$impl::available($q.needle);
            // We only define iter benchmarks when the count is >1. Since
            // queries with count<=1 are usually constructed such that the
            // match appears at the end of the haystack, it doesn't make much
            // sense to also benchmark iteration for that case. Instead, we only
            // benchmark iteration for queries that match more frequently.
            if $q.count > 1 && available.contains(&config) {
                macro_rules! define {
                    ($dir:expr, $find_iter:expr) => {
                        let name = format!(
                            "{dir}/{imp}/{config}/{inp}/{freq}-{q}",
                            dir = $dir,
                            imp = stringify!($impl),
                            config = config,
                            inp = $inp.name,
                            freq = $freq,
                            q = $q.name,
                        );
                        define(
                            c,
                            &name,
                            $inp.corpus.as_bytes(),
                            Box::new(move |b| {
                                b.iter(|| {
                                    let it =
                                        $find_iter($inp.corpus, $q.needle);
                                    assert_eq!($q.count, it.count());
                                });
                            }),
                        );
                    };
                }
                define!("memmem", imp::$impl::fwd::oneshotiter);
                if available.contains(&"reverse") {
                    define!("memrmem", imp::$impl::rev::oneshotiter);
                }
            }
        };
    }
    macro_rules! def_all_impls {
        ($inp:expr, $q:expr, $freq:expr) => {
            def_impl!($inp, $q, $freq, krate);
            def_impl!($inp, $q, $freq, krate_nopre);
            def_impl!($inp, $q, $freq, bstr);
            def_impl!($inp, $q, $freq, regex);
            def_impl!($inp, $q, $freq, stud);
            def_impl!($inp, $q, $freq, twoway);
            def_impl!($inp, $q, $freq, sliceslice);
            def_impl!($inp, $q, $freq, libc);
        };
    }
    for inp in INPUTS {
        for q in inp.never {
            def_all_impls!(inp, q, "never");
        }
        for q in inp.rare {
            def_all_impls!(inp, q, "rare");
        }
        for q in inp.common {
            def_all_impls!(inp, q, "common");
        }
    }
}

fn prebuilt_iter(c: &mut Criterion) {
    macro_rules! def_impl {
        ($inp:expr, $q:expr, $freq:expr, $impl:ident) => {
            let config = "prebuiltiter";
            let available = imp::$impl::available($q.needle);
            // We only define iter benchmarks when the count is >1. Since
            // queries with count<=1 are usually constructed such that the
            // match appears at the end of the haystack, it doesn't make much
            // sense to also benchmark iteration for that case. Instead, we only
            // benchmark iteration for queries that match more frequently.
            if $q.count > 1 && available.contains(&config) {
                macro_rules! define {
                    ($dir:expr, $new_finder:expr) => {
                        let name = format!(
                            "{dir}/{imp}/{config}/{inp}/{freq}-{q}",
                            dir = $dir,
                            imp = stringify!($impl),
                            config = config,
                            inp = $inp.name,
                            freq = $freq,
                            q = $q.name,
                        );
                        define(
                            c,
                            &name,
                            $inp.corpus.as_bytes(),
                            Box::new(move |b| {
                                let finder = $new_finder($q.needle);
                                b.iter(|| {
                                    let it = finder.iter($inp.corpus);
                                    assert_eq!($q.count, it.count());
                                });
                            }),
                        );
                    };
                }
                define!("memmem", imp::$impl::fwd::prebuiltiter);
                if available.contains(&"reverse") {
                    define!("memrmem", imp::$impl::rev::prebuiltiter);
                }
            }
        };
    }
    macro_rules! def_all_impls {
        ($inp:expr, $q:expr, $freq:expr) => {
            def_impl!($inp, $q, $freq, krate);
            def_impl!($inp, $q, $freq, krate_nopre);
            def_impl!($inp, $q, $freq, bstr);
            def_impl!($inp, $q, $freq, regex);
            def_impl!($inp, $q, $freq, stud);
            def_impl!($inp, $q, $freq, twoway);
            def_impl!($inp, $q, $freq, sliceslice);
            def_impl!($inp, $q, $freq, libc);
        };
    }
    for inp in INPUTS {
        for q in inp.never {
            def_all_impls!(inp, q, "never");
        }
        for q in inp.rare {
            def_all_impls!(inp, q, "rare");
        }
        for q in inp.common {
            def_all_impls!(inp, q, "common");
        }
    }
}
