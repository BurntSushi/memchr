use std::prelude::v1::*;

mod iter;
mod memchr;

/// A set of tests for memchr-like functions.
///
/// These tests mostly try to cover the short string cases. We cover the longer
/// string cases via the benchmarks (which are tests themselves) and via
/// quickcheck tests.
const MEMCHR_TESTS: &[MemchrTest] = &[
    // one needle (applied to memchr + memchr2 + memchr3)
    MemchrTest {
        corpus: "a",
        needles: &[b'a'],
        positions: &[0],
    },
    MemchrTest {
        corpus: "aa",
        needles: &[b'a'],
        positions: &[0, 1],
    },
    MemchrTest {
        corpus: "aaa",
        needles: &[b'a'],
        positions: &[0, 1, 2],
    },
    MemchrTest {
        corpus: "",
        needles: &[b'a'],
        positions: &[],
    },
    MemchrTest {
        corpus: "z",
        needles: &[b'a'],
        positions: &[],
    },
    MemchrTest {
        corpus: "zz",
        needles: &[b'a'],
        positions: &[],
    },
    MemchrTest {
        corpus: "zza",
        needles: &[b'a'],
        positions: &[2],
    },
    MemchrTest {
        corpus: "zaza",
        needles: &[b'a'],
        positions: &[1, 3],
    },
    MemchrTest {
        corpus: "zzza",
        needles: &[b'a'],
        positions: &[3],
    },
    MemchrTest {
        corpus: "\x00a",
        needles: &[b'a'],
        positions: &[1],
    },
    MemchrTest {
        corpus: "\x00",
        needles: &[b'\x00'],
        positions: &[0],
    },
    MemchrTest {
        corpus: "\x00\x00",
        needles: &[b'\x00'],
        positions: &[0, 1],
    },
    MemchrTest {
        corpus: "\x00a\x00",
        needles: &[b'\x00'],
        positions: &[0, 2],
    },

    // two needles (applied to memchr2 + memchr3)
    MemchrTest {
        corpus: "az",
        needles: &[b'a', b'z'],
        positions: &[0, 1],
    },
    MemchrTest {
        corpus: "az",
        needles: &[b'a', b'z'],
        positions: &[0, 1],
    },
    MemchrTest {
        corpus: "az",
        needles: &[b'x', b'y'],
        positions: &[],
    },
    MemchrTest {
        corpus: "az",
        needles: &[b'a', b'y'],
        positions: &[0],
    },
    MemchrTest {
        corpus: "az",
        needles: &[b'x', b'z'],
        positions: &[1],
    },
    MemchrTest {
        corpus: "yyyyaz",
        needles: &[b'a', b'z'],
        positions: &[4, 5],
    },
    MemchrTest {
        corpus: "yyyyaz",
        needles: &[b'z', b'a'],
        positions: &[4, 5],
    },

    // three needles (applied to memchr3)
    MemchrTest {
        corpus: "xyz",
        needles: &[b'x', b'y', b'z'],
        positions: &[0, 1, 2],
    },
    MemchrTest {
        corpus: "zxy",
        needles: &[b'x', b'y', b'z'],
        positions: &[0, 1, 2],
    },
    MemchrTest {
        corpus: "zxy",
        needles: &[b'x', b'a', b'z'],
        positions: &[0, 1],
    },
    MemchrTest {
        corpus: "zxy",
        needles: &[b't', b'a', b'z'],
        positions: &[0],
    },
    MemchrTest {
        corpus: "yxz",
        needles: &[b't', b'a', b'z'],
        positions: &[2],
    },
];

/// A description of a test on a memchr like function.
#[derive(Clone, Debug)]
struct MemchrTest {
    /// The thing to search. We use `&str` instead of `&[u8]` because they
    /// are nicer to write in tests, and we don't miss much since memchr
    /// doesn't care about UTF-8.
    corpus: &'static str,
    /// The needles to search for. This is intended to be an "alternation" of
    /// needles. The number of needles may cause this test to be skipped for
    /// some memchr variants. For example, a test with 2 needles cannot be used
    /// to test `memchr`, but can be used to test `memchr2` and `memchr3`.
    /// However, a test with only 1 needle can be used to test all of `memchr`,
    /// `memchr2` and `memchr3`. We achieve this by filling in the needles with
    /// bytes that we never used in the corpus (such as \xFF, which is invalid
    /// UTF-8).
    needles: &'static [u8],
    /// The positions expected to match for all of the needles.
    positions: &'static [usize],
}

