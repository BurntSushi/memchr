use core::iter::Rev;

pub use self::iter::{Memchr, Memchr2, Memchr3};

// N.B. If you're looking for the cfg knobs for libc, see build.rs.
#[cfg(memchr_libc)]
mod c;
#[allow(dead_code)]
pub mod fallback;
mod genericsimd;
mod iter;
pub mod naive;

/// An iterator over all occurrences of the needle in a haystack.
#[inline]
pub fn memchr_iter(needle: u8, haystack: &[u8]) -> Memchr<'_> {
    Memchr::new(needle, haystack)
}

/// An iterator over all occurrences of the needles in a haystack.
#[inline]
pub fn memchr2_iter(needle1: u8, needle2: u8, haystack: &[u8]) -> Memchr2<'_> {
    Memchr2::new(needle1, needle2, haystack)
}

/// An iterator over all occurrences of the needles in a haystack.
#[inline]
pub fn memchr3_iter(
    needle1: u8,
    needle2: u8,
    needle3: u8,
    haystack: &[u8],
) -> Memchr3<'_> {
    Memchr3::new(needle1, needle2, needle3, haystack)
}

/// An iterator over all occurrences of the needle in a haystack, in reverse.
#[inline]
pub fn memrchr_iter(needle: u8, haystack: &[u8]) -> Rev<Memchr<'_>> {
    Memchr::new(needle, haystack).rev()
}

/// An iterator over all occurrences of the needles in a haystack, in reverse.
#[inline]
pub fn memrchr2_iter(
    needle1: u8,
    needle2: u8,
    haystack: &[u8],
) -> Rev<Memchr2<'_>> {
    Memchr2::new(needle1, needle2, haystack).rev()
}

/// An iterator over all occurrences of the needles in a haystack, in reverse.
#[inline]
pub fn memrchr3_iter(
    needle1: u8,
    needle2: u8,
    needle3: u8,
    haystack: &[u8],
) -> Rev<Memchr3<'_>> {
    Memchr3::new(needle1, needle2, needle3, haystack).rev()
}

/// This is a helper macro to handle runtime feature detection and delegation to
/// platform-specific implementations. This is called for all exported functions
/// below to call specialized functions on a per-platform basis.
macro_rules! delegate {
    ($method:ident($($param:ident: $ty:ty),*) -> $ret:ty) => (#[allow(unreachable_code)] {
        // Miri for now always uses the naive version to avoid using simd
        // things.
        #[cfg(miri)]
        return naive::$method($($param),*);

        #[cfg(memchr_runtime_simd)]
        {
            // On x86_64 we can optionally use either sse2 or avx2 acceleration.
            // The former is 128-bits wide and the latter is 256-bits wide, so
            // the latter is preferred. The avx2 feature is detected at runtime
            // if the `std` feature is enabled, and otherwise `sse2` is always
            // enabled for x86_64 so that's called as a fallback.
            #[cfg(target_arch = "x86_64")]
            runtime_detect! {
                sig: $method($($param: $ty),*) -> $ret,
                types: if is_x86_feature_detected!("avx2") {
                    genericsimd::$method::<core::arch::x86_64::__m256i>
                } else if is_x86_feature_detected!("sse2") {
                    genericsimd::$method::<core::arch::x86_64::__m128i>
                },
            }

            // On wasm platforms when the simd128 feature is enabled then the
            // `v128` type can be used to avoid having to use the naive fallback
            // implementation of these functions.
            #[cfg(target_family = "wasm")]
            return unsafe {
                genericsimd::$method::<core::arch::wasm32::v128>($($param),*)
            };
        }

        // If the libc feature is enabled then this will delegate to the
        // appropriate libc function for the `$method` specified, or it will do
        // nothing if libc doesn't have an equivalent.
        #[cfg(memchr_libc)]
        maybe_delegate_libc!($method($($param),*));

        // If all else fails we use the in-Rust-written versions as a fallback.
        fallback::$method($($param),*)
    })
}

