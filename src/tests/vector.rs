use crate::vector::{MoveMask, Vector};

fn test_move_mask_first_offset<M: MoveMask, const SIZE: usize>() {
    for n in 0usize..SIZE {
        let n_zeros = M::all_ones_except_least_significant(n);
        assert_eq!(n_zeros.first_offset(), n);
    }
}

fn test_move_mask_clear_first_offset<M: MoveMask, const SIZE: usize>() {
    for n in 0usize..SIZE {
        let n_zeros = M::all_ones_except_least_significant(n);
        assert_ne!(n_zeros.clear_least_significant_bit().first_offset(), n);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_offset() {
        #[cfg(target_arch = "aarch64")]
        test_move_mask_first_offset::<<std::arch::aarch64::uint8x16_t as Vector>::Mask, 16>();
    }

    #[test]
    fn clear_first_offset() {
        #[cfg(target_arch = "aarch64")]
        test_move_mask_clear_first_offset::<<std::arch::aarch64::uint8x16_t as Vector>::Mask, 16>();
    }
}
