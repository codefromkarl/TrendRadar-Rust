set shell := ["bash", "-uc"]

bootstrap:
    ./scripts/bootstrap.sh

install-githooks:
    ./scripts/install_githooks.sh

env-check:
    ./scripts/check_environment.sh

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all --check

lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
    cargo nextest run --workspace --all-features
    cargo test --doc --workspace

test-basic:
    cargo test --workspace

test-doc:
    cargo test --doc --workspace

check:
    cargo check --workspace --all-targets

doc:
    cargo doc --workspace --no-deps

doc-open:
    cargo doc --workspace --no-deps --open

docs-remind:
    ./scripts/doc_sync_reminder.sh

cov:
    cargo llvm-cov nextest --workspace --all-features

deny:
    cargo deny check

verify-basic: fmt-check check test-basic

# Clean build artifacts older than N days (default: 7)
sweep days="7":
    cargo sweep --time {{days}}

# Clean all unused build artifacts
sweep-all:
    cargo sweep --all

# Watch for file changes and auto-run tests
watch-test:
    cargo watch -x test

# Watch for file changes and auto-run check
watch-check:
    cargo watch -x check

verify: fmt-check lint test
