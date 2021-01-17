use memchr::{memrchr, memrchr2, memrchr3, Memchr, Memchr2, Memchr3};

use crate::memchr::{c, fallback, naive};

pub fn memchr1_count(b1: u8, haystack: &[u8]) -> usize {
    Memchr::new(b1, haystack).count()
}

pub fn memchr1_libc_count(b1: u8, haystack: &[u8]) -> usize {
    let mut count = 0;
    let mut start = 0;
    while let Some(i) = c::memchr(b1, &haystack[start..]) {
        count += 1;
        start += i + 1;
    }
    count
}

pub fn fallback1_count(b1: u8, haystack: &[u8]) -> usize {
    let mut count = 0;
    let mut start = 0;
    while let Some(i) = fallback::memchr(b1, &haystack[start..]) {
        count += 1;
        start += i + 1;
    }
    count
}

pub fn naive1_count(b1: u8, haystack: &[u8]) -> usize {
    let mut count = 0;
    let mut start = 0;
    while let Some(i) = naive::memchr(b1, &haystack[start..]) {
        count += 1;
        start += i + 1;
    }
    count
}

pub fn memchr2_count(b1: u8, b2: u8, haystack: &[u8]) -> usize {
    Memchr2::new(b1, b2, haystack).count()
}

pub fn fallback2_count(b1: u8, b2: u8, haystack: &[u8]) -> usize {
    let mut count = 0;
    let mut start = 0;
    while let Some(i) = fallback::memchr2(b1, b2, &haystack[start..]) {
        count += 1;
        start += i + 1;
    }
    count
}

pub fn naive2_count(b1: u8, b2: u8, haystack: &[u8]) -> usize {
    let mut count = 0;
    let mut start = 0;
    while let Some(i) = naive::memchr2(b1, b2, &haystack[start..]) {
        count += 1;
        start += i + 1;
    }
    count
}

pub fn memchr3_count(b1: u8, b2: u8, b3: u8, haystack: &[u8]) -> usize {
    Memchr3::new(b1, b2, b3, haystack).count()
}

pub fn fallback3_count(b1: u8, b2: u8, b3: u8, haystack: &[u8]) -> usize {
    let mut count = 0;
    let mut start = 0;
    while let Some(i) = fallback::memchr3(b1, b2, b3, &haystack[start..]) {
        count += 1;
        start += i + 1;
    }
    count
}

pub fn naive3_count(b1: u8, b2: u8, b3: u8, haystack: &[u8]) -> usize {
    let mut count = 0;
    let mut start = 0;
    while let Some(i) = naive::memchr3(b1, b2, b3, &haystack[start..]) {
        count += 1;
        start += i + 1;
    }
    count
}

pub fn memrchr1_count(b1: u8, haystack: &[u8]) -> usize {
    let mut count = 0;
    let mut end = haystack.len();
    while let Some(i) = memrchr(b1, &haystack[..end]) {
        count += 1;
        end = i;
    }
    count
}

#[cfg(all(target_os = "linux"))]
pub fn memrchr1_libc_count(b1: u8, haystack: &[u8]) -> usize {
    let mut count = 0;
    let mut end = haystack.len();
    while let Some(i) = c::memrchr(b1, &haystack[..end]) {
        count += 1;
        end = i;
    }
    count
}

pub fn memrchr2_count(b1: u8, b2: u8, haystack: &[u8]) -> usize {
    let mut count = 0;
    let mut end = haystack.len();
    while let Some(i) = memrchr2(b1, b2, &haystack[..end]) {
        count += 1;
        end = i;
    }
    count
}

pub fn memrchr3_count(b1: u8, b2: u8, b3: u8, haystack: &[u8]) -> usize {
    let mut count = 0;
    let mut end = haystack.len();
    while let Some(i) = memrchr3(b1, b2, b3, &haystack[..end]) {
        count += 1;
        end = i;
    }
    count
}