// This is a helper macro to do something similar to "ifunc" but also works when
// our ifunc mechanism, the `std` feature, is disabled.
#[allow(unused_macros)] // this is used conditionally so just squelch this warning
macro_rules! runtime_detect {
    (
        sig: $method:ident($($param:ident: $argty:ty),*) -> $ret:ty,
        types: $( $(else)? if $mac:ident ! ($feat:tt) { $e:path } )*,
    ) => ({
        let dst: Option<unsafe fn($($argty),*) -> $ret> = {
            // If `std` is disabled then all we can do is static dispatch. Use
            // cfg!(target_feature) for this. If no target features are
            // available then `None` is returned, meaning we'll fall through
            // to other implementations like libc or the `fallback`.
            #[cfg(not(feature = "std"))]
            {
                $(
                    if cfg!(target_feature = $feat) {
                        Some($e)
                    }
                )else*
                else {
                    None
                }
            }

            // If the "std" feature is enabled then we rely on an
            // ifunc-like-mechanism where an indirect function call is made
            // where the first invocation will switch the target of future
            // indirect calls to the appropriate implementaiton to skip the
            // feature detection.
            #[cfg(feature = "std")]
            {
                use std::sync::atomic::{AtomicPtr, Ordering};
                use std::mem;

                static FN: AtomicPtr<()> = AtomicPtr::new(detect as *mut());

                unsafe fn detect($($param: $argty),*) -> $ret {
                    let dst: unsafe fn($($argty),*) -> $ret =
                        $(
                            if $mac!($feat) {
                                $e
                            }
                        )else*
                        else {
                            fallback::$method
                        };
                    FN.store(dst as *mut (), Ordering::Relaxed);
                    dst($($param),*)
                }

                let ptr = FN.load(Ordering::Relaxed);
                Some(unsafe {
                    mem::transmute::<*mut (), unsafe fn($($argty),*) -> $ret>(ptr)
                })
            }
        };
        if let Some(dst) = dst {
            unsafe {
                return dst($($param),*);
            }
        }
    })
}

#[allow(unused_macros)] // this is used conditionally so just squelch this warning
macro_rules! maybe_delegate_libc {
    (memchr($($param:tt)*)) => (return c::memchr($($param)*););
    (memrchr($($param:tt)*)) => (return c::memrchr($($param)*););
    ($($other:tt)*) => ();
}

/// Search for the first occurrence of a byte in a slice.
///
/// This returns the index corresponding to the first occurrence of `needle` in
/// `haystack`, or `None` if one is not found. If an index is returned, it is
/// guaranteed to be less than `usize::MAX`.
///
/// While this is operationally the same as something like
/// `haystack.iter().position(|&b| b == needle)`, `memchr` will use a highly
/// optimized routine that can be up to an order of magnitude faster in some
/// cases.
///
/// # Example
///
/// This shows how to find the first position of a byte in a byte string.
///
/// ```
/// use memchr::memchr;
///
/// let haystack = b"the quick brown fox";
/// assert_eq!(memchr(b'k', haystack), Some(8));
/// ```
#[inline]
pub fn memchr(needle: u8, haystack: &[u8]) -> Option<usize> {
    if haystack.is_empty() {
        return None;
    }
    delegate!(memchr(needle: u8, haystack: &[u8]) -> Option<usize>)
}

/// Like `memchr`, but searches for either of two bytes instead of just one.
///
/// This returns the index corresponding to the first occurrence of `needle1`
/// or the first occurrence of `needle2` in `haystack` (whichever occurs
/// earlier), or `None` if neither one is found. If an index is returned, it is
/// guaranteed to be less than `usize::MAX`.
///
/// While this is operationally the same as something like
/// `haystack.iter().position(|&b| b == needle1 || b == needle2)`, `memchr2`
/// will use a highly optimized routine that can be up to an order of magnitude
/// faster in some cases.
///
/// # Example
///
/// This shows how to find the first position of either of two bytes in a byte
/// string.
///
/// ```
/// use memchr::memchr2;
///
/// let haystack = b"the quick brown fox";
/// assert_eq!(memchr2(b'k', b'q', haystack), Some(4));
/// ```
#[inline]
pub fn memchr2(needle1: u8, needle2: u8, haystack: &[u8]) -> Option<usize> {
    if haystack.is_empty() {
        return None;
    }
    delegate!(memchr2(needle1: u8, needle2: u8, haystack: &[u8]) -> Option<usize>)
}

