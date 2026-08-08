set shell := ["bash", "-euo", "pipefail", "-c"]

mod containers "just/containers.just"
mod devl "just/devl.just"

# List repository commands and component submodules.
default:
    @just --list --list-submodules

install-hooks:
    git config --local core.hooksPath .githooks
    @echo "Installed repository hooks from .githooks"
