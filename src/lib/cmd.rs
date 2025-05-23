use colored::Colorize;
use std::io::{BufRead, BufReader};
use std::process;

/// Stream output from child process stdout and stderr
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
