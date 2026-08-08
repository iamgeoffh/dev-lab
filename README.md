# DEV-LAB

Containers, compose files, and other tools and templates to help me develop software and agentic apps.

## `devl` CLI

The Rust workspace under `tools/` builds the `devl` command from `tools/devl/`.
Its first command
creates a clean development-lab project from embedded, generalized templates:

```sh
mise exec -- cargo run --manifest-path tools/Cargo.toml -- init /path/to/new-project
```

The destination must be new or empty. `devl init` does not overwrite files,
initialize Git, install tools, download artifacts, or start containers. The
generated project uses rootless Podman, an unprivileged container user, locked
generic tools, and an offline clone for agent work. Review its generated
`README.md`, executable configuration, and trust boundaries before using it.

Run the CLI checks separately from the base-image smoke test:

```sh
just devl check
```

The other CLI commands are `just devl build` and `just devl test`.

## Local validation

Install the repository-managed Git hooks once per clone:

```sh
just install-hooks
```

The pre-push hook runs `just containers test` before a push that updates any
remote branch other than `main`. Pushes that update only `main`, tags, or
deletions skip the local test. Git hooks are a developer guardrail rather than
an access-control boundary: `git push --no-verify` bypasses them. Treat changes
below `.githooks/` as executable code and review them before pushing.

Container commands are namespaced by collection and image:

```sh
just containers build
just containers test
just containers agent-dev-base build
just containers agent-dev-base test
```

The collection commands operate on every maintained image; currently that is
only `agent-dev-base`.

There is intentionally no GitHub Actions workflow for feature-branch pushes or
pull requests. This keeps routine validation on the developer's machine and
avoids consuming hosted-runner minutes.

## Publishing

When `main` advances, `.github/workflows/publish-agent-dev-base.yml` builds and
smoke-tests the image with rootless Podman. A successful run:

1. assigns the immutable image tag `build-<workflow-run-number>`;
2. creates the annotated Git tag
   `agent-dev-base-build-<workflow-run-number>` on the pushed commit;
3. publishes
   `ghcr.io/iamgeoffh/agent-dev-base:build-<workflow-run-number>`; and
4. updates the `fedora-44` and `latest` floating tags only when the commit is
   still the remote `main` tip.

Workflow runs are serialized, and rerunning a partially failed workflow reuses
the same build and source tags. GitHub assigns the monotonically increasing
workflow run number automatically. Authentication uses the repository-scoped
`GITHUB_TOKEN`; no registry token is stored as a repository secret.

Before the first publication, configure the repository to allow GitHub Actions,
ensure workflow tokens may receive the declared `contents: write` and
`packages: write` permissions, and protect `main` from force-pushes. After the
package is first created, choose its required GHCR visibility; public packages
can be pulled anonymously.
