use std::arch::aarch64::*;
use std::mem::transmute;

const VEC_SIZE: usize = 16;

/// Unroll size for mem{r}chr.
const UNROLL_SIZE_1: usize = 4;
/// Unroll size for mem{r}chr{2,3}.
const UNROLL_SIZE_23: usize = 2;

#[target_feature(enable = "neon")]
pub unsafe fn memchr(n1: u8, haystack: &[u8]) -> Option<usize> {
    memchr_generic_neon::<true, 1, { 2 * 1 }, { 4 * 1 }, UNROLL_SIZE_1>(
        [n1],
        haystack,
    )
}

#[target_feature(enable = "neon")]
pub unsafe fn memchr2(n1: u8, n2: u8, haystack: &[u8]) -> Option<usize> {
    memchr_generic_neon::<true, 2, { 2 * 2 }, { 4 * 2 }, UNROLL_SIZE_23>(
        [n1, n2],
        haystack,
    )
}

#[target_feature(enable = "neon")]
pub unsafe fn memchr3(
    n1: u8,
    n2: u8,
    n3: u8,
    haystack: &[u8],
) -> Option<usize> {
    memchr_generic_neon::<true, 3, { 2 * 3 }, { 4 * 3 }, UNROLL_SIZE_23>(
        [n1, n2, n3],
        haystack,
    )
}

#[target_feature(enable = "neon")]
pub unsafe fn memrchr(n1: u8, haystack: &[u8]) -> Option<usize> {
    memchr_generic_neon::<false, 1, { 2 * 1 }, { 4 * 1 }, UNROLL_SIZE_1>(
        [n1],
        haystack,
    )
}

#[target_feature(enable = "neon")]
pub unsafe fn memrchr2(n1: u8, n2: u8, haystack: &[u8]) -> Option<usize> {
    memchr_generic_neon::<false, 2, { 2 * 2 }, { 4 * 2 }, UNROLL_SIZE_23>(
        [n1, n2],
        haystack,
    )
}

#[target_feature(enable = "neon")]
pub(crate) unsafe fn memrchr3(
    n1: u8,
    n2: u8,
    n3: u8,
    haystack: &[u8],
) -> Option<usize> {
    memchr_generic_neon::<false, 3, { 2 * 3 }, { 4 * 3 }, UNROLL_SIZE_23>(
        [n1, n2, n3],
        haystack,
    )
}

const fn generate_mask32() -> u32 {
    let mut mask = 0;
    let mut byte = 0b0000_0011;

    let mut i = 0;
    while i < 4 {
        mask |= byte;
        byte <<= 8 + 2;

        i += 1;
    }

    mask
}

const fn generate_mask64() -> u64 {
    let mut mask = 0;
    let mut byte = 0b0000_0001;

    let mut i = 0;
    while i < 8 {
        mask |= byte;
        byte <<= 8 + 1;

        i += 1;
    }

    mask
}

/// Returns true if the all bits in the register are set to 0.
#[inline(always)]
unsafe fn eq0(x: uint8x16_t) -> bool {
    low64(vpmaxq_u8(x, x)) == 0
}

#[inline(always)]
unsafe fn low64(x: uint8x16_t) -> u64 {
    vgetq_lane_u64(vreinterpretq_u64_u8(x), 0)
}

// .fold() and .reduce() cause LLVM to generate a huge dependency chain,
// so we need a custom function to explicitly parallelize the bitwise OR
// reduction to better take advantage of modern superscalar CPUs.
#[inline(always)]
unsafe fn parallel_reduce<const N: usize>(
    mut masks: [uint8x16_t; N],
) -> uint8x16_t {
    let mut len = masks.len();

    while len != 1 {
        for i in 0..len / 2 {
            masks[i] = vorrq_u8(masks[i * 2], masks[i * 2 + 1]);
        }
        if len & 1 != 0 {
            masks[0] = vorrq_u8(masks[0], masks[len - 1]);
        }
        len /= 2;
    }

    masks[0]
}

/// Search 64 bytes
#[inline(always)]
unsafe fn search64<
    const IS_FWD: bool,
    const N: usize,
    const N2: usize,
    const N4: usize,
