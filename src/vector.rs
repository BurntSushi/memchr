/// A trait for describing vector operations used by vectorized searchers.
///
/// The trait is highly constrained to low level vector operations needed.
/// In general, it was invented mostly to be generic over x86's __m128i and
/// __m256i types. At time of writing, it also supports wasm and aarch64
/// 128-bit vector types as well.
///
/// # Safety
///
/// All methods are not safe since they are intended to be implemented using
/// vendor intrinsics, which are also not safe. Callers must ensure that the
/// appropriate target features are enabled in the calling function, and that
/// the current CPU supports them. All implementations should avoid marking the
/// routines with #[target_feature] and instead mark them as #[inline(always)]
/// to ensure they get appropriately inlined. (inline(always) cannot be used
/// with target_feature.)
pub(crate) trait Vector: Copy + core::fmt::Debug {
    /// The number of bytes in the vector. That is, this is the size of the
    /// vector in memory.
    const BYTES: usize = core::mem::size_of::<Self>();
    /// The bits that must be zero in order for a `*const u8` pointer to be
    /// correctly aligned to read vector values.
    const ALIGN: usize = core::mem::size_of::<Self>() - 1;

    /// The type of the value returned by `Vector::movemask`.
    ///
    /// This supports abstracting over the specific representation used in
    /// order to accommodate different representations in different ISAs.
    type Mask: MoveMask;

    /// Create a vector with 8-bit lanes with the given byte repeated into each
    /// lane.
    unsafe fn splat(byte: u8) -> Self;

    /// Read a vector-size number of bytes from the given pointer. The pointer
    /// must be aligned to the size of the vector.
    ///
    /// # Safety
    ///
    /// Callers must guarantee that at least `BYTES` bytes are readable from
    /// `data` and that `data` is aligned to a `BYTES` boundary.
    unsafe fn load_aligned(data: *const u8) -> Self;

    /// Read a vector-size number of bytes from the given pointer. The pointer
    /// does not need to be aligned.
    ///
    /// # Safety
    ///
    /// Callers must guarantee that at least `BYTES` bytes are readable from
    /// `data`.
    unsafe fn load_unaligned(data: *const u8) -> Self;

    /// _mm_movemask_epi8 or _mm256_movemask_epi8
    unsafe fn movemask(self) -> Self::Mask;
    /// _mm_cmpeq_epi8 or _mm256_cmpeq_epi8
    unsafe fn cmpeq(self, vector2: Self) -> Self;

    /// Compare for equality, but only the least significant value is correct.
    ///
    /// For most implementations, this is identical to [`Self::cmpeq`], but for
    /// e.g. swar vectors, it is possible to implement this more efficiently when we don't
    /// need to preserve the values of higher lanes than the least significant match.
    ///
    /// If no lanes match, the result will be all zeroes. If at least one lane matches,
    /// the result will have a value set in the least significant matching lane, and all lower
    /// lanes will be zero. The value of higher lanes is unspecified.
    #[inline(always)]
    unsafe fn cmpeq_only_least_significant_match(self, vector2: Self) -> Self {
        self.cmpeq(vector2)
    }

    /// Compare for equality, but only the most significant value is correct.
    ///
    /// For most implementations, this is identical to [`Self::cmpeq`], but for
    /// e.g. swar vectors, it is possible to implement this more efficiently when we don't
    /// need to preserve the values of lower lanes than the most significant match.
    ///
    /// If no lanes match, the result will be all zeroes. If at least one lane matches,
    /// the result will have a value set in the most significant matching lane, and all higher
    /// lanes will be zero. The value of lower lanes is unspecified.
    #[inline(always)]
    unsafe fn cmpeq_only_most_significant_match(self, vector2: Self) -> Self {
        self.cmpeq(vector2)
    }

