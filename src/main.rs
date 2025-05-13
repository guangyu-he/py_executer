use clap::Parser;
use colored::*;
use dotenv::from_path;
use py_executer_lib::macros::error_println;
use py_executer_lib::path::parse_and_validate_script_path;
use py_executer_lib::utils::{get_python_exec_path, set_additional_env_var};
use py_executer_lib::uv::venv;
use py_executer_lib::warning_println;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::{env, process};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Script path
    #[clap(index = 1)]
    script: PathBuf,

    /// If there is a valid venv provided, it will be used directly without managed by uv
    /// clean mode is not an option and will be ignored
    /// requirements.txt will not be installed
    #[clap(short, long, default_value = ".venv")]
    venv: PathBuf,

    /// Additional environment variables in the format KEY=VALUE (can be used multiple times)
    #[clap(short = 'E', long)]
    env: Vec<String>,

    /// .env file
    #[clap(short = 'e', long, default_value = ".env")]
    env_file: PathBuf,

    /// Suppress output
    #[clap(long, default_value_t = false)]
    quiet: bool,

    /// Clean mode
    /// if specified, it will clean the created uv .venv and configs
    /// if those files originally exist, they will not be deleted
    #[clap(long, default_value_t = false)]
    clean: bool,

    /// Python arguments, must be placed as the last argument
    #[arg(short = 'A', long = "py_arg", num_args = 1.., value_delimiter = ' ')]
    py_arg: Vec<String>,
}

fn setup_environment(args: &Args) -> (PathBuf, Vec<PathBuf>) {
    let dotenv_path = if args.env_file.exists() {
        &args.env_file
    } else {
        // try using absolute path
        match &args.env_file.canonicalize() {
            Ok(path) => &path.clone(),
            Err(err) => {
                warning_println!(
                    "{} not exists, {}",
                    args.env_file.display().to_string().bold(),
                    err
                );
                &PathBuf::from("")
            }
        }
    };
    from_path(dotenv_path).ok();

    // load venv
    let (venv_path, files_to_clean) = match venv(&args.venv, args.quiet, args.clean) {
        Ok(venv_path) => venv_path,
        Err(e) => {
            error_println!("Failed to get venv path with error: {}", e);
            process::exit(1);
        }
    };
    (venv_path, files_to_clean)
}

/// Construct the command to run the Python script
fn construct_command(
    venv_path: &PathBuf,
    script_path: &PathBuf,
    python_args: &[String],
    additional_env: &std::collections::HashMap<String, String>,
) -> Command {
    let mut command = Command::new(get_python_exec_path(venv_path));
    command
        .arg(&script_path)
        .args(python_args)
        .envs(env::vars())
        .envs(additional_env)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command
}

/// Stream output from child process stdout and stderr
fn stream_output(mut child: std::process::Child) -> process::ExitCode {
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

fn main() -> process::ExitCode {
    let args = Args::parse();

    let script_path = match parse_and_validate_script_path(&args.script) {
        Ok(script_path) => script_path,
        Err(e) => {
            error_println!("{}", e);
            process::exit(1);
        }
    };

    let quiet = args.quiet;
    let clean = args.clean;

    let (venv_path, files_to_clean) = setup_environment(&args);
    if !quiet {
        println!("Using venv: {}", venv_path.display().to_string().bold());
        println!(
            "Executing script: {}",
            &script_path.display().to_string().bold()
        );
    }
    let additional_env = set_additional_env_var(args.env.clone(), quiet);
    let python_args = args.py_arg.clone();
    if !quiet {
        for arg in &python_args {
            println!("Python arg: {}", arg.bold());
        }

        if clean && !files_to_clean.is_empty() {
            warning_println!(
                "These following files will be deleted because you activate clean mode"
            );
            for path in &files_to_clean {
                println!("{}", path.display().to_string().bold());
            }
        }

        println!("-------------------------------");
    }
    let mut command = construct_command(&venv_path, &script_path, &python_args, &additional_env);
    let child = command.spawn().unwrap_or_else(|e| {
        error_println!("Failed to execute Python script: {}", e.to_string().bold());
        process::exit(1);
    });
    let result = stream_output(child);
    if !files_to_clean.is_empty() {
        for path in files_to_clean.iter() {
            if path.is_dir() {
                if let Err(e) = std::fs::remove_dir_all(path) {
                    error_println!(
                        "Failed to delete {}: {}",
                        path.display().to_string().bold(),
                        e.to_string().bold()
                    );
                }
            } else {
                if let Err(e) = std::fs::remove_file(path) {
                    error_println!(
                        "Failed to delete {}: {}",
                        path.display().to_string().bold(),
                        e.to_string().bold()
                    );
                }
            }
        }
    }
    result
}
