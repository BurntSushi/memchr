#![no_main]

use std::collections::HashSet;

use libfuzzer_sys::fuzz_target;
use memchr::memchrn_iter;

fuzz_target!(|data: &[u8]| {
    if data.len() < 8 {
        return;
    }
    let needles = &data[..8];
    let haystack = &data[8..];

    let unique_needles =
        needles.into_iter().map(|v| *v).collect::<HashSet<_>>();
    if unique_needles.len() != needles.len() {
        return;
    }

    let cnt1 = memchrn_iter(&needles, haystack).count();
    let cnt2 = haystack.iter().filter(|c| needles.contains(c)).count();
    assert_eq!(cnt1, cnt2);
});
