use criterion::Criterion;
use memchr::memmem::HeuristicFrequencyRank;

use crate::define;

pub(crate) fn all(c: &mut Criterion) {
    finder_construction(c);
    byte_frequencies(c);
}

fn finder_construction(c: &mut Criterion) {
    // This benchmark is purely for measuring the time taken to create a
    // `Finder`. It is here to prevent regressions when adding new features
    // to the `Finder`, such as the ability to construct with a custom
    // `HeuristicFrequencyRank`.
    const NEEDLES: [&str; 3] = ["a", "abcd", "abcdefgh12345678"];

    for needle in NEEDLES {
        define(
            c,
            &format!(
                "memmem/krate/bytefreq/construct-finder/default(len={})",
                needle.len()
            ),
            needle.as_bytes(),
            Box::new(move |b| {
                b.iter(|| {
                    memchr::memmem::FinderBuilder::new()
                        .build_forward(needle.as_bytes())
                });
            }),
        );
        define(
            c,
            &format!(
                "memmem/krate/bytefreq/construct-finder/custom(len={})",
                needle.len()
            ),
            needle.as_bytes(),
            Box::new(move |b| {
                b.iter(|| {
                    memchr::memmem::FinderBuilder::new()
                        .build_forward_with_ranker(Binary, needle.as_bytes())
                });
            }),
        );
    }
}

fn byte_frequencies(c: &mut Criterion) {
    // This benchmark exists to demonstrate a common use case for
    // customizing the byte frequency table used by a `Finder`
    // and the relative performance gain from using an optimal table.
    // This is essentially why `HeuristicFrequencyRank` was added.

    // Bytes we want to scan for that are rare in strings but common in
    // executables.
    const NEEDLE: &[u8] = b"\x00\x00\xdd\xdd'";

    // The input for the benchmark is the benchmark binary itself
    let exe = std::env::args().next().unwrap();
    let corpus = std::fs::read(exe).unwrap();

    let bin = corpus.clone();
    define(
        c,
        &format!("memmem/krate/bytefreq/default"),
        &corpus,
        Box::new(move |b| {
            let finder =
                memchr::memmem::FinderBuilder::new().build_forward(NEEDLE);
            b.iter(|| {
                assert_eq!(1, finder.find_iter(&bin).count());
            });
        }),
    );

    let bin = corpus.clone();
    define(
        c,
        &format!("memmem/krate/bytefreq/custom"),
        &corpus,
        Box::new(move |b| {
            let finder = memchr::memmem::FinderBuilder::new()
                .build_forward_with_ranker(Binary, NEEDLE);
            b.iter(|| {
                assert_eq!(1, finder.find_iter(&bin).count());
            });
        }),
    );
}

/// A byte-frequency table that is good for scanning binary executables.
struct Binary;

impl HeuristicFrequencyRank for Binary {
    fn rank(&self, byte: u8) -> u8 {
        const TABLE: [u8; 256] = [
            255, 128, 61, 43, 50, 41, 27, 28, 57, 15, 21, 13, 24, 17, 17, 89,
            58, 16, 11, 7, 14, 23, 7, 6, 24, 9, 6, 5, 9, 4, 7, 16, 68, 11, 9,
            6, 88, 7, 4, 4, 23, 9, 4, 8, 8, 5, 10, 4, 30, 11, 9, 24, 11, 5, 5,
            5, 19, 11, 6, 17, 9, 9, 6, 8, 48, 58, 11, 14, 53, 40, 9, 9, 254,
            35, 3, 6, 52, 23, 6, 6, 27, 4, 7, 11, 14, 13, 10, 11, 11, 5, 2,
            10, 16, 12, 6, 19, 19, 20, 5, 14, 16, 31, 19, 7, 14, 20, 4, 4, 19,
            8, 18, 20, 24, 1, 25, 19, 58, 29, 10, 5, 15, 20, 2, 2, 9, 4, 3, 5,
            51, 11, 4, 53, 23, 39, 6, 4, 13, 81, 4, 186, 5, 67, 3, 2, 15, 0,
            0, 1, 3, 2, 0, 0, 5, 0, 0, 0, 2, 0, 0, 0, 12, 2, 1, 1, 3, 1, 1, 1,
            6, 1, 2, 1, 3, 1, 1, 2, 9, 1, 1, 0, 2, 2, 4, 4, 11, 6, 7, 3, 6, 9,
            4, 5, 46, 18, 8, 18, 17, 3, 8, 20, 16, 10, 3, 7, 175, 4, 6, 7, 13,
            3, 7, 3, 3, 1, 3, 3, 10, 3, 1, 5, 2, 0, 1, 2, 16, 3, 5, 1, 6, 1,
            1, 2, 58, 20, 3, 14, 12, 2, 1, 3, 16, 3, 5, 8, 3, 1, 8, 6, 17, 6,
            5, 3, 8, 6, 13, 175,
        ];
        TABLE[byte as usize]
    }
}