>(
    n: [u8; N],
    ptr: *const u8,
    start_ptr: *const u8,
) -> Option<usize> {
    assert!(N4 == 4 * N);
    assert!(N2 == 2 * N);

    const MASK4: u64 = generate_mask64();

    let repmask4 = vreinterpretq_u8_u64(vdupq_n_u64(MASK4));

    let x1 = vld1q_u8(ptr);
    let x2 = vld1q_u8(ptr.add(1 * VEC_SIZE));
    let x3 = vld1q_u8(ptr.add(2 * VEC_SIZE));
    let x4 = vld1q_u8(ptr.add(3 * VEC_SIZE));

    let mut nv: [uint8x16_t; N] = [vdupq_n_u8(0); N];
    for i in 0..N {
        nv[i] = vdupq_n_u8(n[i]);
    }

    let mut masks1 = [vdupq_n_u8(0); N];
    let mut masks2 = [vdupq_n_u8(0); N];
    let mut masks3 = [vdupq_n_u8(0); N];
    let mut masks4 = [vdupq_n_u8(0); N];

    for i in 0..N {
        masks1[i] = vceqq_u8(x1, nv[i]);
        masks2[i] = vceqq_u8(x2, nv[i]);
        masks3[i] = vceqq_u8(x3, nv[i]);
        masks4[i] = vceqq_u8(x4, nv[i]);
    }

    let cmpmask = parallel_reduce({
        let mut mask1234 = [vdupq_n_u8(0); N4];
        mask1234[..N].copy_from_slice(&masks1);
        mask1234[N..2 * N].copy_from_slice(&masks2);
        mask1234[2 * N..3 * N].copy_from_slice(&masks3);
        mask1234[3 * N..4 * N].copy_from_slice(&masks4);
        mask1234
    });

    if !eq0(cmpmask) {
        let cmp1 = parallel_reduce(masks1);
        let cmp2 = parallel_reduce(masks2);
        let cmp3 = parallel_reduce(masks3);
        let cmp4 = parallel_reduce(masks4);

        let cmp1 = vandq_u8(repmask4, cmp1);
        let cmp2 = vandq_u8(repmask4, cmp2);
        let cmp3 = vandq_u8(repmask4, cmp3);
        let cmp4 = vandq_u8(repmask4, cmp4);

        let reduce1 = vpaddq_u8(cmp1, cmp2);
        let reduce2 = vpaddq_u8(cmp3, cmp4);
        let reduce3 = vpaddq_u8(reduce1, reduce2);
        let reduce4 = vpaddq_u8(reduce3, reduce3);

        let low64: u64 = low64(reduce4);

        let offset = ptr as usize - start_ptr as usize;

        if IS_FWD {
            return Some(offset + low64.trailing_zeros() as usize);
        } else {
            return Some(
                offset + (4 * VEC_SIZE - 1) - (low64.leading_zeros() as usize),
            );
        }
    }

    None
}

/// Search 32 bytes
#[inline(always)]
unsafe fn search32<
    const IS_FWD: bool,
    const N: usize,
    const N2: usize,
    const N4: usize,
>(
    n: [u8; N],
    ptr: *const u8,
    start_ptr: *const u8,
) -> Option<usize> {
    assert!(N2 == 2 * N);

    const MASK: u32 = generate_mask32();
    let repmask2 = vdupq_n_u32(MASK);

    let x1 = vld1q_u8(ptr);
    let x2 = vld1q_u8(ptr.add(VEC_SIZE));

    let mut nv: [uint8x16_t; N] = [vdupq_n_u8(0); N];
    for i in 0..N {
        nv[i] = vdupq_n_u8(n[i]);
    }

    let mut masks1 = [vdupq_n_u8(0); N];
    let mut masks2 = [vdupq_n_u8(0); N];

    for i in 0..N {
        masks1[i] = vceqq_u8(x1, nv[i]);
        masks2[i] = vceqq_u8(x2, nv[i]);
    }

    let cmpmask = parallel_reduce({
        let mut mask12 = [vdupq_n_u8(0); N2];
        mask12[..N].copy_from_slice(&masks1);
        mask12[N..2 * N].copy_from_slice(&masks2);
        mask12
    });

    if !eq0(cmpmask) {
        let cmp1 = parallel_reduce(masks1);
        let cmp2 = parallel_reduce(masks2);

        let cmp1 = vandq_u8(transmute(repmask2), cmp1);
        let cmp2 = vandq_u8(transmute(repmask2), cmp2);

        let reduce1 = vpaddq_u8(cmp1, cmp2);
        let reduce2 = vpaddq_u8(reduce1, reduce1);

        let low64: u64 = low64(reduce2);

        let offset = ptr as usize - start_ptr as usize;

        if IS_FWD {
            return Some(offset + low64.trailing_zeros() as usize / 2);
        } else {
            return Some(
                offset + (2 * VEC_SIZE - 1)
                    - (low64.leading_zeros() as usize / 2),
            );
        }
    }

    None
}

