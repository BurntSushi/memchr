/*!
This crate defines two functions, `memchr` and `memrchr`, which expose a safe interface
to the corresponding functions in `libc`.
*/

#![deny(missing_docs)]
#![allow(unused_imports)]

#![cfg_attr(not(feature = "use_std"), no_std)]

#[cfg(all(test, not(feature = "use_std")))]
#[macro_use]
extern crate std;

extern crate libc;

use libc::c_void;
use libc::{c_int, size_t};

#[cfg(feature = "use_std")]
use std::cmp;
#[cfg(not(feature = "use_std"))]
use core::cmp;

const LO_U64: u64 = 0x0101010101010101;
const HI_U64: u64 = 0x8080808080808080;

// use truncation
const LO_USIZE: usize = LO_U64 as usize;
const HI_USIZE: usize = HI_U64 as usize;

#[cfg(target_pointer_width = "32")]
const USIZE_BYTES: usize = 4;
#[cfg(target_pointer_width = "64")]
const USIZE_BYTES: usize = 8;

/// Return `true` if `x` contains any zero byte.
///
/// From *Matters Computational*, J. Arndt
///
/// "The idea is to subtract one from each of the bytes and then look for
/// bytes where the borrow propagated all the way to the most significant
/// bit."
#[inline]
fn contains_zero_byte(x: usize) -> bool {
    x.wrapping_sub(LO_USIZE) & !x & HI_USIZE != 0
}

#[cfg(target_pointer_width = "32")]
#[inline]
fn repeat_byte(b: u8) -> usize {
    let mut rep = (b as usize) << 8 | b as usize;
    rep = rep << 16 | rep;
    rep
}

#[cfg(target_pointer_width = "64")]
#[inline]
fn repeat_byte(b: u8) -> usize {
    let mut rep = (b as usize) << 8 | b as usize;
    rep = rep << 16 | rep;
    rep = rep << 32 | rep;
    rep
}

/// An iterator for memchr
pub struct Memchr<'a> {
    needle: u8,
    // The haystack to iterate over
    haystack: &'a [u8],
    // The index
    position: usize,
    rposition: usize,
}

impl<'a> Memchr<'a> {
    /// Creates a new iterator that yields all positions of needle in haystack.
    pub fn new(needle: u8, haystack: &[u8]) -> Memchr {
        Memchr {
            needle: needle,
            haystack: haystack,
            position: 0,
            rposition: 0,
        }
    }
}

impl<'a> Iterator for Memchr<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        let search_result = memchr(self.needle, &self.haystack);
        match search_result {
            Some(index) => {
                // Move our internal position
                self.haystack = self.haystack.split_at(index + 1).1;
                self.position = self.position + index + 1;
                Some(self.position)
            }
            None => None,
        }
    }
}

impl<'a> DoubleEndedIterator for Memchr<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let search_result = memrchr(self.needle, &self.haystack);
        match search_result {
            Some(index) => {
                // Move our internal position
                self.haystack = self.haystack.split_at(index).0;
                if self.rposition > 0{
                    //memrchr has been used once already.
                    self.rposition = self.rposition - index;
                    return Some(self.rposition);
                }else{
                    return Some(self.position + index+1);
                }
            }
            None => None,
        }
    }
}

/// A safe interface to `memchr`.
///
/// Returns the index corresponding to the first occurrence of `needle` in
/// `haystack`, or `None` if one is not found.
///
/// memchr reduces to super-optimized machine code at around an order of
/// magnitude faster than `haystack.iter().position(|&b| b == needle)`.
/// (See benchmarks.)
///
/// # Example
///
/// This shows how to find the first position of a byte in a byte string.
///
/// ```rust
/// use memchr::memchr;
///
/// let haystack = b"the quick brown fox";
/// assert_eq!(memchr(b'k', haystack), Some(8));
/// ```
#[inline(always)] // reduces constant overhead
pub fn memchr(needle: u8, haystack: &[u8]) -> Option<usize> {
    // libc memchr
    #[cfg(any(not(target_os = "windows"),
              not(any(target_pointer_width = "32",
                      target_pointer_width = "64"))))]
    #[inline(always)] // reduces constant overhead
    fn memchr_specific(needle: u8, haystack: &[u8]) -> Option<usize> {
        use libc::memchr as libc_memchr;

        let p = unsafe {
            libc_memchr(haystack.as_ptr() as *const c_void,
                        needle as c_int,
                        haystack.len() as size_t)
        };
        if p.is_null() {
            None
        } else {
            Some(p as usize - (haystack.as_ptr() as usize))
        }
    }

    // use fallback on windows, since it's faster
    #[cfg(all(target_os = "windows",
              any(target_pointer_width = "32",
                  target_pointer_width = "64")))]
    fn memchr_specific(needle: u8, haystack: &[u8]) -> Option<usize> {
        fallback::memchr(needle, haystack)
    }

    memchr_specific(needle, haystack)
}

