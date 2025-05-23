use colored::Colorize;
use std::io::{BufRead, BufReader};
use std::process;

/// Stream the output of a child process to stdout and stderr,
/// coloring any lines from stderr red.
///
/// The function takes a mutable reference to a `Child` process and
/// returns an `ExitCode` indicating the exit status of the process.
///
/// The function will block until the child process has finished
/// executing.
///
/// # Errors
///
/// If there is an error waiting for the child process to finish,
/// an error message will be printed to stderr and the process will
/// exit with a status code of 1.
pub fn stream_output(mut child: process::Child) -> process::ExitCode {
    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let stdout_reader = BufReader::new(stdout);
    let stdout_lines = stdout_reader.lines();
    let stderr = child.stderr.take().expect("Failed to capture stderr");
    let stderr_reader = BufReader::new(stderr);
    let stderr_lines = stderr_reader.lines();
    let stdout_handle = std::thread::spawn(move || {
        for line in stdout_lines {
            if let Ok(line) = line {
                println!("{}", line);
            }
        }
    });
    let stderr_handle = std::thread::spawn(move || {
        for line in stderr_lines {
            if let Ok(line) = line {
                eprintln!("{}", line.red());
            }
        }
    });
    stdout_handle.join().unwrap();
    stderr_handle.join().unwrap();
    match child.wait() {
        Ok(status) => {
            if status.success() {
                process::ExitCode::SUCCESS
            } else {
                process::ExitCode::FAILURE
            }
        }
        Err(e) => {
            eprintln!("Failed to wait for Python process: {}", e);
            process::ExitCode::FAILURE
        }
    }
}
