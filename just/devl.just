set shell := ["bash", "-euo", "pipefail", "-c"]

repo_root := justfile_directory()
manifest := repo_root / "tools/Cargo.toml"

# Build the devl CLI.
build:
    mise exec -- cargo build --manifest-path '{{ manifest }}'

# Run the devl CLI test suite.
test:
    mise exec -- cargo test --manifest-path '{{ manifest }}'

# Format, lint, and test devl.
check:
    mise exec -- cargo fmt --manifest-path '{{ manifest }}' --check
    mise exec -- cargo clippy --manifest-path '{{ manifest }}' --all-targets --all-features -- -D warnings
    mise exec -- cargo test --manifest-path '{{ manifest }}'
