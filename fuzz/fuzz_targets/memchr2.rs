#![no_main]

use libfuzzer_sys::fuzz_target;
use memchr::memchr2_iter;

fuzz_target!(|data: &[u8]| {
    if data.len() < 2 {
        return;
    }
    memchr2_iter(data[0], data[1], &data[2..]).count();
});
