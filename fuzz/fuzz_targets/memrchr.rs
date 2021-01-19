#![no_main]

use libfuzzer_sys::fuzz_target;
use memchr::memchr_iter;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    memchr_iter(data[0], &data[1..]).rev().count();
});
