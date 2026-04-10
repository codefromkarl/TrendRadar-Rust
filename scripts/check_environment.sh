#!/usr/bin/env bash
set -euo pipefail

missing=0

check_cmd() {
    local name="$1"
    local cmd="$2"

    if command -v "$cmd" >/dev/null 2>&1; then
        echo "[env-check] OK: $name ($cmd)"
    else
        echo "[env-check] MISSING: $name ($cmd)"
        missing=1
    fi
}

check_rustup_component() {
    local component="$1"

    if rustup component list --installed 2>/dev/null | grep -E "^${component}(-|$)" >/dev/null 2>&1; then
        echo "[env-check] OK: rustup component $component"
    else
        echo "[env-check] MISSING: rustup component $component"
        missing=1
    fi
}

if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "[env-check] OK: current directory is a Git work tree"
else
    echo "[env-check] MISSING: current directory is not a Git work tree"
    missing=1
fi

check_cmd "git" "git"
check_cmd "rg" "rg"
check_cmd "rustup" "rustup"
check_cmd "cargo" "cargo"
check_cmd "just" "just"

check_rustup_component "rustfmt"
check_rustup_component "clippy"
check_rustup_component "rust-analyzer"
check_rustup_component "rust-src"

check_cmd "cargo-nextest" "cargo-nextest"
check_cmd "cargo-deny" "cargo-deny"
check_cmd "cargo-llvm-cov" "cargo-llvm-cov"

if [[ "$missing" -eq 1 ]]; then
    echo "[env-check] 环境检查未通过，请先执行 ./scripts/bootstrap.sh 或手动补齐工具。"
    exit 1
fi

echo "[env-check] 环境检查通过。"
