#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
workspace_dir="$(cd -- "${script_dir}/.." && pwd)"

git config --global init.defaultBranch main
git config --global pull.ff only

mise trust --yes "${workspace_dir}/mise.toml"
mise install --yes --locked --cd "${workspace_dir}"