/// A safe interface to `memrchr`.
///
/// Returns the index corresponding to the last occurrence of `needle` in
/// `haystack`, or `None` if one is not found.
///
/// # Example
///
/// This shows how to find the last position of a byte in a byte string.
///
/// ```rust
/// use memchr::memrchr;
///
/// let haystack = b"the quick brown fox";
/// assert_eq!(memrchr(b'o', haystack), Some(17));
/// ```
#[inline(always)] // reduces constant overhead
pub fn memrchr(needle: u8, haystack: &[u8]) -> Option<usize> {

    #[cfg(target_os = "linux")]
    #[inline(always)] // reduces constant overhead
    fn memrchr_specific(needle: u8, haystack: &[u8]) -> Option<usize> {
        // GNU's memrchr() will - unlike memchr() - error if haystack is empty.
        if haystack.is_empty() {
            return None;
        }
        let p = unsafe {
            libc::memrchr(haystack.as_ptr() as *const c_void,
                          needle as c_int,
                          haystack.len() as size_t)
        };
        if p.is_null() {
            None
        } else {
            Some(p as usize - (haystack.as_ptr() as usize))
        }
    }

    #[cfg(all(not(target_os = "linux"),
              any(target_pointer_width = "32", target_pointer_width = "64")))]
    fn memrchr_specific(needle: u8, haystack: &[u8]) -> Option<usize> {
        fallback::memrchr(needle, haystack)
    }

    // For the rare case of neither 32 bit nor 64-bit platform.
    #[cfg(all(not(target_os = "linux"),
              not(target_pointer_width = "32"),
              not(target_pointer_width = "64")))]
    fn memrchr_specific(needle: u8, haystack: &[u8]) -> Option<usize> {
        haystack.iter().rposition(|&b| b == needle)
    }

    memrchr_specific(needle, haystack)
}

/// An iterator for Memchr2
pub struct Memchr2<'a> {
    needle1: u8,
    needle2: u8,
    // The haystack to iterate over
    haystack: &'a [u8],
    // The index
    position: usize,
}

impl<'a> Memchr2<'a> {
    /// Creates a new iterator that yields all positions of needle in haystack.
    pub fn new(needle1: u8, needle2: u8, haystack: &[u8]) -> Memchr2 {
        Memchr2 {
            needle1: needle1,
            needle2: needle2,
            haystack: haystack,
            position: 0,
        }
    }
}

impl<'a> Iterator for Memchr2<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        let search_result = memchr2(self.needle1, self.needle2, &self.haystack);
        match search_result {
            Some(index) => {
                // Move our internal position
                self.haystack = self.haystack.split_at(index + 1).1;
                self.position = self.position + index + 1;
                Some(self.position)
            }
            None => None,
        }
    }
}


