set shell := ["bash", "-euo", "pipefail", "-c"]

repo_root := justfile_directory()

# Show the configured agent clone's resolved devcontainer configuration.
config:
    @cd '{{ repo_root }}' && scripts/workspace-container config

# Show the configured agent clone's devcontainer status.
status:
    @cd '{{ repo_root }}' && scripts/workspace-container status

# Build the configured agent clone's image without starting it.
build:
    @cd '{{ repo_root }}' && scripts/workspace-container build

# Create and start the configured agent clone's container.
up:
    @cd '{{ repo_root }}' && scripts/workspace-container up

# Rebuild and replace the configured agent clone's container.
rebuild:
    @cd '{{ repo_root }}' && scripts/workspace-container rebuild

# Open a shell in the configured agent clone's running container.
shell:
    @cd '{{ repo_root }}' && scripts/workspace-container shell

# Start the agent container and open its clone in a new Zed window.
zed:
    @cd '{{ repo_root }}' && scripts/workspace-container zed

# Follow the configured agent clone's container logs.
logs:
    @cd '{{ repo_root }}' && scripts/workspace-container logs

# Stop the configured agent container without removing it.
stop:
    @cd '{{ repo_root }}' && scripts/workspace-container stop

# Stop and remove the configured agent container.
down:
    @cd '{{ repo_root }}' && scripts/workspace-container down

# Authenticate Codex inside the configured agent container.
login:
    @cd '{{ repo_root }}' && scripts/workspace-container login

# Delete the persistent Codex profile after a report and confirmation.
reset-codex:
    @cd '{{ repo_root }}' && scripts/workspace-container reset-codex

