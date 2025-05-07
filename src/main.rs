use clap::Parser;
use colored::*;
use dotenv::from_path;
use py_executer_lib::macros::{error_println, warning_println};
use py_executer_lib::uv::{get_uv_path, venv};
use std::collections::HashMap;
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
    #[clap(short = 'A', long)]
    py_arg: Vec<String>,
}

fn append_pwd_to_pythonpath(current_dir: &PathBuf) -> bool {
    if !current_dir.exists() {
        error_println!(
            "Current directory not valid: {}",
            current_dir.display().to_string().bold()
        );
        false
    } else {
        // if the pwd is valid, append it to PYTHONPATH
        let mut path = env::var("PYTHONPATH").unwrap_or_default();
        if !path.contains(&current_dir.to_string_lossy().to_string()) {
            if !path.is_empty() {
                path.push(':');
            }
            path.push_str(current_dir.to_string_lossy().to_string().as_str());
            unsafe {
                env::set_var("PYTHONPATH", path);
            }
        }
        true
    }
}

fn set_additional_env_var(
    additional_env_from_args: Vec<String>,
    quiet: bool,
) -> HashMap<String, String> {
    // Process additional environment variables
    let mut additional_env = HashMap::new();
    for env_var in additional_env_from_args {
        if let Some(pos) = env_var.find('=') {
            let key = env_var[..pos].to_string();
            let value = env_var[pos + 1..].to_string();
            additional_env.insert(key.clone(), value.clone());
            if !quiet {
                println!("Setting env: {} = {}", key.bold(), value);
            }
        } else {
            warning_println!(
                "Warning: Ignoring malformed environment variable: {}",
                env_var.bold()
            );
        }
    }
    additional_env
}

fn main() -> process::ExitCode {
    let args = Args::parse();
    let quiet = args.quiet;
    let script_path = args.script;

    // check script path
    if !script_path.exists() {
        error_println!("{} not exists", script_path.display().to_string().bold());
        return process::ExitCode::FAILURE;
    }

    let script_path = match script_path.canonicalize() {
        Ok(script_path) => script_path,
        Err(err) => {
            error_println!(
                "Failed to get absolute path of {}: {}",
                script_path.display().to_string().bold(),
                err
            );
            return process::ExitCode::FAILURE;
        }
    };

    let script_parent_path = match script_path.parent() {
        Some(script_parent_path) => script_parent_path.to_path_buf(),
        None => {
            error_println!("Failed to get script parent directory");
            return process::ExitCode::FAILURE;
        }
    };
    if !append_pwd_to_pythonpath(&script_parent_path) {
        return process::ExitCode::FAILURE;
    }

    // prepare dotenv path
    let dotenv_path = if args.env_file.to_str().unwrap_or("") == ".env" {
        // default .env file name
        script_parent_path.join(args.env_file)
    } else {
        if args.env_file.exists() {
            args.env_file
        } else {
            error_println!("{} not exists", args.env_file.display().to_string().bold());
            PathBuf::from("")
        }
    };
    // load dotenv
    from_path(dotenv_path).ok();

    // load venv
    let venv_path = match venv(args.venv, &script_parent_path, args.quiet) {
        Ok(venv_path) => venv_path,
        Err(e) => {
            error_println!("Failed to get venv path with error: {}", e);
            return process::ExitCode::FAILURE;
        }
    };

    let uv_path = match get_uv_path() {
        Ok(uv_path) => uv_path,
        Err(e) => {
            error_println!("Failed to get uv path with error: {}", e);
            return process::ExitCode::FAILURE;
        }
    };

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

    // Process additional environment variables
    let additional_env = set_additional_env_var(args.env, quiet);

    // Display python args
    let python_args = args.py_arg;
    if !quiet {
        for arg in &python_args {
            println!("Python arg: {}", arg.bold());
        }
    }

    // Finish setup
    if !quiet {
        println!("-------------------------------");
    }

    // execute python script with streaming output
    let mut command = Command::new(uv_path);
    command
        .arg("run")
        .args(["--directory", &venv_path.to_string_lossy().to_string()])
        .arg(&script_parent_path.join(&script_path))
        .args(python_args)
        .envs(env::vars())
        .envs(additional_env)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command.spawn().unwrap_or_else(|e| {
        error_println!("Failed to execute Python script: {}", e.to_string().bold());
        process::exit(1);
    });

    // Stream stdout in real-time
    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let stdout_reader = BufReader::new(stdout);
    let stdout_lines = stdout_reader.lines();

    // Stream stderr in real-time
    let stderr = child.stderr.take().expect("Failed to capture stderr");
    let stderr_reader = BufReader::new(stderr);
    let stderr_lines = stderr_reader.lines();

    // Create a thread to handle stdout
    let stdout_handle = std::thread::spawn(move || {
        for line in stdout_lines {
            if let Ok(line) = line {
                println!("{}", line);
            }
        }
    });

    // Create a thread to handle stderr
    let stderr_handle = std::thread::spawn(move || {
        for line in stderr_lines {
            if let Ok(line) = line {
                eprintln!("{}", line.red());
            }
        }
    });

    // Wait for output threads to finish
    stdout_handle.join().unwrap();
    stderr_handle.join().unwrap();

    // Wait for the child process to finish and get exit status
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