    /// _mm_and_si128 or _mm256_and_si256
    unsafe fn and(self, vector2: Self) -> Self;
    /// _mm_or or _mm256_or_si256
    unsafe fn or(self, vector2: Self) -> Self;
    /// Returns true if and only if `Self::movemask` would return a mask that
    /// contains at least one non-zero bit.
    #[inline(always)]
    unsafe fn movemask_will_have_non_zero(self) -> bool {
        self.movemask().has_non_zero()
    }
}

/// A trait that abstracts over a vector-to-scalar operation called
/// "move mask."
///
/// On x86-64, this is `_mm_movemask_epi8` for SSE2 and `_mm256_movemask_epi8`
/// for AVX2. It takes a vector of `u8` lanes and returns a scalar where the
/// `i`th bit is set if and only if the most significant bit in the `i`th lane
/// of the vector is set. The simd128 ISA for wasm32 also supports this
/// exact same operation natively.
///
/// ... But aarch64 doesn't. So we have to fake it with more instructions and
/// a slightly different representation. We could do extra work to unify the
/// representations, but then would require additional costs in the hot path
/// for `memchr` and `packedpair`. So instead, we abstraction over the specific
/// representation with this trait and define the operations we actually need.
pub(crate) trait MoveMask: Copy + core::fmt::Debug {
    /// Return a mask that is all zeros except for the least significant `n`
    /// lanes in a corresponding vector.
    fn all_zeros_except_least_significant(n: usize) -> Self;

    /// Returns true if and only if this mask has a a non-zero bit anywhere.
    fn has_non_zero(self) -> bool;

    /// Returns the number of bits set to 1 in this mask.
    fn count_ones(self) -> usize;

    /// Does a bitwise `and` operation between `self` and `other`.
    fn and(self, other: Self) -> Self;

    /// Does a bitwise `or` operation between `self` and `other`.
    fn or(self, other: Self) -> Self;

    /// Returns a mask that is equivalent to `self` but with the least
    /// significant 1-bit set to 0.
    fn clear_least_significant_bit(self) -> Self;

    /// Returns the offset of the first non-zero lane this mask represents.
    fn first_offset(self) -> usize;

    /// Returns the offset of the last non-zero lane this mask represents.
    fn last_offset(self) -> usize;
}

/// This is a "sensible" movemask implementation where each bit represents
/// whether the most significant bit is set in each corresponding lane of a
/// vector. This is used on x86-64 and wasm, but such a mask is more expensive
/// to get on aarch64 so we use something a little different.
///
/// We call this "sensible" because this is what we get using native sse/avx
/// movemask instructions. But neon has no such native equivalent.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SensibleMoveMask(u32);

impl SensibleMoveMask {
    /// Get the mask in a form suitable for computing offsets.
    ///
    /// Basically, this normalizes to little endian. On big endian, this swaps
    /// the bytes.
    #[inline(always)]
    fn get_for_offset(self) -> u32 {
        #[cfg(target_endian = "big")]
        {
            self.0.swap_bytes()
        }
        #[cfg(target_endian = "little")]
        {
            self.0
        }
    }
}

impl MoveMask for SensibleMoveMask {
    #[inline(always)]
    fn all_zeros_except_least_significant(n: usize) -> SensibleMoveMask {
        debug_assert!(n < 32);
        SensibleMoveMask(!((1 << n) - 1))
    }

    #[inline(always)]
    fn has_non_zero(self) -> bool {
        self.0 != 0
    }

    #[inline(always)]
    fn count_ones(self) -> usize {
        self.0.count_ones() as usize
    }

    #[inline(always)]
    fn and(self, other: SensibleMoveMask) -> SensibleMoveMask {
        SensibleMoveMask(self.0 & other.0)
    }

    #[inline(always)]
    fn or(self, other: SensibleMoveMask) -> SensibleMoveMask {
        SensibleMoveMask(self.0 | other.0)
    }

    #[inline(always)]
    fn clear_least_significant_bit(self) -> SensibleMoveMask {
        SensibleMoveMask(self.0 & (self.0 - 1))
    }

