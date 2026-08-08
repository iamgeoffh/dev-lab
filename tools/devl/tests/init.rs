//! Black-box integration tests for `devl init`.
//!
//! Unlike the unit tests beside the implementation in `src/lib.rs`, every test
//! in this file starts the compiled `devl` executable through
//! `CARGO_BIN_EXE_devl`. Together they verify the command's externally visible
//! contract:
//!
//! - the exact generated file set and project-name substitutions;
//! - default initialization into the current directory;
//! - refusal to modify a nonempty destination;
//! - byte-for-byte deterministic output;
//! - exclusion of application-specific, personal, secret, host-specific, and
//!   socket-related values from generated files;
//! - executable permissions on generated shell scripts; and
//! - a complete offline Git handoff from a human clone to an agent clone and
//!   back to human `main`.
//!
//! Temporary directories isolate each test and are removed when its
//! `TestDirectory` guard is dropped. The tests do not access a network, start a
//! container, or modify the repository under test.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock is after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "devl-test-{label}-{}-{nonce}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).expect("create test directory");
        Self(path)
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn devl() -> Command {
    Command::new(env!("CARGO_BIN_EXE_devl"))
}

fn relative_files(root: &Path) -> Vec<PathBuf> {
    fn visit(root: &Path, directory: &Path, files: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(directory).expect("read generated directory") {
            let entry = entry.expect("read generated entry");
            if entry.file_type().expect("read generated type").is_dir() {
                visit(root, &entry.path(), files);
            } else {
                files.push(entry.path().strip_prefix(root).unwrap().to_owned());
            }
        }
    }

    let mut files = Vec::new();
    visit(root, root, &mut files);
    files.sort();
    files
}

#[test]
fn initializes_expected_project_without_side_effects() {
    // Verifies the primary new-directory path: normalization is reported to the
    // user, the complete expected scaffold is created, project-specific names
    // are substituted, fixed ports are absent, and Git is not initialized as
    // an implicit side effect.
    let parent = TestDirectory::new("new");
    let destination = parent.0.join("My_Secure.App");
    let output = devl()
        .args(["init", destination.to_str().unwrap()])
        .output()
        .expect("run devl");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("my-secure-app"));
    assert_eq!(
        relative_files(&destination),
        [
            ".containerignore",
            ".devcontainer/Containerfile",
            ".devcontainer/devcontainer.json",
            ".devcontainer/post-create.sh",
            ".github/workflows/devcontainer.yml",
            ".gitignore",
            "AGENTS.md",
            "README.md",
            "just/agent.just",
            "just/devcontainer.just",
            "just/git.just",
            "just/human.just",
            "justfile",
            "mise.lock",
            "mise.toml",
            "scripts/workspace-container",
            "scripts/workspace-git",
        ]
        .into_iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>()
    );
    assert!(!destination.join(".git").exists());

    let configuration = fs::read_to_string(destination.join(".devcontainer/devcontainer.json"))
        .expect("read configuration");
    assert!(configuration.contains("my-secure-app-agent-dev"));
    assert!(configuration.contains("my-secure-app-codex"));
    assert!(!configuration.contains("appPort"));
    assert!(!configuration.contains("8180"));
}

#[test]
fn initializes_current_empty_directory() {
    // Verifies that omitting PATH targets the process's current directory and
    // succeeds when that directory already exists but is empty.
    let destination = TestDirectory::new("current-project");
    let output = devl()
        .arg("init")
        .current_dir(&destination.0)
        .output()
        .expect("run devl");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(destination.0.join("justfile").is_file());
}

#[test]
fn refuses_nonempty_destination_without_changing_it() {
    // Verifies the non-destructive boundary: a nonempty destination is rejected,
    // its existing bytes are preserved, and no generated files are left behind.
    let destination = TestDirectory::new("occupied");
    let sentinel = destination.0.join("keep.txt");
    fs::write(&sentinel, "user data").unwrap();

    let output = devl()
        .args(["init", destination.0.to_str().unwrap()])
        .output()
        .expect("run devl");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("not empty"));
    assert_eq!(fs::read_to_string(sentinel).unwrap(), "user data");
    assert_eq!(relative_files(&destination.0), [PathBuf::from("keep.txt")]);
}

#[test]
fn output_is_deterministic_for_the_same_project_name() {
    // Verifies reproducibility by generating the same normalized project name
    // under two different parents and comparing every path and byte. Host paths
    // and run-specific values must therefore never leak into template output.
    let first_parent = TestDirectory::new("deterministic-a");
    let second_parent = TestDirectory::new("deterministic-b");
    let first = first_parent.0.join("sample");
    let second = second_parent.0.join("sample");

    assert!(
        devl()
            .args(["init", first.to_str().unwrap()])
            .status()
            .unwrap()
            .success()
    );
    assert!(
        devl()
            .args(["init", second.to_str().unwrap()])
            .status()
            .unwrap()
            .success()
    );

    let paths = relative_files(&first);
    assert_eq!(paths, relative_files(&second));
    for path in paths {
        assert_eq!(
            fs::read(first.join(&path)).unwrap(),
            fs::read(second.join(path)).unwrap()
        );
    }
}

