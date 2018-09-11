use std::prelude::v1::*;

use fallback;
use naive;
use {memchr, memrchr, memchr2, memchr3};

use tests::memchr_tests;

#[test]
fn memchr1_find() {
    for test in memchr_tests() {
        test.one(false, memchr);
    }
}

#[test]
fn memchr1_fallback_find() {
    for test in memchr_tests() {
        test.one(false, fallback::memchr);
    }
}

#[test]
fn memchr2_find() {
    for test in memchr_tests() {
        test.two(false, memchr2);
    }
}

#[test]
fn memchr2_fallback_find() {
    for test in memchr_tests() {
        test.two(false, fallback::memchr2);
    }
}

#[test]
fn memchr3_find() {
    for test in memchr_tests() {
        test.three(false, memchr3);
    }
}

#[test]
fn memchr3_fallback_find() {
    for test in memchr_tests() {
        test.three(false, fallback::memchr3);
    }
}

#[test]
fn memrchr1_find() {
    for test in memchr_tests() {
        test.one(true, memrchr);
    }
}

#[test]
fn memrchr1_fallback_find() {
    for test in memchr_tests() {
        test.one(true, fallback::memrchr);
    }
}

quickcheck! {
    fn qc_memchr1_never_fail(n1: u8, corpus: Vec<u8>) -> bool {
        memchr(n1, &corpus);
        true
    }
}

quickcheck! {
    fn qc_memchr2_never_fail(n1: u8, n2: u8, corpus: Vec<u8>) -> bool {
        memchr2(n1, n2, &corpus);
        true
    }
}

quickcheck! {
    fn qc_memchr3_never_fail(
        n1: u8, n2: u8, n3: u8,
        corpus: Vec<u8>
    ) -> bool {
        memchr3(n1, n2, n3, &corpus);
        true
    }
}

quickcheck! {
    fn qc_memrchr1_never_fail(n1: u8, corpus: Vec<u8>) -> bool {
        memrchr(n1, &corpus);
        true
    }
}

quickcheck! {
    fn qc_memchr1_matches_naive(n1: u8, corpus: Vec<u8>) -> bool {
        memchr(n1, &corpus) == naive::memchr(n1, &corpus)
    }
}

quickcheck! {
    fn qc_memchr2_matches_naive(n1: u8, n2: u8, corpus: Vec<u8>) -> bool {
        memchr2(n1, n2, &corpus) == naive::memchr2(n1, n2, &corpus)
    }
}

quickcheck! {
    fn qc_memchr3_matches_naive(
        n1: u8, n2: u8, n3: u8,
        corpus: Vec<u8>
    ) -> bool {
        memchr3(n1, n2, n3, &corpus) == naive::memchr3(n1, n2, n3, &corpus)
    }
}

quickcheck! {
    fn qc_memrchr1_matches_naive(n1: u8, corpus: Vec<u8>) -> bool {
        memrchr(n1, &corpus) == naive::memrchr(n1, &corpus)
    }
}
