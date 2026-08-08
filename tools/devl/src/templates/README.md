# {{PROJECT_NAME}}

This repository starts with a contained development lab for coding-agent work.
It uses the headless Fedora 44 `agent-dev-base` image, locked project tools, an
offline agent clone, and rootless Podman.

## Trust boundary

The container is a containment layer, not a complete security boundary. Run it
from a dedicated unprivileged host account with no developer credentials.
Review the Containerfile, lifecycle hooks, mise configuration, package scripts,
and entrypoints as executable code before running them.

The lab does not mount the host home directory, SSH agent, credential helper,
browser profile, password store, or Podman socket. Codex state is kept in the
project-specific `{{PROJECT_NAME}}-codex` named volume. Authenticate only from
inside the isolated agent container.

## Initial setup

Review all generated files, then initialize and configure the human clone:

```sh
git init
git config --local workspace.role human
mise trust mise.toml
mise install --locked
```

Add the application toolchain to `mise.toml`, generate and commit `mise.lock`,
and add the ecosystem lockfiles. Extend `just check` with the application's
real tests before relying on the CI workflow.

## Offline agent workflow

Create a separate clone or working copy for agent work, then bootstrap it from
the human clone:

```sh
just human bootstrap /absolute/path/to/agent-clone
just human to-agent
just human container up
```

The bootstrap removes all remotes from the agent clone, disables interactive
Git credentials there, records its role, and switches it to `agent/codex`.
Review the safety report and exact confirmation token before continuing.

Use `just --list --list-submodules` to see the complete workflow. Destructive
container, volume, and clone operations require explicit confirmation.

## Rootless Podman commands

Build the project image without starting it:

```sh
just human container build
```

Create the container and run project checks:

```sh
just human container up
just check
```

The Dev Container specification retains schema and CLI option names containing
`dockerfile` and `--docker-path`; this project passes the Podman executable to
that compatibility interface and does not support or mount Docker.

