use anyhow::{Result, anyhow};
use clap::Parser;
use colored::*;
use std::env::current_dir;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::{env, process};

use py_executer_lib::macros::{error_println, warning_println};
use py_executer_lib::{
    get_python_exec_path, get_uv_path, set_additional_env_var, validate_to_absolute_path,
};

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

/// Stream output from child process stdout and stderr
fn stream_output(mut child: process::Child) -> process::ExitCode {
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

fn validate_venv(venv_path: PathBuf) -> Result<PathBuf> {
    if !venv_path.exists() {
        Err(anyhow!("{} not exists", venv_path.display().to_string()))
    } else {
        let python_exec_paths = get_python_exec_path(&venv_path);
        if !python_exec_paths.exists() {
            Err(anyhow!(
                "Python executable {} not exists",
                python_exec_paths.display().to_string()
            ))
        } else {
            Ok(venv_path)
        }
    }
}

fn main() -> process::ExitCode {
    let args = Args::parse();
    let quiet = args.quiet;

    if !quiet {
        println!("------------------");
    }

    let mut files_to_clean: Vec<PathBuf> = Vec::new();

    // Get the absolute path of the script and the current runtime directory
    let script_path = validate_to_absolute_path(&args.script).unwrap_or_else(|err| {
        error_println!("Failed to get absolute path of script: {}", err);
        process::exit(1);
    });
    let runtime_path = current_dir().unwrap();

    // Get uv installation information
    let uv_path = get_uv_path().unwrap_or("".to_string());
    if !uv_path.is_empty() {
        // uv is installed
        if !quiet {
            println!("Using uv from: {}", uv_path.bold());
        }
    } else {
        // uv is not installed, will try native python
        if !quiet {
            warning_println!("Failed to get uv path, will not use it then");
        }
    }

    // Get python native as backup
    let python_native_path = if uv_path.is_empty() {
        #[cfg(not(target_os = "windows"))]
        let find_executable = "which";

        // For Windows
        #[cfg(target_os = "windows")]
        let find_executable = "where";

        let output = Command::new(find_executable)
            .arg("python3")
            .output()
            .unwrap();
        if output.status.success() {
            String::from_utf8(output.stdout)
                .unwrap_or("".to_string())
                .trim()
                .to_string()
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    };

    // If uv and native python are both empty, exit with error
    if python_native_path.is_empty() && uv_path.is_empty() {
        error_println!("Failed to get any python executable");
        process::exit(1);
    }

    // Validate provided venv
    // if not
    // try to find a possible venv under current directory
    // or create a new venv
    let venv = match validate_venv(args.venv) {
        Ok(venv) => venv,
        Err(e) => {
            warning_println!(
                "Failed to validate  provided venv: {}, looking for a possible one under current directory",
                e
            );
            let possible_venv_dir_names = ["venv", ".venv"];
            possible_venv_dir_names
                .iter()
                .map(|name| runtime_path.join(name))
                .find(|path| path.exists())
                .unwrap_or_else(|| {
                    if !quiet {
                        warning_println!("No venv found in current directory, will generate one");
                    }
                    let new_venv_path = runtime_path.join(".venv");
                    let _ = Command::new(if uv_path.is_empty() {
                        &python_native_path
                    } else {
                        &uv_path
                    })
                    .args(["venv", &new_venv_path.to_str().unwrap()])
                    .stdout(if quiet {
                        Stdio::null()
                    } else {
                        Stdio::inherit()
                    })
                    .stderr(if quiet {
                        Stdio::null()
                    } else {
                        Stdio::inherit()
                    })
                    .output()
                    .unwrap();
                    if args.clean {
                        files_to_clean.push(new_venv_path.clone());
                    }
                    new_venv_path
                })
        }
    };

    let python_exec_path = get_python_exec_path(&venv).to_str().unwrap().to_string();

    // Prepare dependencies
    let project_config_path = runtime_path.join("pyproject.toml");
    let requirements_path = runtime_path.join("requirements.txt");
    if !uv_path.is_empty() {
        if !project_config_path.exists() && !requirements_path.exists() {
            // both config are not exist
        } else {
            if project_config_path.exists() {
                let cmd = Command::new(&uv_path)
                    .args(["sync"])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output()
                    .unwrap();
                if !cmd.status.success() {
                    error_println!(
                        "Failed to install requirements: {:#?}",
                        String::from_utf8(cmd.stderr).unwrap()
                    );
                    process::exit(1);
                }
            }
            if requirements_path.exists() {
                let cmd = Command::new(&uv_path)
                    .args(["pip", "install", "-r", requirements_path.to_str().unwrap()])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output()
                    .unwrap();
                if !cmd.status.success() {
                    error_println!(
                        "Failed to install requirements: {:#?}",
                        String::from_utf8(cmd.stderr).unwrap()
                    );
                    process::exit(1);
                }
            }
        }
    } else {
        // if uv not installed
        if requirements_path.exists() {
            let cmd = Command::new(&python_exec_path)
                .args([
                    "-m",
                    "pip",
                    "install",
                    "-r",
                    requirements_path.to_str().unwrap(),
                ])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .unwrap();
            if !cmd.status.success() {
                error_println!(
                    "Failed to install requirements: {:#?}",
                    String::from_utf8(cmd.stderr).unwrap()
                );
                process::exit(1);
            }
        }
    }

    println!("Using venv: {}", venv.display().to_string().bold());
    // load dot env
    dotenv::from_path(args.env_file).ok();
    // load additional env from args
    let additional_env = set_additional_env_var(args.env, quiet);

    // Construct the command
    let py_cmd = Command::new(if !uv_path.is_empty() {
        &uv_path
    } else {
        &python_exec_path
    })
    .args(if !uv_path.is_empty() {
        Vec::from(["run", script_path.to_str().unwrap()])
    } else {
        Vec::from([script_path.to_str().unwrap()])
    })
    .args(args.py_arg)
    .envs(env::vars())
    .envs(additional_env)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .unwrap_or_else(|e| {
        error_println!("Failed to execute Python script: {}", e.to_string().bold());
        process::exit(1);
    });

    if !quiet {
        println!("------------------");
    }

    // Stream the output
    let result = stream_output(py_cmd);
    if args.clean {
        for path in files_to_clean.iter() {
            if path.is_dir() {
                if let Err(_) = std::fs::remove_dir_all(path) {
                    ();
                }
            } else {
                if let Err(_) = std::fs::remove_file(path) {
                    ();
                }
            }
        }
    }
    result
}
