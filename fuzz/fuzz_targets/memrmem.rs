#![no_main]

use libfuzzer_sys::fuzz_target;
use memchr::memmem;

fuzz_target!(|data: &[u8]| {
    if data.len() < 2 {
        return;
    }
    let split = std::cmp::max(data[0] as usize, 1) % data.len() as usize;
    let (needle, haystack) = (&data[..split], &data[split..]);
    memmem::rfind_iter(haystack, needle).count();
});
