set shell := ["bash", "-euo", "pipefail", "-c"]

image := "localhost/agent-dev-base:fedora-44"

build:
    podman build \
        --tag "{{image}}" \
        --file containers/agent-dev-base/Containerfile \
        containers/agent-dev-base

test: build
    containers/agent-dev-base/tests/smoke-test.sh "{{image}}"