    #[inline(always)]
    fn first_offset(self) -> usize {
        // We are dealing with little endian here (and if we aren't, we swap
        // the bytes so we are in practice), where the most significant byte
        // is at a higher address. That means the least significant bit that
        // is set corresponds to the position of our first matching byte.
        // That position corresponds to the number of zeros after the least
        // significant bit.
        self.get_for_offset().trailing_zeros() as usize
    }

    #[inline(always)]
    fn last_offset(self) -> usize {
        // We are dealing with little endian here (and if we aren't, we swap
        // the bytes so we are in practice), where the most significant byte is
        // at a higher address. That means the most significant bit that is set
        // corresponds to the position of our last matching byte. The position
        // from the end of the mask is therefore the number of leading zeros
        // in a 32 bit integer, and the position from the start of the mask is
        // therefore 32 - (leading zeros) - 1.
        32 - self.get_for_offset().leading_zeros() as usize - 1
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub(crate) struct SwarVector(usize);

impl SwarVector {
    const LO: usize = SwarVector::splat_const(0x01);
    const HI: usize = SwarVector::splat_const(0x80);
    const O7F: usize = SwarVector::splat_const(0x7F);

    #[inline(always)]
    const fn splat_const(byte: u8) -> usize {
        byte as usize * (usize::MAX / 255)
    }

    /// Return a mask where the top bit of each byte is set iff the value of the
    /// input for that byte was 0xff.
    ///
    /// This is a slightly more efficient method than computing the mask where zero bytes are.
    #[inline(always)]
    fn movemask_0xff_bytes(x: usize) -> usize {
        // Allow subtraction by 0x7F, by setting the most significant bit.
        let allow_subtraction = x | Self::HI;
        // If the bottom 7 bits of the byte are all set (0x7F), then subtracting 0x7F
        // will leave the top bit set, and the bottom 7 bits unset. If any of the lower bits
        // were unset, then subtracting 0x7F will unset the top bit.
        let only_top_bit = allow_subtraction - Self::O7F;
        // Remove the false positives: and with the original value: the top bit will remain set only
        // if the original value already had the top bit set before we set it.
        let without_false_positives = only_top_bit & x;

        // AND with the top bit mask to only care about the top bit.
        without_false_positives & Self::HI
    }

    /// Return a mask where `x` contains any zero byte, but only the lowest zero lane is correct
    ///
    /// Similar to [`Self::zero_byte_mask`], but only the lowest zero lane is guaranteed to be
    /// correct.
    #[inline(always)]
    fn low_zero_lane(x: usize) -> usize {
        x.wrapping_sub(Self::LO) & !x & Self::HI
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub(crate) struct SwarVectorMask(usize);

impl Vector for SwarVector {
    type Mask = SwarVectorMask;

    #[inline(always)]
    unsafe fn splat(byte: u8) -> Self {
        Self(Self::splat_const(byte))
    }

    #[inline(always)]
    unsafe fn load_aligned(data: *const u8) -> Self {
        Self(data.cast::<usize>().read().to_le())
    }

    #[inline(always)]
    unsafe fn load_unaligned(data: *const u8) -> Self {
        Self(data.cast::<usize>().read_unaligned().to_le())
    }

    #[inline(always)]
    unsafe fn movemask(self) -> Self::Mask {
        SwarVectorMask(self.0)
    }

    #[inline(always)]
    unsafe fn cmpeq(self, vector2: Self) -> Self {
        Self(Self::movemask_0xff_bytes(self.0 ^ !vector2.0))
    }

    #[inline(always)]
    unsafe fn cmpeq_only_least_significant_match(self, vector2: Self) -> Self {
        Self(Self::low_zero_lane(self.0 ^ vector2.0))
    }

    #[inline(always)]
    unsafe fn and(self, vector2: Self) -> Self {
        Self(self.0 & vector2.0)
    }

    #[inline(always)]
    unsafe fn or(self, vector2: Self) -> Self {
        Self(self.0 | vector2.0)
    }
}

impl MoveMask for SwarVectorMask {
    #[inline(always)]
    fn all_zeros_except_least_significant(n: usize) -> Self {
        Self(1 << (n * 8))
    }

    #[inline(always)]
    fn has_non_zero(self) -> bool {
        self.0 != 0
    }

    #[inline(always)]
    fn count_ones(self) -> usize {
        self.0.count_ones() as usize
    }

    #[inline(always)]
    fn and(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    #[inline(always)]
    fn or(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[inline(always)]
    fn clear_least_significant_bit(self) -> Self {
        Self(self.0 & (self.0 - 1))
    }

    #[inline(always)]
    fn first_offset(self) -> usize {
        self.0.trailing_zeros() as usize / 8
    }

    #[inline(always)]
    fn last_offset(self) -> usize {
        ((usize::BITS - self.0.leading_zeros()) / 8) as usize - 1
    }
}

#[cfg(kani)]
mod kani_tests {
    use super::*;

    #[kani::proof]
    fn mask_is_correct() {
        unsafe {
            let needle = kani::any::<u8>();
            let needle_v = SwarVector::splat(needle);
            let chunk = SwarVector(kani::any());
            let mask = needle_v.cmpeq(chunk);
            for i in 0..size_of::<usize>() {
                let byte = (chunk.0 >> (i * 8)) as u8;

                let expected_mask_byte = if byte == needle { 0x80 } else { 0 };
                let actual_mask_byte = (mask.0 >> (i * 8)) as u8;
                assert_eq!(actual_mask_byte, expected_mask_byte);
            }
        }
    }

    #[kani::proof]
    fn least_significant_match_correct() {
        unsafe {
            let needle = kani::any::<u8>();
            let needle_v = SwarVector::splat(needle);
            let chunk = SwarVector(kani::any());

            let mask = needle_v.cmpeq_only_least_significant_match(chunk);
            for i in 0..size_of::<usize>() {
                let byte = (chunk.0 >> (i * 8)) as u8;

                let expected_mask_byte = if byte == needle { 0x80 } else { 0 };
                let actual_mask_byte = (mask.0 >> (i * 8)) as u8;
                assert_eq!(actual_mask_byte, expected_mask_byte);
                if byte == needle {
                    break;
                }
            }
        }
    }
}

#[cfg(target_arch = "x86_64")]
mod x86sse2 {
    use core::arch::x86_64::*;

    use super::{SensibleMoveMask, Vector};

    impl Vector for __m128i {
        type Mask = SensibleMoveMask;

        #[inline(always)]
        unsafe fn splat(byte: u8) -> __m128i {
            _mm_set1_epi8(byte as i8)
        }

        #[inline(always)]
        unsafe fn load_aligned(data: *const u8) -> __m128i {
            _mm_load_si128(data as *const __m128i)
        }

        #[inline(always)]
        unsafe fn load_unaligned(data: *const u8) -> __m128i {
            _mm_loadu_si128(data as *const __m128i)
        }

        #[inline(always)]
        unsafe fn movemask(self) -> SensibleMoveMask {
            SensibleMoveMask(_mm_movemask_epi8(self) as u32)
        }

        #[inline(always)]
        unsafe fn cmpeq(self, vector2: Self) -> __m128i {
            _mm_cmpeq_epi8(self, vector2)
        }

        #[inline(always)]
        unsafe fn and(self, vector2: Self) -> __m128i {
            _mm_and_si128(self, vector2)
        }

        #[inline(always)]
        unsafe fn or(self, vector2: Self) -> __m128i {
            _mm_or_si128(self, vector2)
        }
    }
}

#[cfg(target_arch = "x86_64")]
mod x86avx2 {
    use core::arch::x86_64::*;

    use super::{SensibleMoveMask, Vector};

    impl Vector for __m256i {
        type Mask = SensibleMoveMask;

        #[inline(always)]
        unsafe fn splat(byte: u8) -> __m256i {
            _mm256_set1_epi8(byte as i8)
        }

        #[inline(always)]
        unsafe fn load_aligned(data: *const u8) -> __m256i {
            _mm256_load_si256(data as *const __m256i)
        }

        #[inline(always)]
        unsafe fn load_unaligned(data: *const u8) -> __m256i {
            _mm256_loadu_si256(data as *const __m256i)
        }

        #[inline(always)]
        unsafe fn movemask(self) -> SensibleMoveMask {
            SensibleMoveMask(_mm256_movemask_epi8(self) as u32)
        }

        #[inline(always)]
        unsafe fn cmpeq(self, vector2: Self) -> __m256i {
            _mm256_cmpeq_epi8(self, vector2)
        }

        #[inline(always)]
        unsafe fn and(self, vector2: Self) -> __m256i {
            _mm256_and_si256(self, vector2)
        }

        #[inline(always)]
        unsafe fn or(self, vector2: Self) -> __m256i {
            _mm256_or_si256(self, vector2)
        }
    }
}

#[cfg(target_arch = "aarch64")]
mod aarch64neon {
    use core::arch::aarch64::*;

    use super::{MoveMask, Vector};

    impl Vector for uint8x16_t {
        type Mask = NeonMoveMask;

        #[inline(always)]
        unsafe fn splat(byte: u8) -> uint8x16_t {
            vdupq_n_u8(byte)
        }

        #[inline(always)]
        unsafe fn load_aligned(data: *const u8) -> uint8x16_t {
            // I've tried `data.cast::<uint8x16_t>().read()` instead, but
            // couldn't observe any benchmark differences.
            Self::load_unaligned(data)
        }

        #[inline(always)]
        unsafe fn load_unaligned(data: *const u8) -> uint8x16_t {
            vld1q_u8(data)
        }

        #[inline(always)]
        unsafe fn movemask(self) -> NeonMoveMask {
            let asu16s = vreinterpretq_u16_u8(self);
            let mask = vshrn_n_u16(asu16s, 4);
            let asu64 = vreinterpret_u64_u8(mask);
            let scalar64 = vget_lane_u64(asu64, 0);
            NeonMoveMask(scalar64 & 0x8888888888888888)
        }

        #[inline(always)]
        unsafe fn cmpeq(self, vector2: Self) -> uint8x16_t {
            vceqq_u8(self, vector2)
        }

        #[inline(always)]
        unsafe fn and(self, vector2: Self) -> uint8x16_t {
            vandq_u8(self, vector2)
        }

        #[inline(always)]
        unsafe fn or(self, vector2: Self) -> uint8x16_t {
            vorrq_u8(self, vector2)
        }

        /// This is the only interesting implementation of this routine.
        /// Basically, instead of doing the "shift right narrow" dance, we use
        /// adjacent folding max to determine whether there are any non-zero
        /// bytes in our mask. If there are, *then* we'll do the "shift right
        /// narrow" dance. In benchmarks, this does lead to slightly better
        /// throughput, but the win doesn't appear huge.
        #[inline(always)]
        unsafe fn movemask_will_have_non_zero(self) -> bool {
            let low = vreinterpretq_u64_u8(vpmaxq_u8(self, self));
            vgetq_lane_u64(low, 0) != 0
        }
    }

    /// Neon doesn't have a `movemask` that works like the one in x86-64, so we
    /// wind up using a different method[1]. The different method also produces
    /// a mask, but 4 bits are set in the neon case instead of a single bit set
    /// in the x86-64 case. We do an extra step to zero out 3 of the 4 bits,
    /// but we still wind up with at least 3 zeroes between each set bit. This
    /// generally means that we need to do some division by 4 before extracting
    /// offsets.
    ///
    /// In fact, the existence of this type is the entire reason that we have
    /// the `MoveMask` trait in the first place. This basically lets us keep
    /// the different representations of masks without being forced to unify
    /// them into a single representation, which could result in extra and
    /// unnecessary work.
    ///
    /// [1]: https://community.arm.com/arm-community-blogs/b/infrastructure-solutions-blog/posts/porting-x86-vector-bitmask-optimizations-to-arm-neon
    #[derive(Clone, Copy, Debug)]
    pub(crate) struct NeonMoveMask(u64);

    impl NeonMoveMask {
        /// Get the mask in a form suitable for computing offsets.
        ///
        /// Basically, this normalizes to little endian. On big endian, this
        /// swaps the bytes.
        #[inline(always)]
        fn get_for_offset(self) -> u64 {
            #[cfg(target_endian = "big")]
            {
                self.0.swap_bytes()
            }
            #[cfg(target_endian = "little")]
            {
                self.0
            }
        }
    }

    impl MoveMask for NeonMoveMask {
        #[inline(always)]
        fn all_zeros_except_least_significant(n: usize) -> NeonMoveMask {
            debug_assert!(n < 16);
            NeonMoveMask(!(((1 << n) << 2) - 1))
        }

        #[inline(always)]
        fn has_non_zero(self) -> bool {
            self.0 != 0
        }

        #[inline(always)]
        fn count_ones(self) -> usize {
            self.0.count_ones() as usize
        }

        #[inline(always)]
        fn and(self, other: NeonMoveMask) -> NeonMoveMask {
            NeonMoveMask(self.0 & other.0)
        }

        #[inline(always)]
        fn or(self, other: NeonMoveMask) -> NeonMoveMask {
            NeonMoveMask(self.0 | other.0)
        }

        #[inline(always)]
        fn clear_least_significant_bit(self) -> NeonMoveMask {
            NeonMoveMask(self.0 & (self.0 - 1))
        }

        #[inline(always)]
        fn first_offset(self) -> usize {
            // We are dealing with little endian here (and if we aren't,
            // we swap the bytes so we are in practice), where the most
            // significant byte is at a higher address. That means the least
            // significant bit that is set corresponds to the position of our
            // first matching byte. That position corresponds to the number of
            // zeros after the least significant bit.
            //
            // Note that unlike `SensibleMoveMask`, this mask has its bits
            // spread out over 64 bits instead of 16 bits (for a 128 bit
            // vector). Namely, where as x86-64 will turn
            //
            //   0x00 0xFF 0x00 0x00 0xFF
            //
            // into 10010, our neon approach will turn it into
            //
            //   10000000000010000000
            //
            // And this happens because neon doesn't have a native `movemask`
            // instruction, so we kind of fake it[1]. Thus, we divide the
            // number of trailing zeros by 4 to get the "real" offset.
            //
            // [1]: https://community.arm.com/arm-community-blogs/b/infrastructure-solutions-blog/posts/porting-x86-vector-bitmask-optimizations-to-arm-neon
            (self.get_for_offset().trailing_zeros() >> 2) as usize
        }

        #[inline(always)]
        fn last_offset(self) -> usize {
            // See comment in `first_offset` above. This is basically the same,
            // but coming from the other direction.
            16 - (self.get_for_offset().leading_zeros() >> 2) as usize - 1
        }
    }
}

#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
mod wasm_simd128 {
    use core::arch::wasm32::*;

    use super::{SensibleMoveMask, Vector};

    impl Vector for v128 {
        type Mask = SensibleMoveMask;

        #[inline(always)]
        unsafe fn splat(byte: u8) -> v128 {
            u8x16_splat(byte)
        }

        #[inline(always)]
        unsafe fn load_aligned(data: *const u8) -> v128 {
            *data.cast()
        }

        #[inline(always)]
        unsafe fn load_unaligned(data: *const u8) -> v128 {
            v128_load(data.cast())
        }

        #[inline(always)]
        unsafe fn movemask(self) -> SensibleMoveMask {
            SensibleMoveMask(u8x16_bitmask(self).into())
        }

        #[inline(always)]
        unsafe fn cmpeq(self, vector2: Self) -> v128 {
            u8x16_eq(self, vector2)
        }

        #[inline(always)]
        unsafe fn and(self, vector2: Self) -> v128 {
            v128_and(self, vector2)
        }

        #[inline(always)]
        unsafe fn or(self, vector2: Self) -> v128 {
            v128_or(self, vector2)
        }
    }
}
