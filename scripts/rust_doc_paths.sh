#!/usr/bin/env bash
set -euo pipefail

toolchain="${1:-1.85.0}"
sysroot="$(rustc +"$toolchain" --print sysroot)"
doc_root="$sysroot/share/doc/rust/html"

echo "std: $doc_root/std/index.html"
echo "book: $doc_root/book/index.html"
echo "reference: $doc_root/reference/index.html"
