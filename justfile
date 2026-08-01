_list:
    @just --list

# Check Rust formatting.
fmt:
    cargo +nightly fmt --all -- --check

# Lint all workspace targets.
clippy:
    cargo +nightly clippy --workspace --all-targets --all-features -- -D warnings

# Test the workspace.
test:
    cargo +nightly nextest run --workspace --all-features --no-tests=pass

# Run documentation tests.
doc-test:
    cargo +nightly test --workspace --doc --all-features

# Check documentation links and warnings.
docs:
    RUSTDOCFLAGS='-D warnings' cargo +nightly doc --workspace --no-deps --all-features
