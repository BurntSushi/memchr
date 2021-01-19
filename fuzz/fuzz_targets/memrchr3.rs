#![no_main]

use libfuzzer_sys::fuzz_target;
use memchr::memchr3_iter;

fuzz_target!(|data: &[u8]| {
    if data.len() < 3 {
        return;
    }
    memchr3_iter(data[0], data[1], data[2], &data[3..]).rev().count();
});
