#!/usr/bin/env bash
# Reproducible build of the OutLayer PnL worker (wasm32-wasip2).
# Same toolchain, same Cargo.lock, and remapped paths give the same sha256 on any machine.
# Prints the wasm path on the last line of stdout. Reads no secrets.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TOOLCHAIN="${WORKER_TOOLCHAIN:-1.99.1}"
CARGO_HOME_DIR="${CARGO_HOME:-$HOME/.cargo}"
TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"

if ! rustup run "$TOOLCHAIN" rustc --version >/dev/null 2>&1; then
  rustup toolchain install "$TOOLCHAIN" --profile minimal --target wasm32-wasip2 >&2
fi
rustup target add --toolchain "$TOOLCHAIN" wasm32-wasip2 >/dev/null 2>&1 || true

# Absolute paths end up in panic messages. Map them to fixed prefixes.
export RUSTFLAGS="--remap-path-prefix=${CARGO_HOME_DIR}=/cargo --remap-path-prefix=${ROOT}=/src"
export CARGO_TARGET_DIR="$TARGET_DIR"
cargo "+${TOOLCHAIN}" build --locked -p outlayer-job --target wasm32-wasip2 --release \
  --manifest-path "$ROOT/Cargo.toml" >&2

WASM="$TARGET_DIR/wasm32-wasip2/release/outlayer-job.wasm"
if [[ ! -f "$WASM" ]]; then
  echo "Wasm was not found after the build." >&2
  exit 1
fi
echo "toolchain $(rustup run "$TOOLCHAIN" rustc --version)" >&2
echo "wasm sha256 $(sha256sum "$WASM" | awk '{print $1}')" >&2
echo "$WASM"