#[test]
fn generated_files_exclude_reference_application_and_sensitive_values() {
    // Verifies template hygiene across the entire generated tree. The denylist
    // represents values found in the reference application or prohibited by the
    // lab's secret, host-isolation, fixed-port, and Podman-only requirements.
    // It also checks that no internal template placeholder reaches the user.
    let parent = TestDirectory::new("clean");
    let destination = parent.0.join("clean-room");
    assert!(
        devl()
            .args(["init", destination.to_str().unwrap()])
            .status()
            .unwrap()
            .success()
    );

    let combined = relative_files(&destination)
        .iter()
        .map(|path| fs::read_to_string(destination.join(path)).unwrap())
        .collect::<Vec<_>>()
        .join("\n");

    for forbidden in [
        "Geoff Hill",
        "codex@iamgeoffh.com",
        "sites-iamgeoffh",
        "8180",
        "pnpm",
        "pandoc",
        "wrangler",
        "command -v docker",
        "docker.sock",
        "podman.sock",
        "/home/w1",
    ] {
        assert!(
            !combined.contains(forbidden),
            "found forbidden value {forbidden:?}"
        );
    }
    assert!(!combined.contains("{{PROJECT_NAME}}"));
}

#[cfg(unix)]
#[test]
fn generated_scripts_are_executable() {
    // Verifies the Unix filesystem contract required for lifecycle and workspace
    // scripts to run directly after generation. This test is Unix-only because
    // executable permission bits are not portable to other platforms.
    use std::os::unix::fs::PermissionsExt;

    let parent = TestDirectory::new("modes");
    let destination = parent.0.join("modes");
    assert!(
        devl()
            .args(["init", destination.to_str().unwrap()])
            .status()
            .unwrap()
            .success()
    );

    for path in [
        ".devcontainer/post-create.sh",
        "scripts/workspace-container",
        "scripts/workspace-git",
    ] {
        let mode = fs::metadata(destination.join(path))
            .unwrap()
            .permissions()
            .mode();
        assert_ne!(mode & 0o111, 0, "{path} is not executable");
    }
}

#[test]
fn generated_offline_git_handoff_round_trip_works() {
    // Verifies that the generated workflow is operational, rather than merely
    // present. It bootstraps an ordinary clone as an offline agent clone, proves
    // that all remotes were removed and agent/codex was selected, commits agent
    // work, prepares and receives the handoff ref, fast-forwards human main, and
    // finally checks the landed file contents.
    let parent = TestDirectory::new("git-workflow");
    let human = parent.0.join("human-project");
    let agent = parent.0.join("agent-project");
    assert!(
        devl()
            .args(["init", human.to_str().unwrap()])
            .status()
            .unwrap()
            .success()
    );

    git(&human, ["init", "--initial-branch=main"]);
    git(&human, ["add", "."]);
    git(
        &human,
        [
            "-c",
            "user.name=Test User",
            "-c",
            "user.email=test@example.invalid",
            "commit",
            "-m",
            "Initial project",
        ],
    );
    git(&human, ["config", "--local", "workspace.role", "human"]);
    assert!(
        Command::new("git")
            .args(["clone", human.to_str().unwrap(), agent.to_str().unwrap()])
            .status()
            .unwrap()
            .success()
    );

    assert!(
        Command::new(human.join("scripts/workspace-git"))
            .args(["human-bootstrap", agent.to_str().unwrap()])
            .env(
                "WORKSPACE_CONFIRM",
                format!("BOOTSTRAP:{}", agent.display())
            )
            .status()
            .unwrap()
            .success()
    );
    assert_eq!(git_output(&agent, ["remote"]), "");
    assert_eq!(
        git_output(&agent, ["branch", "--show-current"]),
        "agent/codex"
    );

    fs::write(agent.join("agent-change.txt"), "agent work\n").unwrap();
    git(&agent, ["add", "agent-change.txt"]);
    git(
        &agent,
        [
            "-c",
            "user.name=Test Agent",
            "-c",
            "user.email=agent@example.invalid",
            "commit",
            "-m",
            "Agent work",
        ],
    );
    assert!(
        Command::new(agent.join("scripts/workspace-git"))
            .arg("agent-to-human")
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new(human.join("scripts/workspace-git"))
            .arg("human-from-agent")
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new(human.join("scripts/workspace-git"))
            .arg("human-agent-to-main")
            .status()
            .unwrap()
            .success()
    );
    assert_eq!(
        fs::read_to_string(human.join("agent-change.txt")).unwrap(),
        "agent work\n"
    );
}

fn git<const N: usize>(directory: &Path, arguments: [&str; N]) {
    assert!(
        Command::new("git")
            .args(arguments)
            .current_dir(directory)
            .status()
            .expect("run git")
            .success()
    );
}

fn git_output<const N: usize>(directory: &Path, arguments: [&str; N]) -> String {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(directory)
        .output()
        .expect("run git");
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}
