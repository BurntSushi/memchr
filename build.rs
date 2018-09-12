extern crate version_check;

use std::env;

use version_check::is_min_version;

fn main() {
    enable_simd_optimizations();
}

fn enable_simd_optimizations() {
    if is_env_set("CARGO_CFG_MEMCHR_DISABLE_AUTO_SIMD") {
        return;
    }
    if !is_min_version("1.27.0").map(|(yes, _)| yes).unwrap_or(false) {
        return;
    }

    println!("cargo:rustc-cfg=memchr_runtime_simd");
    println!("cargo:rustc-cfg=memchr_runtime_sse2");
    println!("cargo:rustc-cfg=memchr_runtime_sse42");
    println!("cargo:rustc-cfg=memchr_runtime_avx");
}

fn is_env_set(name: &str) -> bool {
    env::var(name).is_ok()
}
