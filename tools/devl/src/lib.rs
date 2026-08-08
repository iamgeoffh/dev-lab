mod templates;

use std::{
    ffi::{OsStr, OsString},
    fmt, fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const HELP: &str = "Clean, contained development labs\n\nUsage:\n  devl init [PATH]\n  devl help\n\nCommands:\n  init    Create a new development-lab project (default PATH: current directory)\n  help    Print this help\n\nOptions:\n  -h, --help       Print help\n  -V, --version    Print version";

const INIT_HELP: &str = "Create a new development-lab project\n\nUsage:\n  devl init [PATH]\n\nArguments:\n  [PATH]    New or empty destination directory (default: current directory)\n\nThe command never overwrites files, initializes Git, installs tools, or starts containers.";

#[derive(Debug)]
pub struct Error {
    message: String,
    exit_code: u8,
}

impl Error {
    fn usage(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            exit_code: 2,
        }
    }

    fn operation(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            exit_code: 1,
        }
    }

    #[must_use]
    pub const fn exit_code(&self) -> u8 {
        self.exit_code
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Eq, PartialEq)]
enum Command {
    Help,
    Version,
    Init(PathBuf),
    InitHelp,
}

/// Parse command-line arguments and run the selected command.
///
/// # Errors
///
/// Returns an error for invalid arguments or when project initialization fails.
pub fn run<I, S>(args: I, current_dir: &Path) -> Result<String, Error>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    match parse(args)? {
        Command::Help => Ok(HELP.to_owned()),
        Command::Version => Ok(format!("devl {}", env!("CARGO_PKG_VERSION"))),
        Command::InitHelp => Ok(INIT_HELP.to_owned()),
        Command::Init(path) => {
            let destination = if path.is_absolute() {
                path
            } else {
                current_dir.join(path)
            };
            let project = init(&destination)?;
            Ok(format!(
                "Created development lab {project} at {}\n\nNext steps:\n  cd {}\n  review the generated executable configuration\n  git init\n  mise trust mise.toml\n  mise install --locked",
                destination.display(),
                destination.display()
            ))
        }
    }
}

fn parse<I, S>(args: I) -> Result<Command, Error>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let mut arguments = args.into_iter().map(Into::into);
    let _program = arguments.next();
    let Some(command) = arguments.next() else {
        return Ok(Command::Help);
    };

    if command == "help" || command == "-h" || command == "--help" {
        if arguments.next().is_some() {
            return Err(Error::usage("help does not accept arguments"));
        }
        return Ok(Command::Help);
    }
    if command == "-V" || command == "--version" {
        if arguments.next().is_some() {
            return Err(Error::usage("--version does not accept arguments"));
        }
        return Ok(Command::Version);
    }
    if command != "init" {
        return Err(Error::usage(format!(
            "unknown command {}; run `devl --help`",
            command.to_string_lossy()
        )));
    }

    let Some(path_or_option) = arguments.next() else {
        return Ok(Command::Init(PathBuf::from(".")));
    };
    if path_or_option == "-h" || path_or_option == "--help" {
        if arguments.next().is_some() {
            return Err(Error::usage("init --help does not accept arguments"));
        }
        return Ok(Command::InitHelp);
    }
    if path_or_option.to_string_lossy().starts_with('-') {
        return Err(Error::usage(format!(
            "unknown init option {}; run `devl init --help`",
            path_or_option.to_string_lossy()
        )));
    }
    if arguments.next().is_some() {
        return Err(Error::usage("init accepts at most one path"));
    }
    Ok(Command::Init(PathBuf::from(path_or_option)))
}

