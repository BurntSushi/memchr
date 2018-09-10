use memchr::{Memchr, Memchr2, Memchr3, memrchr};

pub fn memchr1_count(b1: u8, haystack: &[u8]) -> usize {
    Memchr::new(b1, haystack).count()
}

pub fn naive1_count(b1: u8, haystack: &[u8]) -> usize {
    haystack.iter().filter(|&&b| b == b1).count()
}

pub fn memchr2_count(b1: u8, b2: u8, haystack: &[u8]) -> usize {
    Memchr2::new(b1, b2, haystack).count()
}

pub fn memchr3_count(b1: u8, b2: u8, b3: u8, haystack: &[u8]) -> usize {
    Memchr3::new(b1, b2, b3, haystack).count()
}

pub fn memrchr1_count(b1: u8, haystack: &[u8]) -> usize {
    let mut count = 0;
    let mut end = haystack.len();
    while let Some(i) = memrchr(b1, &haystack[..end]) {
        end = i;
        count += 1;
    }
    count
}
