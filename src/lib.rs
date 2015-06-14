/*!
This crate defines a single function, `memchr`, which exposes a safe interface
to the corresponding function in `libc`.
*/

#![deny(missing_docs)]

extern crate libc;

use libc::funcs::c95::string;
use libc::types::common::c95::c_void;
use libc::types::os::arch::c95::{c_int, size_t};

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
pub fn memchr(needle: u8, haystack: &[u8]) -> Option<usize> {
    let p = unsafe {
        string::memchr(
            haystack.as_ptr() as *const c_void,
            needle as c_int,
            haystack.len() as size_t)
    };
    if p.is_null() {
        None
    } else {
        Some(p as usize - (haystack.as_ptr() as usize))
    }
}

#[cfg(test)]
mod tests {
    extern crate quickcheck;

    use super::memchr;

    #[test]
    fn matches_one() {
        assert_eq!(Some(0), memchr(b'a', b"a"));
    }

    #[test]
    fn matches_begin() {
        assert_eq!(Some(0), memchr(b'a', b"aaaa"));
    }

    #[test]
    fn matches_end() {
        assert_eq!(Some(4), memchr(b'z', b"aaaaz"));
    }

    #[test]
    fn matches_nul() {
        assert_eq!(Some(4), memchr(b'\x00', b"aaaa\x00"));
    }

    #[test]
    fn matches_past_nul() {
        assert_eq!(Some(5), memchr(b'z', b"aaaa\x00z"));
    }

    #[test]
    fn no_match_empty() {
        assert_eq!(None, memchr(b'a', b""));
    }

    #[test]
    fn no_match() {
        assert_eq!(None, memchr(b'a', b"xyz"));
    }

    #[test]
    fn qc_never_fail() {
        fn prop(needle: u8, haystack: Vec<u8>) -> bool {
            memchr(needle, &haystack); true
        }
        quickcheck::quickcheck(prop as fn(u8, Vec<u8>) -> bool);
    }
}
