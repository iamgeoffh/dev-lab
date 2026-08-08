use std::{env, process::ExitCode};

fn main() -> ExitCode {
    match devl::run(env::args_os(), &env::current_dir().unwrap_or_default()) {
        Ok(output) => {
            if !output.is_empty() {
                println!("{output}");
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(error.exit_code())
        }
    }
}
