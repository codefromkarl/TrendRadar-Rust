#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]


def in_repo() -> bool:
    try:
        Path.cwd().resolve().relative_to(REPO_ROOT)
        return True
    except ValueError:
        return False


def get_rust_version() -> str:
    toolchain_file = REPO_ROOT / "rust-toolchain.toml"
    for line in toolchain_file.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if line.startswith("channel"):
            return line.split("=", 1)[1].strip().strip('"')
    return "unknown"


def local_docs_ready(version: str) -> bool:
    result = subprocess.run(
        ["rustup", "doc", "--toolchain", version, "--path", "std"],
        check=False,
        capture_output=True,
        text=True,
    )
    return result.returncode == 0 and bool(result.stdout.strip())


def main() -> int:
    if not in_repo():
        return 0

    version = get_rust_version()
    docs_ready = local_docs_ready(version)

    lines = [
        "[trendradar-rust] SessionStart reminder",
        f"- pinned Rust version: {version}",
        "- medium/large changes: update docs first, then code",
        "- small bug fixes: fix code first, then decide whether docs or dev-journal need updates",
        "- prefer local pinned Rust docs over online stable docs",
        f"- local pinned docs ready: {'yes' if docs_ready else 'no'}",
    ]
    sys.stderr.write("\n".join(lines) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