/// Like `memchr`, but searches for two bytes instead of one.
pub fn memchr2(needle1: u8, needle2: u8, haystack: &[u8]) -> Option<usize> {
    fn slow(b1: u8, b2: u8, haystack: &[u8]) -> Option<usize> {
        haystack.iter().position(|&b| b == b1 || b == b2)
    }

    let len = haystack.len();
    let ptr = haystack.as_ptr();
    let align = (ptr as usize) & (USIZE_BYTES - 1);
    let mut i = 0;
    if align > 0 {
        i = cmp::min(USIZE_BYTES - align, len);
        if let Some(found) = slow(needle1, needle2, &haystack[..i]) {
            return Some(found);
        }
    }
    let repeated_b1 = repeat_byte(needle1);
    let repeated_b2 = repeat_byte(needle2);
    if len >= USIZE_BYTES {
        while i <= len - USIZE_BYTES {
            unsafe {
                let u = *(ptr.offset(i as isize) as *const usize);
                let found_ub1 = contains_zero_byte(u ^ repeated_b1);
                let found_ub2 = contains_zero_byte(u ^ repeated_b2);
                if found_ub1 || found_ub2 {
                    break;
                }
            }
            i += USIZE_BYTES;
        }
    }
    slow(needle1, needle2, &haystack[i..]).map(|pos| i + pos)
}

/// An iterator for Memchr3
pub struct Memchr3<'a> {
    needle1: u8,
    needle2: u8,
    needle3: u8,
    // The haystack to iterate over
    haystack: &'a [u8],
    // The index
    position: usize,
}

impl<'a> Memchr3<'a> {
    /// Create a new Memchr2 that's initalized to zero with a haystack
    pub fn new(needle1: u8, needle2: u8, needle3: u8, haystack: &[u8]) -> Memchr3 {
        Memchr3 {
            needle1: needle1,
            needle2: needle2,
            needle3: needle3,
            haystack: haystack,
            position: 0,
        }
    }
}

impl<'a> Iterator for Memchr3<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        let search_result = memchr3(self.needle1, self.needle2, self.needle3, &self.haystack);
        match search_result {
            Some(index) => {
                // Move our internal position
                self.haystack = self.haystack.split_at(index + 1).1;
                self.position = self.position + index + 1;
                Some(self.position)
            }
            None => None,
        }
    }
}

/// Like `memchr`, but searches for three bytes instead of one.
pub fn memchr3(needle1: u8, needle2: u8, needle3: u8, haystack: &[u8]) -> Option<usize> {
    fn slow(b1: u8, b2: u8, b3: u8, haystack: &[u8]) -> Option<usize> {
        haystack.iter().position(|&b| b == b1 || b == b2 || b == b3)
    }

    let len = haystack.len();
    let ptr = haystack.as_ptr();
    let align = (ptr as usize) & (USIZE_BYTES - 1);
    let mut i = 0;
    if align > 0 {
        i = cmp::min(USIZE_BYTES - align, len);
        if let Some(found) = slow(needle1, needle2, needle3, &haystack[..i]) {
            return Some(found);
        }
    }
    let repeated_b1 = repeat_byte(needle1);
    let repeated_b2 = repeat_byte(needle2);
    let repeated_b3 = repeat_byte(needle3);
    if len >= USIZE_BYTES {
        while i <= len - USIZE_BYTES {
            unsafe {
                let u = *(ptr.offset(i as isize) as *const usize);
                let found_ub1 = contains_zero_byte(u ^ repeated_b1);
                let found_ub2 = contains_zero_byte(u ^ repeated_b2);
                let found_ub3 = contains_zero_byte(u ^ repeated_b3);
                if found_ub1 || found_ub2 || found_ub3 {
                    break;
                }
            }
            i += USIZE_BYTES;
        }
    }
    slow(needle1, needle2, needle3, &haystack[i..]).map(|pos| i + pos)
}

#[allow(dead_code)]
#[cfg(any(test, all(not(target_os = "linux"),
          any(target_pointer_width = "32", target_pointer_width = "64"))))]
mod fallback {
    #[cfg(feature = "use_std")]
    use std::cmp;
    #[cfg(not(feature = "use_std"))]
    use core::cmp;

    use super::{LO_U64, HI_U64, LO_USIZE, HI_USIZE, USIZE_BYTES, contains_zero_byte, repeat_byte};