impl MemchrTest {
    fn one<F: FnOnce(u8, &[u8]) -> Option<usize>>(
        &self,
        reverse: bool,
        f: F,
    ) {
        let needles = match self.needles(1) {
            None => return,
            Some(needles) => needles,
        };
        assert_eq!(
            self.positions(reverse).get(0).cloned(),
            f(needles[0], self.corpus.as_bytes()),
            r"search for {:?} failed in: {:?}",
            needles[0] as char,
            self.corpus
        );
    }

    fn two<F: FnOnce(u8, u8, &[u8]) -> Option<usize>>(
        &self,
        reverse: bool,
        f: F,
    ) {
        let needles = match self.needles(2) {
            None => return,
            Some(needles) => needles,
        };
        assert_eq!(
            self.positions(reverse).get(0).cloned(),
            f(needles[0], needles[1], self.corpus.as_bytes()),
            r"search for {:?}|{:?} failed in: {:?}",
            needles[0] as char,
            needles[1] as char,
            self.corpus
        );
    }

    fn three<F: FnOnce(u8, u8, u8, &[u8]) -> Option<usize>>(
        &self,
        reverse: bool,
        f: F,
    ) {
        let needles = match self.needles(3) {
            None => return,
            Some(needles) => needles,
        };
        assert_eq!(
            self.positions(reverse).get(0).cloned(),
            f(needles[0], needles[1], needles[2], self.corpus.as_bytes()),
            r"search for {:?}|{:?}|{:?} failed in: {:?}",
            needles[0] as char,
            needles[1] as char,
            needles[2] as char,
            self.corpus
        );
    }

    fn iter_one<I, F>(&self, reverse: bool, f: F)
    where F: FnOnce(u8, &'static [u8]) -> I,
          I: Iterator<Item=usize>
    {
        if let Some(ns) = self.needles(1) {
            self.iter(reverse, f(ns[0], self.corpus.as_bytes()));
        }
    }

    fn iter_two<I, F>(&self, reverse: bool, f: F)
    where F: FnOnce(u8, u8, &'static [u8]) -> I,
          I: Iterator<Item=usize>
    {
        if let Some(ns) = self.needles(2) {
            self.iter(reverse, f(ns[0], ns[1], self.corpus.as_bytes()));
        }
    }

    fn iter_three<I, F>(&self, reverse: bool, f: F)
    where F: FnOnce(u8, u8, u8, &'static [u8]) -> I,
          I: Iterator<Item=usize>
    {
        if let Some(ns) = self.needles(3) {
            self.iter(reverse, f(ns[0], ns[1], ns[2], self.corpus.as_bytes()));
        }
    }

    /// Test that the positions yielded by the given iterator match the
    /// positions in this test. If reverse is true, then reverse the positions
    /// before comparing them.
    fn iter<I: Iterator<Item=usize>>(&self, reverse: bool, it: I) {
        assert_eq!(
            self.positions(reverse),
            it.collect::<Vec<usize>>(),
            r"search for {:?} failed in: {:?}",
            self.needles.iter().map(|&b| b as char).collect::<Vec<char>>(),
            self.corpus
        );
    }

    /// Return exactly `count` needles from this test. If this test has less
    /// than `count` needles, then add `\xFF` until the number of needles
    /// matches `count`. If this test has more than `count` needles, then
    /// return `None` (because there is no way to use this test data for a
    /// search using fewer needles).
    fn needles(&self, count: usize) -> Option<Vec<u8>> {
        if self.needles.len() > count {
            return None;
        }

        let mut needles = self.needles.to_vec();
        for _ in needles.len()..count {
            // we assume \xFF is never used in tests.
            needles.push(b'\xFF');
        }
        Some(needles)
    }

    /// Return the positions in this test, reversed if `reverse` is true.
    fn positions(&self, reverse: bool) -> Vec<usize> {
        if reverse {
            let mut positions = self.positions.to_vec();
            positions.reverse();
            positions
        } else {
            self.positions.to_vec()
        }
    }
}
