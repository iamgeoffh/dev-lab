AGENTS.md

Purpose

This repository builds and maintains reusable development-container images forcoding agents.

The first image is agent-dev-base, a headless Fedora 44 base image withmise preinstalled. Project repositories extend this image and use their owncommitted mise.toml, mise.lock, and ecosystem lockfiles to install theirtoolchains.

The image is a containment layer, not a complete security boundary. It isintended to run with rootless Podman under a dedicated, unprivileged hostaccount that has no developer credentials.

Repository layout

Keep each independently buildable image below containers/:

containers/
└── agent-dev-base/
    ├── Containerfile
    ├── README.md
    ├── .containerignore
    └── tests/

Put repository-wide commands in the root justfile. Keep image-specificdocumentation and tests with the image.

Working agreements

Inspect the repository and relevant files before changing them.

Make the smallest coherent change that completes the requested work.

Preserve unrelated user changes. Do not reset, discard, or rewrite them.

Do not commit, push, publish images, create releases, or modify remoteresources unless the user explicitly asks.

Do not add dependencies, tools, abstractions, or compatibility layers withouta demonstrated need.

Update documentation and smoke tests when observable image behaviour changes.

Report verification actually performed; never imply an unrun check passed.

Container rules

Use OCI and Podman terminology and commands.

Use Containerfile, never Dockerfile.

Do not introduce Docker, Docker Compose, a Docker daemon or socket, orDocker-specific assumptions.

Base agent-dev-base on the official Fedora 44 container image.

Keep the image headless.

Optimise for rootless Podman on an SELinux-enforcing Fedora host.

Run development work as an unprivileged dev user.

Do not install or configure sudo unless a later requirement explicitlyjustifies it.

Use OCI labels for useful image metadata.

Keep layers deliberate and remove package-manager caches in the layer thatcreates them.

Prefer clear, maintainable build steps over clever size optimisations.

Image contents

The base image may contain only broadly reusable development infrastructure:

mise;

Git and Git LFS;

CA certificates and network/bootstrap utilities;

common archive utilities;

standard native build essentials;

a small set of generic shell and process utilities needed in most agentenvironments.

Do not bake project toolchains or coding agents into the base image. Inparticular, do not install Python, uv, Node.js, pnpm, Rust, Java, Codex,OpenCode, or project dependencies merely for convenience. A project that needsone declares it in its own configuration.

If a new base package is proposed, require a concrete cross-project use case anddocument why it belongs in the shared image.

Supply-chain and secret handling

Pin externally downloaded bootstrap artifacts and verify their checksum orsignature.

Do not pipe network responses directly into a shell.

Prefer Fedora packages when they provide a suitable maintained version.

Keep reproducibility inputs visible in source control.

Never add credentials, tokens, Git identity, registry authentication, SSHmaterial, user-specific configuration, or host-specific paths to the image orbuild context.

Never mount or depend on a host home directory, SSH agent, credential helper,browser profile, password store, or Podman socket.

Treat the build context, lifecycle hooks, mise configuration, packagescripts, and container entrypoints as executable code.

Canonical commands

Expose routine operations through root just recipes:

just build
just test

The recipes must use rootless-compatible Podman commands and a local image nameof:

localhost/agent-dev-base:fedora-44

If command names or image names change, update this file, the root justfile,and the image README together.

Verification

After a relevant change, run the narrowest useful checks and finish with thefull smoke test when Podman is available.

The smoke test must verify at least:

the image builds with Podman;

the default/effective development user is non-root;

the development user's home and working directory are writable;

mise, Git, and the documented generic utilities run;

sudo is absent or unusable;

Docker tooling and Docker/Podman sockets are absent;

no credentials or user-specific configuration were copied into the image.

When changing package contents, also inspect the resulting package list andimage size. A build or test blocked by the environment must be reported as ablocker, not worked around by weakening these rules.

Documentation

The image README must state:

what the image contains and deliberately excludes;

its trust and security boundaries;

exact Podman build and smoke-test commands;

how project images extend it;

how project toolchains are installed with locked mise configuration;

the supported local image name and eventual GHCR naming pattern.

Do not document GHCR publishing as operational until publishing automationactually exists and has been verified.