    /// Return the first index matching the byte `a` in `text`.
    pub fn memchr(x: u8, text: &[u8]) -> Option<usize> {
        // Scan for a single byte value by reading two `usize` words at a time.
        //
        // Split `text` in three parts
        // - unaligned inital part, before the first word aligned address in text
        // - body, scan by 2 words at a time
        // - the last remaining part, < 2 word size
        let len = text.len();
        let ptr = text.as_ptr();

        // search up to an aligned boundary
        let align = (ptr as usize) & (USIZE_BYTES - 1);
        let mut offset;
        if align > 0 {
            offset = cmp::min(USIZE_BYTES - align, len);
            if let Some(index) = text[..offset].iter().position(|elt| *elt == x) {
                return Some(index);
            }
        } else {
            offset = 0;
        }

        // search the body of the text
        let repeated_x = repeat_byte(x);

        if len >= 2 * USIZE_BYTES {
            while offset <= len - 2 * USIZE_BYTES {
                debug_assert_eq!((ptr as usize + offset) % USIZE_BYTES, 0);
                unsafe {
                    let u = *(ptr.offset(offset as isize) as *const usize);
                    let v = *(ptr.offset((offset + USIZE_BYTES) as isize) as *const usize);

                    // break if there is a matching byte
                    let zu = contains_zero_byte(u ^ repeated_x);
                    let zv = contains_zero_byte(v ^ repeated_x);
                    if zu || zv {
                        break;
                    }
                }
                offset += USIZE_BYTES * 2;
            }
        }

        // find the byte after the point the body loop stopped
        text[offset..].iter().position(|elt| *elt == x).map(|i| offset + i)
    }

    /// Return the last index matching the byte `a` in `text`.
    pub fn memrchr(x: u8, text: &[u8]) -> Option<usize> {
        // Scan for a single byte value by reading two `usize` words at a time.
        //
        // Split `text` in three parts
        // - unaligned tail, after the last word aligned address in text
        // - body, scan by 2 words at a time
        // - the first remaining bytes, < 2 word size
        let len = text.len();
        let ptr = text.as_ptr();

        // search to an aligned boundary
        let end_align = (ptr as usize + len) & (USIZE_BYTES - 1);
        let mut offset;
        if end_align > 0 {
            offset = if end_align >= len { 0 } else { len - end_align };
            if let Some(index) = text[offset..].iter().rposition(|elt| *elt == x) {
                return Some(offset + index);
            }
        } else {
            offset = len;
        }

        // search the body of the text
        let repeated_x = repeat_byte(x);

        while offset >= 2 * USIZE_BYTES {
            debug_assert_eq!((ptr as usize + offset) % USIZE_BYTES, 0);
            unsafe {
                let u = *(ptr.offset(offset as isize - 2 * USIZE_BYTES as isize) as *const usize);
                let v = *(ptr.offset(offset as isize - USIZE_BYTES as isize) as *const usize);

                // break if there is a matching byte
                let zu = contains_zero_byte(u ^ repeated_x);
                let zv = contains_zero_byte(v ^ repeated_x);
                if zu || zv {
                    break;
                }
            }
            offset -= 2 * USIZE_BYTES;
        }

        // find the byte before the point the body loop stopped
        text[..offset].iter().rposition(|elt| *elt == x)
    }
}

#[cfg(test)]
mod tests {
    extern crate quickcheck;
    use std::prelude::v1::*;

