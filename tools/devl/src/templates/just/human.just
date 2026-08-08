set shell := ["bash", "-euo", "pipefail", "-c"]

mod container "devcontainer.just"

repo_root := justfile_directory()

# Safely configure an existing clone as the offline agent clone.
bootstrap path:
    @cd '{{ repo_root }}' && scripts/workspace-git human-bootstrap '{{ path }}'

# Fetch origin/main and fast-forward clean human main.
from-github:
    @cd '{{ repo_root }}' && scripts/workspace-git human-from-github

# Supply human main to the offline agent clone.
to-agent:
    @cd '{{ repo_root }}' && scripts/workspace-git human-to-agent

# Receive the prepared agent handoff.
from-agent:
    @cd '{{ repo_root }}' && scripts/workspace-git human-from-agent

# Compare received agent/codex with human main in Meld.
inspect-agent:
    @cd '{{ repo_root }}' && scripts/workspace-git human-inspect-agent

# Fast-forward clean human main to received agent/codex.
agent-to-main:
    @cd '{{ repo_root }}' && scripts/workspace-git human-agent-to-main

# Check and push exactly human main to GitHub origin main.
push-main:
    @cd '{{ repo_root }}' && scripts/workspace-git human-push-main

# Permanently dispose of the configured agent clone after a safety audit.
dispose:
    @cd '{{ repo_root }}' && scripts/workspace-git human-dispose

