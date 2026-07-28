#!/usr/bin/env bash
set -euo pipefail

image="${1:-localhost/agent-dev-base:fedora-44}"

configured_user="$(
    podman image inspect --format '{{.Config.User}}' "${image}"
)"
if [[ "${configured_user}" == "0" || "${configured_user}" == "root" || -z "${configured_user}" ]]; then
    echo "expected a configured non-root user, got: ${configured_user:-<empty>}" >&2
    exit 1
fi

configured_environment="$(
    podman image inspect --format '{{range .Config.Env}}{{println .}}{{end}}' "${image}"
)"
if grep -Eq '^(AWS_|AZURE_|GOOGLE_|GITHUB_TOKEN=|GH_TOKEN=|GITLAB_TOKEN=|OPENAI_API_KEY=|ANTHROPIC_API_KEY=|SSH_AUTH_SOCK=|DOCKER_HOST=|CONTAINER_HOST=)' \
    <<<"${configured_environment}"; then
    echo "image configuration contains a credential or host-integration variable" >&2
    exit 1
fi

podman run --rm \
    --network=none \
    --security-opt=no-new-privileges \
    "${image}" \
    bash -euo pipefail -c '
        [[ "$(id -u)" -ne 0 ]]
        [[ "${USER}" == "dev" ]]
        [[ "${HOME}" == "/home/dev" ]]
        [[ "${PWD}" == "/workspace" ]]

        home_probe="${HOME}/.agent-dev-base-home-write-test"
        work_probe="${PWD}/.agent-dev-base-work-write-test"
        : > "${home_probe}"
        : > "${work_probe}"
        rm -f "${home_probe}" "${work_probe}"

        [[ "$(mise --version | awk "{print \$1}")" == "2026.7.13" ]]
        git --version >/dev/null
        git lfs version >/dev/null
        curl --version >/dev/null
        tar --version >/dev/null
        gzip --version >/dev/null
        bzip2 --help >/dev/null 2>&1
        xz --version >/dev/null
        unzip -v >/dev/null
        zip -v >/dev/null
        gcc --version >/dev/null
        g++ --version >/dev/null
        make --version >/dev/null
        pkg-config --version >/dev/null
        patch --version >/dev/null
        ps --version >/dev/null
        find --version >/dev/null
        bash --version >/dev/null

        ! command -v sudo >/dev/null 2>&1
        ! rpm --quiet --query sudo
        ! command -v docker >/dev/null 2>&1
        ! command -v podman >/dev/null 2>&1
        for command in python python3 uv node npm pnpm cargo rustc java codex opencode; do
            ! command -v "${command}" >/dev/null 2>&1
        done
        [[ -z "${DOCKER_HOST:-}" ]]
        [[ -z "${CONTAINER_HOST:-}" ]]
        [[ ! -S /var/run/docker.sock ]]
        [[ ! -S /run/docker.sock ]]
        [[ ! -S /run/podman/podman.sock ]]

        for path in \
            "${HOME}/.ssh" \
            "${HOME}/.gitconfig" \
            "${HOME}/.netrc" \
            "${HOME}/.npmrc" \
            "${HOME}/.pypirc" \
            "${HOME}/.docker" \
            "${HOME}/.config/gh" \
            "${HOME}/.config/containers/auth.json" \
            "${HOME}/.config/mise/config.toml"
        do
            [[ ! -e "${path}" ]]
        done
    '

podman run --rm \
    --user root \
    --network=none \
    --security-opt=no-new-privileges \
    "${image}" \
    bash -euo pipefail -c '
        for path in \
            /root/.ssh \
            /root/.gitconfig \
            /root/.netrc \
            /root/.docker \
            /root/.config/gh \
            /root/.config/containers/auth.json \
            /root/.config/mise/config.toml
        do
            [[ ! -e "${path}" ]]
        done
    '

echo "Smoke test passed for ${image}"
