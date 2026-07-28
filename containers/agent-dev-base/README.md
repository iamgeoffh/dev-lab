# agent-dev-base

`agent-dev-base` is a reusable, headless Fedora 44 base image for development
work performed by coding agents. It is intended to run with rootless Podman on
an SELinux-enforcing Fedora host.

The image contains:

- mise 2026.7.13, installed from pinned upstream binaries with SHA-256
  verification on x86_64 and arm64;
- Git and Git LFS;
- CA certificates, curl, and common archive utilities;
- GCC, G++, make, pkg-config, and patch for standard native builds; and
- a small set of generic Bash, text, filesystem, and process utilities.

It deliberately excludes project language toolchains and dependencies,
including Python, uv, Node.js, pnpm, Rust, and Java. It also excludes coding
agents, sudo, Docker and Podman clients, container sockets, credentials, Git
identity, and user-specific configuration.

## Build and test

From the repository root, the canonical commands are:

```sh
just build
just test
```

The equivalent exact Podman commands are:

```sh
podman build \
  --tag localhost/agent-dev-base:fedora-44 \
  --file containers/agent-dev-base/Containerfile \
  containers/agent-dev-base

containers/agent-dev-base/tests/smoke-test.sh \
  localhost/agent-dev-base:fedora-44
```

The supported local image name is
`localhost/agent-dev-base:fedora-44`.

## Published image

Advancing `main` starts the repository's publishing workflow. After its first
successful run, the image is available as:

```text
ghcr.io/iamgeoffh/agent-dev-base:<version>
```

Each immutable version has the form `v0.1.<workflow-run-number>`. The workflow
creates an annotated Git tag with the identical name on the exact source
commit, so `git show <version>` retrieves the source corresponding to a
registry version. The current hosted workflow publishes `linux/amd64`. It also
maintains these convenience tags:

- `fedora-44`, the newest published Fedora 44 build from the current `main`
  tip; and
- `latest`, currently an alias for `fedora-44`.

For reproducible project images, extend an immutable version rather than a
floating tag:

```Containerfile
FROM ghcr.io/iamgeoffh/agent-dev-base:v0.1.123
```

The first live GHCR publication still depends on the workflow reaching `main`
and the repository's Actions/package permissions allowing it. Package
visibility is managed in GHCR after creation.

## Extending the image

A project image should extend the local image (or an eventual verified
registry equivalent), switch to the project working directory, copy only its
toolchain manifests first, and install the locked tools as the existing
unprivileged `dev` user. For example:

```Containerfile
FROM localhost/agent-dev-base:fedora-44

COPY --chown=dev:dev mise.toml mise.lock /workspace/
RUN mise trust /workspace/mise.toml \
    && mise install --locked
```

Projects commit `mise.toml`, `mise.lock`, and their ecosystem lockfiles. They
install language runtimes, package managers, and dependencies in the project
image rather than adding them to this shared base.

## Trust and security boundaries

The image is a containment layer, not a complete security boundary. Run it
rootlessly from a dedicated, unprivileged host account that holds no developer
credentials. Do not mount a host home directory, SSH agent, credential helper,
browser profile, password store, container socket, or other sensitive host
path. Review project Containerfiles, lifecycle hooks, mise configuration,
package scripts, and entrypoints as executable code before running them.

The build context is intentionally closed by `.containerignore`; this image
does not copy repository or host content. The default user is the unprivileged
`dev` account, with a writable home directory and `/workspace`.
