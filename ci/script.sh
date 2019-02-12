#!/bin/bash

# vim: ft=sh sw=2 ts=2 sts=2

set -e

# Setup some variables for executing cargo commands.
# Things are a little different if we're testing with cross.
if [ -n "$CROSS" ]; then
  rustup target add "$TARGET"
  cargo install cross --force
  export CARGO_CMD="cross"
else
  export CARGO_CMD="cargo"
fi

is_x86_64() {
    case "$TARGET" in
        x86_64-*)
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

host() {
    case "$TRAVIS_OS_NAME" in
        linux)
            echo x86_64-unknown-linux-gnu
            ;;
        osx)
            echo x86_64-apple-darwin
            ;;
    esac
}

set -ex

if command -V lscpu > /dev/null 2>&1; then
  # Show output of current CPU for debugging purposes.
  lscpu
fi

if [[ "$(host)" != "$TARGET" ]]; then
  rustup target add "$TARGET"
fi
"$CARGO_CMD" build --target "$TARGET" --verbose --no-default-features
"$CARGO_CMD" build --target "$TARGET" --verbose
"$CARGO_CMD" doc --target "$TARGET" --verbose

# If we're testing on an older version of Rust, then only check that we
# can build the crate. This is because the dev dependencies might be updated
# more frequently, and therefore might require a newer version of Rust.
#
# This isn't ideal. It's a compromise.
if [ "$TRAVIS_RUST_VERSION" = "1.13.0" ]; then
  exit
fi

# A sanity check for the byte order used in these tests.
"$CARGO_CMD" test --target "$TARGET" --verbose byte_order -- --nocapture

"$CARGO_CMD" test --target "$TARGET" --verbose

# If we're testing on x86_64, then test all possible permutations of SIMD
# config.
if is_x86_64; then
  preamble="--cfg memchr_disable_auto_simd"

  # Force use of fallback without libc.
  RUSTFLAGS="$preamble" "$CARGO_CMD" test --target "$TARGET" --verbose

  # Force use of libc.
  RUSTFLAGS="$preamble" "$CARGO_CMD" test \
    --target "$TARGET" \
    --features libc \
    --verbose

  preamble="$preamble --cfg memchr_runtime_simd"
  # Force use of fallback even when SIMD is enabled.
  RUSTFLAGS="$preamble" "$CARGO_CMD" test --target "$TARGET" --verbose
  # Force use of sse2 only
  RUSTFLAGS="$preamble --cfg memchr_runtime_sse2" \
    "$CARGO_CMD" test --target "$TARGET" --verbose
  # Force use of avx only
  RUSTFLAGS="$preamble --cfg memchr_runtime_avx" \
    "$CARGO_CMD" test --target "$TARGET" --verbose
fi
if [[ "$TRAVIS_RUST_VERSION" = "nightly" ]] && is_x86_64 && [[ "$TRAVIS_OS_NAME" = "linux" ]]; then
  "$CARGO_CMD" bench \
    --manifest-path bench/Cargo.toml \
    --target "$TARGET" \
    --verbose \
    -- \
    --test
fi
