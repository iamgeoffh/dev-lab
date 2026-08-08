set shell := ["bash", "-euo", "pipefail", "-c"]

repo_root := justfile_directory()

# Show branch, worktree, role, and configured clone paths.
status:
    @cd '{{ repo_root }}' && scripts/workspace-git inspect-status

# Show the decorated repository history.
log:
    @cd '{{ repo_root }}' && git log --graph --decorate --oneline --all

# Graph only the authoritative human ref and agent output.
agent-log:
    @cd '{{ repo_root }}' && scripts/workspace-git agent-log

# Compare two commits in Meld using a directory diff.
difftool base="main" head="HEAD":
    @cd '{{ repo_root }}' && scripts/workspace-git difftool '{{ base }}' '{{ head }}'

