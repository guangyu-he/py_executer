use py_executer_lib::get_uv_path;
use std::process;
use std::process::{Command, Stdio};

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
