/// Returns true when AVX2 (and SSE2) are available in the current environment.
///
/// When AVX2 is enabled at compile time, this is a constant `true`. When it is
/// not, runtime CPU feature detection is used if the `std` feature is enabled;
/// otherwise this returns `false`.
#[inline]
pub(crate) fn has_avx2() -> bool {
    #[cfg(not(target_feature = "sse2"))]
    {
        false
    }
    #[cfg(target_feature = "sse2")]
    {
        #[cfg(target_feature = "avx2")]
        {
            true
        }
        #[cfg(not(target_feature = "avx2"))]
        {
            #[cfg(feature = "std")]
            {
                std::is_x86_feature_detected!("avx2")
            }
            #[cfg(not(feature = "std"))]
            {
                false
            }
        }
    }
}