    use super::{memchr, memrchr, memchr2, memchr3, Memchr, Memchr2, Memchr3};
    // Use a macro to test both native and fallback impls on all configurations
    macro_rules! memchr_tests {
        ($mod_name:ident, $memchr:path, $memrchr:path) => {
            mod $mod_name {
            use std::prelude::v1::*;
            use super::quickcheck;
            #[test]
            fn matches_one() {
                assert_eq!(Some(0), $memchr(b'a', b"a"));
            }

            #[test]
            fn matches_begin() {
                assert_eq!(Some(0), $memchr(b'a', b"aaaa"));
            }

            #[test]
            fn matches_end() {
                assert_eq!(Some(4), $memchr(b'z', b"aaaaz"));
            }

            #[test]
            fn matches_nul() {
                assert_eq!(Some(4), $memchr(b'\x00', b"aaaa\x00"));
            }

            #[test]
            fn matches_past_nul() {
                assert_eq!(Some(5), $memchr(b'z', b"aaaa\x00z"));
            }

            #[test]
            fn no_match_empty() {
                assert_eq!(None, $memchr(b'a', b""));
            }

            #[test]
            fn no_match() {
                assert_eq!(None, $memchr(b'a', b"xyz"));
            }

            #[test]
            fn qc_never_fail() {
                fn prop(needle: u8, haystack: Vec<u8>) -> bool {
                    $memchr(needle, &haystack); true
                }
                quickcheck::quickcheck(prop as fn(u8, Vec<u8>) -> bool);
            }

            #[test]
            fn matches_one_reversed() {
                assert_eq!(Some(0), $memrchr(b'a', b"a"));
            }

            #[test]
            fn matches_begin_reversed() {
                assert_eq!(Some(3), $memrchr(b'a', b"aaaa"));
            }

            #[test]
            fn matches_end_reversed() {
                assert_eq!(Some(0), $memrchr(b'z', b"zaaaa"));
            }

            #[test]
            fn matches_nul_reversed() {
                assert_eq!(Some(4), $memrchr(b'\x00', b"aaaa\x00"));
            }

            #[test]
            fn matches_past_nul_reversed() {
                assert_eq!(Some(0), $memrchr(b'z', b"z\x00aaaa"));
            }

            #[test]
            fn no_match_empty_reversed() {
                assert_eq!(None, $memrchr(b'a', b""));
            }

            #[test]
            fn no_match_reversed() {
                assert_eq!(None, $memrchr(b'a', b"xyz"));
            }

            #[test]
            fn qc_never_fail_reversed() {
                fn prop(needle: u8, haystack: Vec<u8>) -> bool {
                    $memrchr(needle, &haystack); true
                }
                quickcheck::quickcheck(prop as fn(u8, Vec<u8>) -> bool);
            }

            #[test]
            fn qc_correct_memchr() {
                fn prop(v: Vec<u8>, offset: u8) -> bool {
                    // test all pointer alignments
                    let uoffset = (offset & 0xF) as usize;
                    let data = if uoffset <= v.len() {
                        &v[uoffset..]
                    } else {
                        &v[..]
                    };
                    for byte in 0..256u32 {
                        let byte = byte as u8;
                        if $memchr(byte, &data) != data.iter().position(|elt| *elt == byte) {
                            return false;
                        }
                    }
                    true
                }
                quickcheck::quickcheck(prop as fn(Vec<u8>, u8) -> bool);
            }

            #[test]
            fn qc_correct_memrchr() {
                fn prop(v: Vec<u8>, offset: u8) -> bool {
                    // test all pointer alignments
                    let uoffset = (offset & 0xF) as usize;
                    let data = if uoffset <= v.len() {
                        &v[uoffset..]
                    } else {
                        &v[..]
                    };
                    for byte in 0..256u32 {
                        let byte = byte as u8;
                        if $memrchr(byte, &data) != data.iter().rposition(|elt| *elt == byte) {
                            return false;
                        }
                    }
                    true
                }
                quickcheck::quickcheck(prop as fn(Vec<u8>, u8) -> bool);
            }
            }
        }
    }

    memchr_tests! { native, ::memchr, ::memrchr }
    memchr_tests! { fallback, ::fallback::memchr, ::fallback::memrchr }

    #[test]
    fn memchr2_matches_one() {
        assert_eq!(Some(0), memchr2(b'a', b'b', b"a"));
        assert_eq!(Some(0), memchr2(b'a', b'b', b"b"));
        assert_eq!(Some(0), memchr2(b'b', b'a', b"a"));
        assert_eq!(Some(0), memchr2(b'b', b'a', b"b"));
    }

    #[test]
    fn memchr2_matches_begin() {
        assert_eq!(Some(0), memchr2(b'a', b'b', b"aaaa"));
        assert_eq!(Some(0), memchr2(b'a', b'b', b"bbbb"));
    }

    #[test]
    fn memchr2_matches_end() {
        assert_eq!(Some(4), memchr2(b'z', b'y', b"aaaaz"));
        assert_eq!(Some(4), memchr2(b'z', b'y', b"aaaay"));
    }

