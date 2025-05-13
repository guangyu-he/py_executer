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

    /// Venv path, can be also pre-defined by PYTHON_VENV_PATH,
    /// if specified, it will be used directly without managed by uv
    /// clean mode is not an option and will be ignored
    /// requirements.txt will not be installed
    #[clap(short, long)]
    venv: Option<PathBuf>,

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

/// Handle argument parsing and validation

/// Setup environment variables, dotenv, pythonpath, and venv
fn setup_environment(args: &Args, script_parent_path: &PathBuf) -> (PathBuf, String) {
    // prepare dotenv path
    let dotenv_path = if args.env_file.exists() {
        &args.env_file
    } else {
        // try using absolute path
        let env_file_path = &args.env_file;
        match env_file_path.canonicalize() {
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
    let (venv_path, uv_path) = match venv(args.venv.clone(), &script_parent_path, args.quiet) {
        Ok(venv_path) => venv_path,
        Err(e) => {
            error_println!("Failed to get venv path with error: {}", e);
            process::exit(1);
        }
    };
    (venv_path, uv_path)
}

/// Construct the command to run the Python script
fn construct_command(
    uv_path: &String,
    venv_path: &PathBuf,
    script_parent_path: &PathBuf,
    script_path: &PathBuf,
    python_args: &[String],
    additional_env: &std::collections::HashMap<String, String>,
    has_custom_venv: bool,
) -> Command {
    let python_exec_path = get_python_exec_path(&venv_path);

    if !has_custom_venv && uv_path.is_empty() {
        panic!("No venv provided while uv path is empty");
    }

    if has_custom_venv {
        let mut command = Command::new(python_exec_path);
        command
            .arg(&script_parent_path.join(script_path))
            .args(python_args)
            .envs(env::vars())
            .envs(additional_env)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        command
    } else {
        let mut command = Command::new(uv_path);
        command
            .arg("run")
            .args(["--python", python_exec_path.as_str()])
            .arg(&script_parent_path.join(script_path))
            .args(python_args)
            .envs(env::vars())
            .envs(additional_env)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        command
    }
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

    let (script_path, script_parent_path) = match parse_and_validate_script_path(&args.script) {
        Ok(script_path) => script_path,
        Err(e) => {
            error_println!("{}", e);
            process::exit(1);
        }
    };

    let quiet = args.quiet;
    let has_custom_venv = args.venv.is_some();

    let clean = args.clean;
    let mut files_to_clean: Vec<PathBuf> = Vec::new();

    if clean && has_custom_venv {
        warning_println!("Clean mode is not activated when using custom venv");
    }

    if clean && !has_custom_venv {
        let original_venv = script_parent_path.join(".venv");
        if !original_venv.exists() {
            // means originally .venv does not exist
            files_to_clean.push(original_venv);
        }

        let pyproject_toml_path = script_parent_path.join("pyproject.toml");
        if !pyproject_toml_path.exists() {
            // means originally pyproject.toml does not exist
            files_to_clean.push(pyproject_toml_path);
        }

        let uv_lock_path = script_parent_path.join("uv.lock");
        if !uv_lock_path.exists() {
            // means originally uv.lock does not exist
            files_to_clean.push(uv_lock_path);
        }
    }

    let (venv_path, uv_path) = setup_environment(&args, &script_parent_path);
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
    let mut command = construct_command(
        &uv_path,
        &venv_path,
        &script_parent_path,
        &script_path,
        &python_args,
        &additional_env,
        has_custom_venv,
    );
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
