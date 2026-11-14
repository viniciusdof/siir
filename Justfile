# List commands.
[default]
help:
    @just --list

# Install pinned tools and this clone's git hooks.
setup:
    mise install
    hk install --mise

# Format Rust sources.
fmt:
    cargo fmt --all

# Fail if rustfmt would change a file. Does not write.
fmt-check:
    cargo fmt --all -- --check

# Clippy on every target and feature. A warning is an error.
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Unit, integration, and doc tests.
test:
    cargo test --all-targets --all-features

# Build every target and feature.
build:
    cargo build --all-targets --all-features

# API docs, without dependencies. A warning is an error.
[env("RUSTDOCFLAGS", "-D warnings")]
doc:
    cargo doc --no-deps --all-features

# RustSec advisories and dependency policy (license, ban, source).
audit:
    cargo audit
    cargo deny check

# Fast pre-commit checks. Does not run tests or the audit.
pre-commit: fmt-check lint

# Required gate. pre-push runs this recipe and nothing else.
ci: fmt-check lint test build doc audit