    #[test]
    fn memchr2_matches_nul() {
        assert_eq!(Some(4), memchr2(b'\x00', b'z', b"aaaa\x00"));
        assert_eq!(Some(4), memchr2(b'z', b'\x00', b"aaaa\x00"));
    }

    #[test]
    fn memchr2_matches_past_nul() {
        assert_eq!(Some(5), memchr2(b'z', b'y', b"aaaa\x00z"));
        assert_eq!(Some(5), memchr2(b'y', b'z', b"aaaa\x00z"));
    }

    #[test]
    fn memchr2_no_match_empty() {
        assert_eq!(None, memchr2(b'a', b'b', b""));
        assert_eq!(None, memchr2(b'b', b'a', b""));
    }

    #[test]
    fn memchr2_no_match() {
        assert_eq!(None, memchr2(b'a', b'b', b"xyz"));
    }

    #[test]
    fn qc_never_fail_memchr2() {
        fn prop(needle1: u8, needle2: u8, haystack: Vec<u8>) -> bool {
            memchr2(needle1, needle2, &haystack);
            true
        }
        quickcheck::quickcheck(prop as fn(u8, u8, Vec<u8>) -> bool);
    }

    #[test]
    fn memchr3_matches_one() {
        assert_eq!(Some(0), memchr3(b'a', b'b', b'c', b"a"));
        assert_eq!(Some(0), memchr3(b'a', b'b', b'c', b"b"));
        assert_eq!(Some(0), memchr3(b'a', b'b', b'c', b"c"));
    }

    #[test]
    fn memchr3_matches_begin() {
        assert_eq!(Some(0), memchr3(b'a', b'b', b'c', b"aaaa"));
        assert_eq!(Some(0), memchr3(b'a', b'b', b'c', b"bbbb"));
        assert_eq!(Some(0), memchr3(b'a', b'b', b'c', b"cccc"));
    }

    #[test]
    fn memchr3_matches_end() {
        assert_eq!(Some(4), memchr3(b'z', b'y', b'x', b"aaaaz"));
        assert_eq!(Some(4), memchr3(b'z', b'y', b'x', b"aaaay"));
        assert_eq!(Some(4), memchr3(b'z', b'y', b'x', b"aaaax"));
    }

    #[test]
    fn memchr3_matches_nul() {
        assert_eq!(Some(4), memchr3(b'\x00', b'z', b'y', b"aaaa\x00"));
        assert_eq!(Some(4), memchr3(b'z', b'\x00', b'y', b"aaaa\x00"));
        assert_eq!(Some(4), memchr3(b'z', b'y', b'\x00', b"aaaa\x00"));
    }

    #[test]
    fn memchr3_matches_past_nul() {
        assert_eq!(Some(5), memchr3(b'z', b'y', b'x', b"aaaa\x00z"));
        assert_eq!(Some(5), memchr3(b'y', b'z', b'x', b"aaaa\x00z"));
        assert_eq!(Some(5), memchr3(b'y', b'x', b'z', b"aaaa\x00z"));
    }

    #[test]
    fn memchr3_no_match_empty() {
        assert_eq!(None, memchr3(b'a', b'b', b'c', b""));
        assert_eq!(None, memchr3(b'b', b'a', b'c', b""));
        assert_eq!(None, memchr3(b'c', b'b', b'a', b""));
    }

    #[test]
    fn memchr3_no_match() {
        assert_eq!(None, memchr3(b'a', b'b', b'c', b"xyz"));
    }

    #[test]
    fn memchr_iter() {
        let haystack = b"aaaabaaaab";
        let mut memchr_iter = Memchr::new(b'b', haystack);
        let first = memchr_iter.next();
        let second = memchr_iter.next();
        let third = memchr_iter.next();
        assert_eq!(Some(5), first);
        assert_eq!(Some(10), second);
        assert_eq!(None, third);
    }

