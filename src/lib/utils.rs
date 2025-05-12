use crate::macros::{error_println, warning_println};
use colored::*;
use std::collections::HashMap;
use std::{env, path::PathBuf};

/// Appends the current directory to PYTHONPATH if it is valid.
pub fn append_pwd_to_pythonpath(current_dir: PathBuf) -> HashMap<String, String> {
    if !current_dir.exists() {
        error_println!(
            "Current directory not valid: {}",
            current_dir.display().to_string().bold()
        );
        HashMap::new()
    } else {
        let mut path = env::var("PYTHONPATH").unwrap_or_default();
        if !path.contains(&current_dir.to_string_lossy().to_string()) {
            if !path.is_empty() {
                path.push(':');
            }
            path.push_str(current_dir.to_string_lossy().to_string().as_str());
            return HashMap::from([("PYTHONPATH".to_string(), path)]);
        }
        HashMap::new()
    }
}

/// Processes additional environment variables from CLI arguments.
pub fn set_additional_env_var(
    additional_env_from_args: Vec<String>,
    quiet: bool,
) -> HashMap<String, String> {
    let mut additional_env = HashMap::new();

    //add current dir to PYTHONPATH
    let current_dir = env::current_dir().unwrap();
    additional_env.extend(append_pwd_to_pythonpath(current_dir));

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

pub fn get_python_exec_path(venv_path: &PathBuf) -> String {
    if cfg!(target_os = "windows") {
        venv_path
            .join("Scripts")
            .join("python.exe")
            .to_string_lossy()
            .to_string()
    } else {
        venv_path
            .join("bin")
            .join("python")
            .to_string_lossy()
            .to_string()
    }
}
