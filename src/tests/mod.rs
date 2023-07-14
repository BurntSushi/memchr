#[macro_use]
pub(crate) mod memchr;
pub(crate) mod packedpair;
#[macro_use]
pub(crate) mod substring;

// For debugging, particularly in CI, print out the byte order of the current
// target.
#[cfg(all(feature = "std", target_endian = "little"))]
#[test]
fn byte_order() {
    std::eprintln!("LITTLE ENDIAN");
}

#[cfg(all(feature = "std", target_endian = "big"))]
#[test]
fn byte_order() {
    std::eprintln!("BIG ENDIAN");
}
