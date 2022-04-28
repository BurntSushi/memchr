/// Feature detection for AVX2.
///
/// This function is equivalent to `std::is_x86_feature_detected!("avx2")`.
#[inline]
#[cfg(all(not(test), feature = "std"))]
pub fn is_avx2_enabled() -> bool {
    std::is_x86_feature_detected!("avx2")
}

/// `no_std` feature detection for AVX2.
///
/// This function is equivalent to `std::is_x86_feature_detected!("avx2")`.
#[inline]
#[cfg(any(test, not(feature = "std")))]
pub fn is_avx2_enabled() -> bool {
    use core::sync::atomic::{AtomicU8, Ordering};

    if cfg!(target_feature = "avx2") {
        return true;
    }

    if cfg!(target_env = "sgx") {
        // CPUID isn't available on SGX
        return false;
    }

    const UNINIT: u8 = 0b00;
    const ENABLED: u8 = 0b01;
    const DISABLED: u8 = 0b10;
    static CACHE: AtomicU8 = AtomicU8::new(UNINIT);

    let cached = CACHE.load(Ordering::Relaxed);
    if cached != UNINIT {
        cached == ENABLED
    } else {
        #[cold]
        fn detect_and_init() -> bool {
            let value = detect_avx2_enabled();
            CACHE.store(
                if value { ENABLED } else { DISABLED },
                Ordering::Relaxed,
            );
            value
        }

        detect_and_init()
    }
}

// References:
// https://en.wikipedia.org/wiki/CPUID
// https://en.wikipedia.org/wiki/Control_register#XCR0_and_XSS
// https://software.intel.com/en-us/blogs/2011/04/14/is-avx-enabled/
// https://www.intel.com/content/dam/develop/external/us/en/documents/36945 §2.2
#[cfg(any(test, not(feature = "std")))]
fn detect_avx2_enabled() -> bool {
    use core::arch::x86_64::*;

    if cfg!(target_env = "sgx") {
        // CPUID isn't available on SGX
        return false;
    }

    // EAX=0: Highest Function Parameter
    let max_basic_leaf = unsafe { __cpuid(0).eax };
    if max_basic_leaf < 7 {
        // CPUID doesn't support "Extended Features"
        return false;
    }

    // EAX=7, ECX=0: Extended Features
    let extended_features_ebx = unsafe { __cpuid(7).ebx };
    if extended_features_ebx & (1 << 5) == 0 {
        // CPU doesn't support AVX2
        return false;
    }

    // EAX=1: Processor Info and Feature Bits
    let proc_info_ecx = unsafe { __cpuid(1).ecx };

    // Check XSAVE support.
    // This is required for the OS to save and restore the AVX state
    let xsave = proc_info_ecx & (1 << 26) != 0;
    if !xsave {
        return false;
    }

    // Check if XSAVE is enabled by the OS
    let osxsave = proc_info_ecx & (1 << 27) != 0;
    if !osxsave {
        return false;
    }

    // Check if the OS has set the following bits in XCR0.
    // .bit 1 = SSE enable
    // .bit 2 = AVX enable
    let xcr0 = unsafe { _xgetbv(0) };
    if xcr0 & 0b110 != 0b110 {
        return false;
    }

    true
}

#[cfg(all(test, feature = "std", not(miri)))]
mod tests {
    #[test]
    fn detect_avx2() {
        assert_eq!(
            super::is_avx2_enabled(),
            std::is_x86_feature_detected!("avx2")
        );
    }
}