/// Search 16 bytes
#[inline(always)]
unsafe fn search16<
    const IS_FWD: bool,
    const N: usize,
    const N2: usize,
    const N4: usize,
>(
    n: [u8; N],
    ptr: *const u8,
    start_ptr: *const u8,
) -> Option<usize> {
    let repmask1 = vreinterpretq_u8_u16(vdupq_n_u16(0xF00F));

    let mut nv: [uint8x16_t; N] = [vdupq_n_u8(0); N];
    for i in 0..N {
        nv[i] = vdupq_n_u8(n[i]);
    }

    let x1 = vld1q_u8(ptr);

    let mut cmp_masks = [vdupq_n_u8(0); N];

    for i in 0..N {
        cmp_masks[i] = vceqq_u8(x1, nv[i]);
    }

    let cmpmask = parallel_reduce(cmp_masks);

    if !eq0(cmpmask) {
        let cmpmask = vandq_u8(cmpmask, repmask1);
        let combined = vpaddq_u8(cmpmask, cmpmask);
        let comb_low: u64 = low64(combined);

        let offset = ptr as usize - start_ptr as usize;

        if IS_FWD {
            return Some(offset + comb_low.trailing_zeros() as usize / 4);
        } else {
            return Some(
                offset + (VEC_SIZE - 1)
                    - (comb_low.leading_zeros() as usize / 4),
            );
        }
    }

    None
}

#[inline]
#[target_feature(enable = "neon")]
unsafe fn memchr_generic_neon<
    const IS_FWD: bool,
    const N: usize,
    const N2: usize,
    const N4: usize,
    const UNROLL: usize,
>(
    n: [u8; N],
    haystack: &[u8],
) -> Option<usize> {
    assert!(UNROLL <= 4 && UNROLL.is_power_of_two());

    let is_match = |x: u8| -> bool { n.iter().any(|&y| y == x) };

    let start_ptr = haystack.as_ptr();

    if haystack.len() < VEC_SIZE {
        if IS_FWD {
            // For whatever reason, LLVM generates significantly worse
            // code when using .copied() on the forward search, but
            // generates very good code for the reverse search (even
            // better than manual pointer arithmetic).
            return haystack.iter().position(|&x| is_match(x));
        } else {
            return haystack.iter().copied().rposition(is_match);
        }
    }

    // dynamic trait object devirtualized by LLVM upon monomorphization
    let iter: &mut dyn Iterator<Item = &[u8]>;

    let mut x1;
    let mut x2;
    let remainder;

    if IS_FWD {
        let temp = haystack.chunks_exact(UNROLL * VEC_SIZE);
        remainder = temp.remainder();
        x1 = temp;
        iter = &mut x1;
    } else {
        let temp = haystack.rchunks_exact(UNROLL * VEC_SIZE);
        remainder = temp.remainder();
        x2 = temp;
        iter = &mut x2;
    }

    let loop_search = match UNROLL {
        1 => search16::<IS_FWD, N, N2, N4>,
        2 => search32::<IS_FWD, N, N2, N4>,
        4 => search64::<IS_FWD, N, N2, N4>,
        _ => unreachable!(),
    };

    for chunk in iter {
        if let Some(idx) = loop_search(n, chunk.as_ptr(), start_ptr) {
            return Some(idx);
        }
    }

    let mut ptr = if IS_FWD {
        remainder.as_ptr()
    } else {
        remainder.as_ptr().add(remainder.len()).offset(-(VEC_SIZE as isize))
    };

    if UNROLL > 1 {
        for _ in 0..remainder.len() / VEC_SIZE {
            if let Some(idx) = if IS_FWD {
                let ret = search16::<IS_FWD, N, N2, N4>(n, ptr, start_ptr);

                ptr = ptr.add(VEC_SIZE);

                ret
            } else {
                let ret = search16::<IS_FWD, N, N2, N4>(n, ptr, start_ptr);

                ptr = ptr.offset(-(VEC_SIZE as isize));

                ret
            } {
                return Some(idx);
            }
        }
    }

    if haystack.len() % VEC_SIZE != 0 {
        // overlapped search of remainder
        if IS_FWD {
            return search16::<IS_FWD, N, N2, N4>(
                n,
                start_ptr.add(haystack.len() - VEC_SIZE),
                start_ptr,
            );
        } else {
            return search16::<IS_FWD, N, N2, N4>(n, start_ptr, start_ptr);
        }
    }

    None
}
