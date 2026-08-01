_list:
    @just --list

toolchain := "+nightly"

# Check Rust formatting.
fmt:
    cargo {{ toolchain }} fmt --all -- --check

# Lint all workspace targets.
clippy:
    cargo {{ toolchain }} clippy --workspace --all-targets --all-features -- -D warnings

# Test the workspace without documentation tests.
[private]
test-no-doc:
    cargo {{ toolchain }} nextest run --workspace --all-features --no-tests=pass

# Run documentation tests.
[private]
test-doc:
    cargo {{ toolchain }} test --workspace --doc --all-features

# Test the workspace.
[parallel]
test: test-no-doc test-doc

# Check documentation links and warnings.
docs:
    RUSTDOCFLAGS='-D warnings' cargo {{ toolchain }} doc --workspace --no-deps --all-features
