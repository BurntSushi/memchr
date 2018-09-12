use std::mem;
use std::sync::atomic::{AtomicUsize, Ordering};

use fallback;

mod avx;
mod sse2;

// This macro employs a gcc-like "ifunc" trick where by upon first calling
// `memchr` (for example), CPU feature detection will be performed at runtime
// to determine the best implementation to use. After CPU feature detection
// is done, we replace `memchr`'s function pointer with the selection. Upon
// subsequent invocations, the CPU-specific routine is invoked directly, which
// skips the CPU feature detection and subsequent branch that's required.
//
// While this typically doesn't matter for rare occurrences or when used on
// larger haystacks, `memchr` can be called in tight loops where the overhead
// of this branch can actually add up *and is measurable*. This trick was
// necessary to bring this implementation up to glibc's speeds for the 'tiny'
// benchmarks, for example.
//
// At some point, I expect the Rust ecosystem will get a nice macro for doing
// exactly this, at which point, we can replace our hand-jammed version of it.
//
// N.B. The ifunc strategy does prevent function inlining of course, but on
// modern CPUs, you'll probably end up with the AVX2 implementation, which
// probably can't be inlined anyway---unless you've compiled your entire
// program with AVX2 enabled. However, even then, the various memchr
// implementations aren't exactly small, so inlining might not help anyway!
macro_rules! ifunc {
    ($fnty:ty, $name:ident, $haystack:ident, $($needle:ident),+) => {{
        static mut FN: $fnty = detect;

        fn detect($($needle: u8),+, haystack: &[u8]) -> Option<usize> {
            let fun =
                if cfg!(memchr_runtime_avx) && is_x86_feature_detected!("avx2") {
                    avx::$name as usize
                } else if cfg!(memchr_runtime_sse2) {
                    sse2::$name as usize
                } else {
                    fallback::$name as usize
                };
            let slot = unsafe { &*(&FN as *const _ as *const AtomicUsize) };
            slot.store(fun as usize, Ordering::Relaxed);
            unsafe {
                mem::transmute::<usize, $fnty>(fun)($($needle),+, haystack)
            }
        }

        unsafe {
            let slot = &*(&FN as *const _ as *const AtomicUsize);
            let fun = slot.load(Ordering::Relaxed);
            mem::transmute::<usize, $fnty>(fun)($($needle),+, $haystack)
        }
    }}
}

#[inline(always)]
pub fn memchr(n1: u8, haystack: &[u8]) -> Option<usize> {
    ifunc!(fn(u8, &[u8]) -> Option<usize>, memchr, haystack, n1)
}

#[inline(always)]
pub fn memchr2(n1: u8, n2: u8, haystack: &[u8]) -> Option<usize> {
    ifunc!(fn(u8, u8, &[u8]) -> Option<usize>, memchr2, haystack, n1, n2)
}

#[inline(always)]
pub fn memchr3(n1: u8, n2: u8, n3: u8, haystack: &[u8]) -> Option<usize> {
    ifunc!(fn(u8, u8, u8, &[u8]) -> Option<usize>, memchr3, haystack, n1, n2, n3)
}

#[inline(always)]
pub fn memrchr(n1: u8, haystack: &[u8]) -> Option<usize> {
    ifunc!(fn(u8, &[u8]) -> Option<usize>, memrchr, haystack, n1)
}

#[inline(always)]
pub fn memrchr2(n1: u8, n2: u8, haystack: &[u8]) -> Option<usize> {
    ifunc!(fn(u8, u8, &[u8]) -> Option<usize>, memrchr2, haystack, n1, n2)
}

#[inline(always)]
pub fn memrchr3(n1: u8, n2: u8, n3: u8, haystack: &[u8]) -> Option<usize> {
    ifunc!(fn(u8, u8, u8, &[u8]) -> Option<usize>, memrchr3, haystack, n1, n2, n3)
}
