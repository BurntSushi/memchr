use super::fallback;

mod neon;

/// AArch64 is a 64-bit architecture introduced with ARMv8. NEON is required
/// in all standard ARMv8 implementations, so no runtime detection is required
/// to call NEON functions.
///
/// # Safety
///
/// There are no safety requirements for this definition of the macro. It is
/// safe for all inputs since it is restricted to either the fallback routine
/// or the NEON routine, which is always safe to call on AArch64 as explained
/// previously.
macro_rules! unsafe_ifunc {
    ($fnty:ty, $name:ident, $haystack:ident, $($needle:ident),+) => {{
        if cfg!(memchr_runtime_neon) {
            unsafe { neon::$name($($needle),+, $haystack) }
        } else {
            fallback::$name($($needle),+, $haystack)
        }
    }}
}

#[inline(always)]
pub fn memchr(n1: u8, haystack: &[u8]) -> Option<usize> {
    unsafe_ifunc!(fn(u8, &[u8]) -> Option<usize>, memchr, haystack, n1)
}

#[inline(always)]
pub fn memchr2(n1: u8, n2: u8, haystack: &[u8]) -> Option<usize> {
    unsafe_ifunc!(
        fn(u8, u8, &[u8]) -> Option<usize>,
        memchr2,
        haystack,
        n1,
        n2
    )
}

#[inline(always)]
pub fn memchr3(n1: u8, n2: u8, n3: u8, haystack: &[u8]) -> Option<usize> {
    unsafe_ifunc!(
        fn(u8, u8, u8, &[u8]) -> Option<usize>,
        memchr3,
        haystack,
        n1,
        n2,
        n3
    )
}

#[inline(always)]
pub fn memrchr(n1: u8, haystack: &[u8]) -> Option<usize> {
    unsafe_ifunc!(fn(u8, &[u8]) -> Option<usize>, memrchr, haystack, n1)
}

#[inline(always)]
pub fn memrchr2(n1: u8, n2: u8, haystack: &[u8]) -> Option<usize> {
    unsafe_ifunc!(
        fn(u8, u8, &[u8]) -> Option<usize>,
        memrchr2,
        haystack,
        n1,
        n2
    )
}

#[inline(always)]
pub fn memrchr3(n1: u8, n2: u8, n3: u8, haystack: &[u8]) -> Option<usize> {
    unsafe_ifunc!(
        fn(u8, u8, u8, &[u8]) -> Option<usize>,
        memrchr3,
        haystack,
        n1,
        n2,
        n3
    )
}
