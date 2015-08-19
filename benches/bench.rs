#![feature(test)]

extern crate memchr;
extern crate test;

use std::iter;

fn bench_data() -> Vec<u8> { iter::repeat(b'z').take(10000).collect() }

#[bench]
fn iterator(b: &mut test::Bencher) {
    let haystack = bench_data();
    let needle = b'a';
    b.iter(|| {
        assert!(haystack.iter().position(|&b| b == needle).is_none());
    });
    b.bytes = haystack.len() as u64;
}

#[bench]
fn libc_memchr(b: &mut test::Bencher) {
    let haystack = bench_data();
    let needle = b'a';
    b.iter(|| {
        assert!(memchr::memchr(needle, &haystack).is_none());
    });
    b.bytes = haystack.len() as u64;
}

#[bench]
fn iterator_reversed(b: &mut test::Bencher) {
    let haystack = bench_data();
    let needle = b'a';
    b.iter(|| {
        assert!(haystack.iter().rposition(|&b| b == needle).is_none());
    });
    b.bytes = haystack.len() as u64;
}

#[bench]
fn libc_memrchr(b: &mut test::Bencher) {
    let haystack = bench_data();
    let needle = b'a';
    b.iter(|| {
        assert!(memchr::memrchr(needle, &haystack).is_none());
    });
    b.bytes = haystack.len() as u64;
}
