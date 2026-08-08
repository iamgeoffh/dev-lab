pub struct Template {
    pub path: &'static str,
    pub contents: &'static str,
    pub executable: bool,
}

pub const FILES: &[Template] = &[
    Template {
        path: ".containerignore",
        contents: include_str!("templates/containerignore"),
        executable: false,
    },
    Template {
        path: ".devcontainer/Containerfile",
        contents: include_str!("templates/devcontainer/Containerfile"),
        executable: false,
    },
    Template {
        path: ".devcontainer/devcontainer.json",
        contents: include_str!("templates/devcontainer/devcontainer.json"),
        executable: false,
    },
    Template {
        path: ".devcontainer/post-create.sh",
        contents: include_str!("templates/devcontainer/post-create.sh"),
        executable: true,
    },
    Template {
        path: ".github/workflows/devcontainer.yml",
        contents: include_str!("templates/github/workflows/devcontainer.yml"),
        executable: false,
    },
    Template {
        path: ".gitignore",
        contents: include_str!("templates/gitignore"),
        executable: false,
    },
    Template {
        path: "AGENTS.md",
        contents: include_str!("templates/AGENTS.md"),
        executable: false,
    },
    Template {
        path: "README.md",
        contents: include_str!("templates/README.md"),
        executable: false,
    },
    Template {
        path: "justfile",
        contents: include_str!("templates/justfile"),
        executable: false,
    },
    Template {
        path: "just/agent.just",
        contents: include_str!("templates/just/agent.just"),
        executable: false,
    },
    Template {
        path: "just/devcontainer.just",
        contents: include_str!("templates/just/devcontainer.just"),
        executable: false,
    },
    Template {
        path: "just/git.just",
        contents: include_str!("templates/just/git.just"),
        executable: false,
    },
    Template {
        path: "just/human.just",
        contents: include_str!("templates/just/human.just"),
        executable: false,
    },
    Template {
        path: "mise.lock",
        contents: include_str!("templates/mise.lock"),
        executable: false,
    },
    Template {
        path: "mise.toml",
        contents: include_str!("templates/mise.toml"),
        executable: false,
    },
    Template {
        path: "scripts/workspace-container",
        contents: include_str!("templates/scripts/workspace-container"),
        executable: true,
    },
    Template {
        path: "scripts/workspace-git",
        contents: include_str!("templates/scripts/workspace-git"),
        executable: true,
    },
];
