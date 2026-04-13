#!/usr/bin/env bash
set -euo pipefail

rustup set profile default
rustup toolchain install 1.85.0 --component rustfmt --component clippy --component rust-analyzer --component rust-src --component rust-docs
rustup default 1.85.0

cargo install just
# Keep tool installs compatible with the workspace's Rust 1.85.0 baseline.
cargo install cargo-nextest --version 0.9.100 --locked
cargo install cargo-llvm-cov --version 0.6.21 --locked
cargo install cargo-deny --version 0.18.3 --locked

# Build performance tools
cargo install cargo-sweep
cargo install sccache
cargo install cargo-watch

bash ./scripts/install_githooks.sh
