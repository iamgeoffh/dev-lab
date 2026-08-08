# Repository Agent Guide

## Purpose

This project uses a contained development lab for work by coding agents. Treat
the image as a containment layer, not a complete security boundary.

## Safety

- Inspect relevant files before changing them.
- Preserve unrelated user changes.
- Do not commit, push, publish, or modify remote resources unless explicitly
  requested.
- Never add credentials, tokens, Git identity, registry authentication, SSH
  material, user-specific configuration, or host-specific paths.
- Never mount a host home directory, SSH agent, credential helper, browser
  profile, password store, or container socket.
- Treat Containerfiles, lifecycle hooks, mise configuration, package scripts,
  and entrypoints as executable code.

## Development environment

- Use OCI and Podman terminology and commands.
- Use `Containerfile`, never `Dockerfile`.
- Run the lab with rootless Podman from a dedicated unprivileged host account
  that holds no developer credentials.
- Work inside the container as the unprivileged `dev` user.
- Declare project toolchains in committed `mise.toml`, `mise.lock`, and
  ecosystem lockfiles.
- Do not add sudo or mount a Podman socket.

## Verification

Run the narrowest relevant checks, followed by `just check`. Report only checks
that were actually run. If Podman is unavailable, report that rather than
weakening the containment rules.