/// Like `memchr`, but searches for any of three bytes instead of just one.
///
/// This returns the index corresponding to the first occurrence of `needle1`,
/// the first occurrence of `needle2`, or the first occurrence of `needle3` in
/// `haystack` (whichever occurs earliest), or `None` if none are found. If an
/// index is returned, it is guaranteed to be less than `usize::MAX`.
///
/// While this is operationally the same as something like
/// `haystack.iter().position(|&b| b == needle1 || b == needle2 ||
/// b == needle3)`, `memchr3` will use a highly optimized routine that can be
/// up to an order of magnitude faster in some cases.
///
/// # Example
///
/// This shows how to find the first position of any of three bytes in a byte
/// string.
///
/// ```
/// use memchr::memchr3;
///
/// let haystack = b"the quick brown fox";
/// assert_eq!(memchr3(b'k', b'q', b'e', haystack), Some(2));
/// ```
#[inline]
pub fn memchr3(
    needle1: u8,
    needle2: u8,
    needle3: u8,
    haystack: &[u8],
) -> Option<usize> {
    if haystack.is_empty() {
        return None;
    }
    delegate!(memchr3(needle1: u8, needle2: u8, needle3: u8, haystack: &[u8]) -> Option<usize>)
}

/// Search for the last occurrence of a byte in a slice.
///
/// This returns the index corresponding to the last occurrence of `needle` in
/// `haystack`, or `None` if one is not found. If an index is returned, it is
/// guaranteed to be less than `usize::MAX`.
///
/// While this is operationally the same as something like
/// `haystack.iter().rposition(|&b| b == needle)`, `memrchr` will use a highly
/// optimized routine that can be up to an order of magnitude faster in some
/// cases.
///
/// # Example
///
/// This shows how to find the last position of a byte in a byte string.
///
/// ```
/// use memchr::memrchr;
///
/// let haystack = b"the quick brown fox";
/// assert_eq!(memrchr(b'o', haystack), Some(17));
/// ```
#[inline]
pub fn memrchr(needle: u8, haystack: &[u8]) -> Option<usize> {
    if haystack.is_empty() {
        return None;
    }
    delegate!(memrchr(needle: u8, haystack: &[u8]) -> Option<usize>)
}

/// Like `memrchr`, but searches for either of two bytes instead of just one.
///
/// This returns the index corresponding to the last occurrence of `needle1` or
/// the last occurrence of `needle2` in `haystack` (whichever occurs later), or
/// `None` if neither one is found. If an index is returned, it is guaranteed
/// to be less than `usize::MAX`.
///
/// While this is operationally the same as something like
/// `haystack.iter().rposition(|&b| b == needle1 || b == needle2)`, `memrchr2`
/// will use a highly optimized routine that can be up to an order of magnitude
/// faster in some cases.
///
/// # Example
///
/// This shows how to find the last position of either of two bytes in a byte
/// string.
///
/// ```
/// use memchr::memrchr2;
///
/// let haystack = b"the quick brown fox";
/// assert_eq!(memrchr2(b'k', b'q', haystack), Some(8));
/// ```
#[inline]
pub fn memrchr2(needle1: u8, needle2: u8, haystack: &[u8]) -> Option<usize> {
    if haystack.is_empty() {
        return None;
    }
    delegate!(memrchr2(needle1: u8, needle2: u8, haystack: &[u8]) -> Option<usize>)
}

/// Like `memrchr`, but searches for any of three bytes instead of just one.
///
/// This returns the index corresponding to the last occurrence of `needle1`,
/// the last occurrence of `needle2`, or the last occurrence of `needle3` in
/// `haystack` (whichever occurs later), or `None` if none are found. If an
/// index is returned, it is guaranteed to be less than `usize::MAX`.
///
/// While this is operationally the same as something like
/// `haystack.iter().rposition(|&b| b == needle1 || b == needle2 ||
/// b == needle3)`, `memrchr3` will use a highly optimized routine that can be
/// up to an order of magnitude faster in some cases.
///
/// # Example
///
/// This shows how to find the last position of any of three bytes in a byte
/// string.
///
/// ```
/// use memchr::memrchr3;
///
/// let haystack = b"the quick brown fox";
/// assert_eq!(memrchr3(b'k', b'q', b'e', haystack), Some(8));
/// ```
#[inline]
pub fn memrchr3(
    needle1: u8,
    needle2: u8,
    needle3: u8,
    haystack: &[u8],
) -> Option<usize> {
    if haystack.is_empty() {
        return None;
    }
    delegate!(memrchr3(needle1: u8, needle2: u8, needle3: u8, haystack: &[u8]) -> Option<usize>)
}
