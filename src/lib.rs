/*!
TODO
*/

#![no_std]

#![deny(missing_docs)]
#![doc(html_root_url = "https://docs.rs/memchr/2.0.0")]

// In tests, we use the standard library. With some effort, we could make
// them no_std as well, but I don't see the point.
#[cfg(test)]
extern crate std;

// Supporting 16-bit would be fine. If you need it, please submit a bug report.
#[cfg(not(any(target_pointer_width = "32", target_pointer_width = "64")))]
compile_error!("memchr currently not supported on non-32 or non-64 bit");

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

pub use fallback::{memchr2, memchr3};
pub use iter::{Memchr, Memchr2, Memchr3};

#[cfg(all(feature = "libc", not(target_arch = "wasm32")))]
mod c;
#[allow(dead_code)]
mod fallback;
mod iter;
mod naive;
#[cfg(test)]
mod tests;

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
    #[cfg(all(feature = "libc", not(target_arch = "wasm32"), not(target_os = "windows")))]
    fn imp(needle: u8, haystack: &[u8]) -> Option<usize> {
        c::memchr(needle, haystack)
    }

    // use fallback on windows, since it's faster
    // use fallback on wasm32, since it doesn't have libc
    #[cfg(not(all(feature = "libc", not(target_arch = "wasm32"), not(target_os = "windows"))))]
    fn imp(needle: u8, haystack: &[u8]) -> Option<usize> {
        fallback::memchr(needle, haystack)
    }

    imp(needle, haystack)
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
    #[cfg(all(feature = "libc", target_os = "linux"))]
    fn imp(needle: u8, haystack: &[u8]) -> Option<usize> {
        c::memrchr(needle, haystack)
    }

    #[cfg(not(all(feature = "libc", target_os = "linux")))]
    fn imp(needle: u8, haystack: &[u8]) -> Option<usize> {
        fallback::memrchr(needle, haystack)
    }

    imp(needle, haystack)
}
