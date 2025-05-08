use clap::Parser;
use colored::*;
use dotenv::from_path;
use py_executer_lib::macros::error_println;
use py_executer_lib::utils::{append_pwd_to_pythonpath, set_additional_env_var};
use py_executer_lib::uv::{get_uv_path, venv};
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

    /// Venv path, can be also pre-defined by PYTHON_VENV_PATH,
    /// the path need to contain python interpreter
    /// default is .venv/bin or .venv/Scripts (windows)
    #[clap(short, long)]
    venv: Option<PathBuf>,

    /// Additional environment variables in the format KEY=VALUE (can be used multiple times)
    #[clap(short = 'E', long)]
    env: Vec<String>,

    /// .env file
    #[clap(short = 'e', long, default_value = ".env")]
    env_file: PathBuf,

    #[clap(long, default_value_t = false)]
    quiet: bool,

    /// Python arguments
    #[arg(short = 'A', long = "py_arg", num_args = 1.., value_delimiter = ' ')]
    py_arg: Vec<String>,
}

/// Handle argument parsing and validation
fn parse_and_validate_args() -> (Args, PathBuf) {
    let args = Args::parse();
    let script_path = args.script.clone();
    // check script path
    if !script_path.exists() {
        error_println!("{} not exists", script_path.display().to_string().bold());
        process::exit(1);
    }
    let script_path = match script_path.canonicalize() {
        Ok(script_path) => script_path,
        Err(err) => {
            error_println!(
                "Failed to get absolute path of {}: {}",
                script_path.display().to_string().bold(),
                err
            );
            process::exit(1);
        }
    };
    (args, script_path)
}

/// Setup environment variables, dotenv, pythonpath, and venv
fn setup_environment(args: &Args, script_path: &PathBuf) -> (PathBuf, PathBuf, String) {
    let script_parent_path = match script_path.parent() {
        Some(script_parent_path) => script_parent_path.to_path_buf(),
        None => {
            error_println!("Failed to get script parent directory");
            process::exit(1);
        }
    };
    if !append_pwd_to_pythonpath(&script_parent_path) {
        process::exit(1);
    }
    // prepare dotenv path
    let dotenv_path = if args.env_file.to_str().unwrap_or("") == ".env" {
        script_parent_path.join(&args.env_file)
    } else {
        if args.env_file.exists() {
            args.env_file.clone()
        } else {
            error_println!("{} not exists", args.env_file.display().to_string().bold());
            PathBuf::from("")
        }
    };
    from_path(dotenv_path).ok();
    // load venv
    let venv_path = match venv(args.venv.clone(), &script_parent_path, args.quiet) {
        Ok(venv_path) => venv_path,
        Err(e) => {
            error_println!("Failed to get venv path with error: {}", e);
            process::exit(1);
        }
    };
    let uv_path = match get_uv_path() {
        Ok(uv_path) => uv_path,
        Err(e) => {
            error_println!("Failed to get uv path with error: {}", e);
            process::exit(1);
        }
    };
    (script_parent_path, venv_path, uv_path)
}

/// Construct the command to run the Python script
fn construct_command(
    uv_path: &String,
    venv_path: &PathBuf,
    script_parent_path: &PathBuf,
    script_path: &PathBuf,
    python_args: &[String],
    additional_env: &std::collections::HashMap<String, String>,
) -> Command {
    let mut command = Command::new(uv_path);
    command
        .arg("run")
        .args(["--directory", &venv_path.to_string_lossy().to_string()])
        .arg(&script_parent_path.join(script_path))
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
    let (args, script_path) = parse_and_validate_args();
    let quiet = args.quiet;
    let (script_parent_path, venv_path, uv_path) = setup_environment(&args, &script_path);
    if !quiet {
        println!("Using venv: {}", venv_path.display().to_string().bold());
        println!(
            "Executing script: {}",
            script_parent_path
                .join(&script_path)
                .display()
                .to_string()
                .bold()
        );
    }
    let additional_env = set_additional_env_var(args.env.clone(), quiet);
    let python_args = args.py_arg.clone();
    if !quiet {
        for arg in &python_args {
            println!("Python arg: {}", arg.bold());
        }
        println!("-------------------------------");
    }
    let mut command = construct_command(
        &uv_path,
        &venv_path,
        &script_parent_path,
        &script_path,
        &python_args,
        &additional_env,
    );
    let child = command.spawn().unwrap_or_else(|e| {
        error_println!("Failed to execute Python script: {}", e.to_string().bold());
        process::exit(1);
    });
    stream_output(child)
}
