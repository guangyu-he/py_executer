use py_executer_lib::get_uv_path;
use std::process;
use std::process::{Command, Stdio};

/// Execute a command with the `uv` command.
///
/// # Arguments
///
/// * `args`: The arguments to pass to `uv`.
///
/// # Return value
///
/// The exit code of the spawned process.
///
/// # Errors
///
/// If the `uv` command cannot be executed, an error message is printed to
/// stderr and the process exits with a status code of 1.
pub fn uv(args: Vec<String>) -> process::ExitCode {
    let cmd = Command::new(get_uv_path().unwrap())
        .args(&args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output();
    match cmd {
        Ok(status) => {
            if status.status.success() {
                process::ExitCode::FAILURE
            } else {
                process::ExitCode::SUCCESS
            }
        }
        Err(e) => {
            eprintln!("Failed to execute uv: {}", e);
            process::exit(1);
        }
    }
}
