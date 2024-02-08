/*!
Vector algorithms for the `wasm32` target.
*/

#[cfg(target_feature = "simd128")]
pub mod simd128;

pub(crate) mod memchr;
