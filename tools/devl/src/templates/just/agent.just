set shell := ["bash", "-euo", "pipefail", "-c"]

repo_root := justfile_directory()

# Show agent role, refs, and commits after human/main.
status:
    @cd '{{ repo_root }}' && scripts/workspace-git agent-status

# Rebase clean agent/codex work onto the supplied human/main baseline.
from-human:
    @cd '{{ repo_root }}' && scripts/workspace-git agent-from-human

# Check clean agent/codex, then set refs/handoffs/codex to its HEAD.
to-human:
    @cd '{{ repo_root }}' && scripts/workspace-git agent-to-human

