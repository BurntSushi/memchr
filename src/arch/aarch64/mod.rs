/*!
Vector algorithms for the `aarch64` target.
*/

#[cfg(target_endian = "little")]
pub mod neon;

#[cfg(target_endian = "little")]
pub(crate) mod memchr;
