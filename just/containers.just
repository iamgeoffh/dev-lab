set shell := ["bash", "-euo", "pipefail", "-c"]

mod agent-dev-base "../containers/agent-dev-base/justfile"

root_justfile := justfile()

# Build every maintained container image.
build:
    @just --justfile '{{ root_justfile }}' containers agent-dev-base build

# Build and smoke-test every maintained container image.
test:
    @just --justfile '{{ root_justfile }}' containers agent-dev-base test