/// Create a development-lab project in a new or empty directory.
///
/// # Errors
///
/// Returns an error if the destination is invalid or nonempty, or if the
/// generated project cannot be staged and installed.
pub fn init(destination: &Path) -> Result<String, Error> {
    let name = destination
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| Error::operation("destination must have a UTF-8 directory name"))?;
    let project = normalize_project_name(name)?;

    let destination_existed = match fs::metadata(destination) {
        Ok(metadata) => {
            if !metadata.is_dir() {
                return Err(Error::operation(format!(
                    "destination is not a directory: {}",
                    destination.display()
                )));
            }
            let mut entries = fs::read_dir(destination)
                .map_err(|error| io_error("inspect destination", destination, &error))?;
            if entries
                .next()
                .transpose()
                .map_err(|error| io_error("inspect destination", destination, &error))?
                .is_some()
            {
                return Err(Error::operation(format!(
                    "destination is not empty; no files were changed: {}",
                    destination.display()
                )));
            }
            true
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => false,
        Err(error) => return Err(io_error("inspect destination", destination, &error)),
    };

    let parent = destination.parent().ok_or_else(|| {
        Error::operation(format!(
            "destination has no parent: {}",
            destination.display()
        ))
    })?;
    fs::create_dir_all(parent).map_err(|error| io_error("create parent", parent, &error))?;

    let staging = unique_staging_path(parent, &project);
    fs::create_dir(&staging)
        .map_err(|error| io_error("create staging directory", &staging, &error))?;

    let result = (|| {
        for template in templates::FILES {
            let target = staging.join(template.path);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| io_error("create directory", parent, &error))?;
            }
            let contents = template.contents.replace("{{PROJECT_NAME}}", &project);
            fs::write(&target, contents)
                .map_err(|error| io_error("write generated file", &target, &error))?;
            set_executable_if_needed(&target, template.executable)?;
        }

        if destination_existed {
            install_into_empty_destination(&staging, destination)
        } else {
            fs::rename(&staging, destination)
                .map_err(|error| io_error("install generated project", destination, &error))
        }
    })();

    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    result.map(|()| project)
}

fn normalize_project_name(name: &str) -> Result<String, Error> {
    let mut normalized = String::new();
    let mut separator_pending = false;

    for character in name.chars() {
        if character.is_ascii_alphanumeric() {
            if separator_pending && !normalized.is_empty() {
                normalized.push('-');
            }
            normalized.push(character.to_ascii_lowercase());
            separator_pending = false;
        } else if matches!(character, '-' | '_' | '.' | ' ') {
            separator_pending = !normalized.is_empty();
        } else {
            return Err(Error::operation(format!(
                "project directory name contains an unsupported character: {character:?}"
            )));
        }
    }

    if normalized.is_empty() {
        return Err(Error::operation(
            "project directory name must contain an ASCII letter or digit",
        ));
    }
    if normalized.len() > 63 {
        return Err(Error::operation(
            "normalized project name must be at most 63 characters",
        ));
    }
    Ok(normalized)
}

fn unique_staging_path(parent: &Path, project: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    parent.join(format!(".{project}.devl-{}-{nonce}", std::process::id()))
}

fn install_into_empty_destination(staging: &Path, destination: &Path) -> Result<(), Error> {
    let entries = fs::read_dir(staging)
        .map_err(|error| io_error("inspect staging directory", staging, &error))?;
    let mut installed: Vec<PathBuf> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| io_error("inspect staging entry", staging, &error))?;
        let target = destination.join(entry.file_name());
        if let Err(error) = fs::rename(entry.path(), &target) {
            for installed_path in installed.iter().rev() {
                let source = staging.join(
                    installed_path
                        .file_name()
                        .expect("installed top-level path has a file name"),
                );
                let _ = fs::rename(installed_path, source);
            }
            return Err(io_error("install generated project", &target, &error));
        }
        installed.push(target);
    }
    fs::remove_dir(staging).map_err(|error| io_error("remove staging directory", staging, &error))
}

#[cfg(unix)]
fn set_executable_if_needed(path: &Path, executable: bool) -> Result<(), Error> {
    use std::os::unix::fs::PermissionsExt;

    if executable {
        let mut permissions = fs::metadata(path)
            .map_err(|error| io_error("read generated file mode", path, &error))?
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions)
            .map_err(|error| io_error("set generated file mode", path, &error))?;
    }
    Ok(())
}

#[cfg(not(unix))]
fn set_executable_if_needed(_path: &Path, _executable: bool) -> Result<(), Error> {
    Ok(())
}

fn io_error(action: &str, path: &Path, error: &io::Error) -> Error {
    Error::operation(format!("failed to {action} {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default_init_path() {
        assert_eq!(
            parse(["devl", "init"]).expect("parse succeeds"),
            Command::Init(PathBuf::from("."))
        );
    }

    #[test]
    fn parses_explicit_init_path() {
        assert_eq!(
            parse(["devl", "init", "My App"]).expect("parse succeeds"),
            Command::Init(PathBuf::from("My App"))
        );
    }

    #[test]
    fn rejects_extra_arguments() {
        let error = parse(["devl", "init", "one", "two"]).expect_err("parse fails");
        assert_eq!(error.exit_code(), 2);
    }

    #[test]
    fn normalizes_project_names() {
        assert_eq!(normalize_project_name("My_App.v2").unwrap(), "my-app-v2");
        assert!(normalize_project_name("project/name").is_err());
        assert!(normalize_project_name("___").is_err());
    }
}