    #[test]
    fn memchr2_iter() {
        let haystack = b"ab";
        let mut memchr_iter = Memchr2::new(b'a', b'b', haystack);
        let first = memchr_iter.next();
        let second = memchr_iter.next();
        let third = memchr_iter.next();
        assert_eq!(Some(1), first);
        assert_eq!(Some(2), second);
        assert_eq!(None, third);
    }

    #[test]
    fn memchr3_iter() {
        let haystack = b"abc";
        let mut memchr_iter = Memchr3::new(b'a', b'b', b'c', haystack);
        let first = memchr_iter.next();
        let second = memchr_iter.next();
        let third = memchr_iter.next();
        let fourth = memchr_iter.next();
        assert_eq!(Some(1), first);
        assert_eq!(Some(2), second);
        assert_eq!(Some(3), third);
        assert_eq!(None, fourth);
    }

    #[test]
    fn memchr_reverse_iter() {
        let haystack = b"aaaabaaaabaaaab";
        let mut memchr_iter = Memchr::new(b'b', haystack);
        let first = memchr_iter.next();
        let second = memchr_iter.next_back();
        let third = memchr_iter.next();
        let fourth = memchr_iter.next_back();

        assert_eq!(Some(5), first);
        assert_eq!(Some(15), second);
        assert_eq!(Some(10), third);
        assert_eq!(None, fourth);
    }

    #[test]
    fn memrchr_iter(){
        let haystack = b"aaaabaaaabaaaab";
        let mut memchr_iter = Memchr::new(b'b', haystack);
        let first = memchr_iter.next_back();
        let second = memchr_iter.next_back();
        let third = memchr_iter.next_back();
        let fourth = memchr_iter.next_back();

        assert_eq!(Some(15), first);
        assert_eq!(Some(10), second);
        assert_eq!(Some(5), third);
        assert_eq!(None, fourth);

    }

    #[test]
    fn qc_never_fail_memchr3() {
        fn prop(needle1: u8, needle2: u8, needle3: u8, haystack: Vec<u8>) -> bool {
            memchr3(needle1, needle2, needle3, &haystack);
            true
        }
        quickcheck::quickcheck(prop as fn(u8, u8, u8, Vec<u8>) -> bool);
    }

    #[test]
    fn qc_correct_memchr() {
        fn prop(v: Vec<u8>, offset: u8) -> bool {
            // test all pointer alignments
            let uoffset = (offset & 0xF) as usize;
            let data = if uoffset <= v.len() {
                &v[uoffset..]
            } else {
                &v[..]
            };
            for byte in 0..256u32 {
                let byte = byte as u8;
                if memchr(byte, &data) != data.iter().position(|elt| *elt == byte) {
                    return false;
                }
            }
            true
        }
        quickcheck::quickcheck(prop as fn(Vec<u8>, u8) -> bool);
    }

    #[test]
    fn qc_correct_memrchr() {
        fn prop(v: Vec<u8>, offset: u8) -> bool {
            // test all pointer alignments
            let uoffset = (offset & 0xF) as usize;
            let data = if uoffset <= v.len() {
                &v[uoffset..]
            } else {
                &v[..]
            };
            for byte in 0..256u32 {
                let byte = byte as u8;
                if memrchr(byte, &data) != data.iter().rposition(|elt| *elt == byte) {
                    return false;
                }
            }
            true
        }
        quickcheck::quickcheck(prop as fn(Vec<u8>, u8) -> bool);
    }

    #[test]
    fn qc_correct_memchr2() {
        fn prop(v: Vec<u8>, offset: u8) -> bool {
            // test all pointer alignments
            let uoffset = (offset & 0xF) as usize;
            let data = if uoffset <= v.len() {
                &v[uoffset..]
            } else {
                &v[..]
            };
            for b1 in 0..256u32 {
                for b2 in 0..256u32 {
                    let (b1, b2) = (b1 as u8, b2 as u8);
                    let expected = data.iter().position(|&b| b == b1 || b == b2);
                    let got = memchr2(b1, b2, &data);
                    if expected != got {
                        return false;
                    }
                }
            }
            true
        }
        quickcheck::quickcheck(prop as fn(Vec<u8>, u8) -> bool);
    }
}
