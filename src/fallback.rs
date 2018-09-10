// This module defines pure Rust platform independent implementations of all
// the memchr routines. We do our best to make them fast. Some of them may even
// get auto-vectorized.

use core::cmp;
use core::usize;

const LO_U64: u64 = 0x0101010101010101;
const HI_U64: u64 = 0x8080808080808080;

const LO_USIZE: usize = LO_U64 as usize;
const HI_USIZE: usize = HI_U64 as usize;

#[cfg(target_pointer_width = "32")]
const USIZE_BYTES: usize = 4;
#[cfg(target_pointer_width = "64")]
const USIZE_BYTES: usize = 8;

/// Return `true` if `x` contains any zero byte.
///
/// From *Matters Computational*, J. Arndt
///
/// "The idea is to subtract one from each of the bytes and then look for
/// bytes where the borrow propagated all the way to the most significant
/// bit."
#[inline(always)]
fn contains_zero_byte(x: usize) -> bool {
    x.wrapping_sub(LO_USIZE) & !x & HI_USIZE != 0
}

/// Repeat the given byte into a word size number. That is, every 8 bits
/// is equivalent to the given byte. For example, if `b` is `\x4E` or
/// `01001110` in binary, then the returned value on a 32-bit system would be:
/// `01001110_01001110_01001110_01001110`.
#[inline(always)]
fn repeat_byte(b: u8) -> usize {
    (b as usize) * (usize::MAX / 255)
}

/// Return the first index matching the byte `x` in `text`.
pub fn memchr(x: u8, text: &[u8]) -> Option<usize> {
    // Scan for a single byte value by reading two `usize` words at a time.
    //
    // Split `text` in three parts
    // - unaligned initial part, before first word aligned address in text
    // - body, scan by 2 words at a time
    // - the last remaining part, < 2 word size
    let len = text.len();
    let ptr = text.as_ptr();

    // search up to an aligned boundary
    let align = (ptr as usize) & (USIZE_BYTES - 1);
    let mut offset;
    if align > 0 {
        offset = cmp::min(USIZE_BYTES - align, len);
        let pos = text[..offset].iter().position(|elt| *elt == x);
        if let Some(index) = pos {
            return Some(index);
        }
    } else {
        offset = 0;
    }

    // search the body of the text
    let repeated_x = repeat_byte(x);

    if len >= 2 * USIZE_BYTES {
        while offset <= len - 2 * USIZE_BYTES {
            debug_assert_eq!((ptr as usize + offset) % USIZE_BYTES, 0);
            unsafe {
                let u = *(ptr.offset(offset as isize) as *const usize);
                let v = *(ptr.offset((offset + USIZE_BYTES) as isize) as *const usize);

                // break if there is a matching byte
                let zu = contains_zero_byte(u ^ repeated_x);
                let zv = contains_zero_byte(v ^ repeated_x);
                if zu || zv {
                    break;
                }
            }
            offset += USIZE_BYTES * 2;
        }
    }

    // find the byte after the point the body loop stopped
    text[offset..].iter().position(|elt| *elt == x).map(|i| offset + i)
}

/// Like `memchr`, but searches for two bytes instead of one.
pub fn memchr2(needle1: u8, needle2: u8, haystack: &[u8]) -> Option<usize> {
    fn slow(b1: u8, b2: u8, haystack: &[u8]) -> Option<usize> {
        haystack.iter().position(|&b| b == b1 || b == b2)
    }

    let len = haystack.len();
    let ptr = haystack.as_ptr();
    let align = (ptr as usize) & (USIZE_BYTES - 1);
    let mut i = 0;
    if align > 0 {
        i = cmp::min(USIZE_BYTES - align, len);
        if let Some(found) = slow(needle1, needle2, &haystack[..i]) {
            return Some(found);
        }
    }
    let repeated_b1 = repeat_byte(needle1);
    let repeated_b2 = repeat_byte(needle2);
    if len >= USIZE_BYTES {
        while i <= len - USIZE_BYTES {
            unsafe {
                let u = *(ptr.offset(i as isize) as *const usize);
                let found_ub1 = contains_zero_byte(u ^ repeated_b1);
                let found_ub2 = contains_zero_byte(u ^ repeated_b2);
                if found_ub1 || found_ub2 {
                    break;
                }
            }
            i += USIZE_BYTES;
        }
    }
    slow(needle1, needle2, &haystack[i..]).map(|pos| i + pos)
}

/// Like `memchr`, but searches for three bytes instead of one.
pub fn memchr3(
    needle1: u8,
    needle2: u8,
    needle3: u8,
    haystack: &[u8],
) -> Option<usize> {
    fn slow(b1: u8, b2: u8, b3: u8, haystack: &[u8]) -> Option<usize> {
        haystack.iter().position(|&b| b == b1 || b == b2 || b == b3)
    }

    let len = haystack.len();
    let ptr = haystack.as_ptr();
    let align = (ptr as usize) & (USIZE_BYTES - 1);
    let mut i = 0;
    if align > 0 {
        i = cmp::min(USIZE_BYTES - align, len);
        if let Some(found) = slow(needle1, needle2, needle3, &haystack[..i]) {
            return Some(found);
        }
    }
    let repeated_b1 = repeat_byte(needle1);
    let repeated_b2 = repeat_byte(needle2);
    let repeated_b3 = repeat_byte(needle3);
    if len >= USIZE_BYTES {
        while i <= len - USIZE_BYTES {
            unsafe {
                let u = *(ptr.offset(i as isize) as *const usize);
                let found_ub1 = contains_zero_byte(u ^ repeated_b1);
                let found_ub2 = contains_zero_byte(u ^ repeated_b2);
                let found_ub3 = contains_zero_byte(u ^ repeated_b3);
                if found_ub1 || found_ub2 || found_ub3 {
                    break;
                }
            }
            i += USIZE_BYTES;
        }
    }
    slow(needle1, needle2, needle3, &haystack[i..]).map(|pos| i + pos)
}

/// Return the last index matching the byte `x` in `text`.
pub fn memrchr(x: u8, text: &[u8]) -> Option<usize> {
    // Scan for a single byte value by reading two `usize` words at a time.
    //
    // Split `text` in three parts
    // - unaligned tail, after the last word aligned address in text
    // - body, scan by 2 words at a time
    // - the first remaining bytes, < 2 word size
    let len = text.len();
    let ptr = text.as_ptr();

    // search to an aligned boundary
    let end_align = (ptr as usize + len) & (USIZE_BYTES - 1);
    let mut offset;
    if end_align > 0 {
        offset = if end_align >= len { 0 } else { len - end_align };
        let pos = text[offset..].iter().rposition(|elt| *elt == x);
        if let Some(index) = pos {
            return Some(offset + index);
        }
    } else {
        offset = len;
    }

    // search the body of the text
    let repeated_x = repeat_byte(x);

    while offset >= 2 * USIZE_BYTES {
        debug_assert_eq!((ptr as usize + offset) % USIZE_BYTES, 0);
        unsafe {
            let u = *(ptr.offset(offset as isize - 2 * USIZE_BYTES as isize) as *const usize);
            let v = *(ptr.offset(offset as isize - USIZE_BYTES as isize) as *const usize);

            // break if there is a matching byte
            let zu = contains_zero_byte(u ^ repeated_x);
            let zv = contains_zero_byte(v ^ repeated_x);
            if zu || zv {
                break;
            }
        }
        offset -= 2 * USIZE_BYTES;
    }

    // find the byte before the point the body loop stopped
    text[..offset].iter().rposition(|elt| *elt == x)
}
