use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::{env, process};

use colored::Colorize;

use py_executer_lib::path::{get_python_native_path, get_venv_path};
use py_executer_lib::{
    error_println, get_python_exec_path, get_uv_path, set_additional_env_var,
    validate_to_absolute_path, warning_println,
};

pub fn python(
    script: PathBuf,
    project: PathBuf,
    env: Vec<String>,
    env_file: Option<PathBuf>,
    quiet: bool,
    clean: bool,
    py_args: Vec<String>,
) -> process::ExitCode {
    if !quiet {
        println!("------------------");
    }

    let mut files_to_clean: Vec<PathBuf> = Vec::new();

    // Get the absolute path of the script and the current runtime directory
    let script_path = validate_to_absolute_path(&script).unwrap_or_else(|err| {
        error_println!("Failed to get absolute path of script: {}", err);
        process::exit(1);
    });
    let runtime_path = project.canonicalize().unwrap_or_else(|err| {
        error_println!("Failed to get absolute path of project: {}", err);
        process::exit(1);
    });

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
    let python_native_path = get_python_native_path(&uv_path);

    // If uv and native python are both empty, exit with error
    if python_native_path.is_empty() && uv_path.is_empty() {
        error_println!("Failed to get any python executable");
        process::exit(1);
    }

    // Validate provided venv
    // if not
    // try to find a possible venv under current directory
    // or create a new venv
    let venv = get_venv_path(
        runtime_path.clone(),
        uv_path.clone(),
        python_native_path.clone(),
        quiet,
        clean,
        &mut files_to_clean,
    );

    let python_exec_path = get_python_exec_path(&venv).to_str().unwrap().to_string();

    // Prepare dependencies
    let project_config_path = runtime_path.join("pyproject.toml");
    let requirements_path = runtime_path.join("requirements.txt");
    if !uv_path.is_empty() {
        if !project_config_path.exists() && !requirements_path.exists() {
            // both config are not exist
            warning_println!(
                "No pyproject.toml or requirements.txt found, will not prepare dependencies"
            );
        } else {
            if project_config_path.exists() {
                let cmd = Command::new(&uv_path)
                    .args(["sync", "--project", runtime_path.to_str().unwrap()])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output()
                    .unwrap();
                if !cmd.status.success() {
                    error_println!(
                        "Failed to sync uv project: {:#?}",
                        String::from_utf8(cmd.stderr).unwrap()
                    );
                    process::exit(1);
                }
            }
            if requirements_path.exists() {
                let cmd = Command::new(&uv_path)
                    .args([
                        "--directory",
                        runtime_path.to_str().unwrap(),
                        "pip",
                        "install",
                        "-r",
                        "requirements.txt",
                    ])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output()
                    .unwrap();
                if !cmd.status.success() {
                    error_println!(
                        "Failed to install pip requirements: {:#?}",
                        String::from_utf8(cmd.stderr).unwrap()
                    );
                    process::exit(1);
                }
            }
        }
    } else {
        // if uv not installed
        // TODO! if there is a uv generated venv, there will be no pip module in it
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

    if !quiet {
        println!("Using venv: {}", venv.display().to_string().bold());
    }

    // load dot env
    match env_file {
        None => {
            // if dot env not provided, but runtime path provided
            let env_file = runtime_path.join(".env");
            if env_file.exists() {
                dotenv::from_path(env_file).ok();
            }
        }
        Some(env_file) => {
            // if dot env provided, use it
            if env_file.exists() {
                dotenv::from_path(env_file).ok();
            } else {
                // provided dot env does not exist
                warning_println!("Provided env file does not exist, will not use it");
            }
        }
    };

    // load additional env from args
    let additional_env = set_additional_env_var(env, &runtime_path, quiet);

    if !quiet {
        println!("------------------");
    }

    // Construct the command
    let py_cmd = Command::new(if !uv_path.is_empty() {
        &uv_path
    } else {
        &python_exec_path
    })
    .args(if !uv_path.is_empty() {
        Vec::from([
            "run",
            "--project",
            runtime_path.to_str().unwrap(),
            script_path.to_str().unwrap(),
        ])
    } else {
        Vec::from([script_path.to_str().unwrap()])
    })
    .args(py_args)
    .envs(env::vars())
    .envs(additional_env)
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .spawn()
    .unwrap_or_else(|e| {
        error_println!("Failed to execute Python script: {}", e.to_string().bold());
        process::exit(1);
    })
    .wait();

    if clean {
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

    match py_cmd {
        Ok(status) => {
            if status.success() {
                process::ExitCode::SUCCESS
            } else {
                process::ExitCode::FAILURE
            }
        }
        Err(e) => {
            error_println!("Failed to execute Python script: {}", e.to_string().bold());
            process::ExitCode::FAILURE
        }
    }
}
