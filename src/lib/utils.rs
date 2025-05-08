use std::collections::HashMap;
use std::{env, path::PathBuf};
use colored::*;
use crate::macros::{error_println, warning_println};

/// Appends the current directory to PYTHONPATH if it is valid.
pub fn append_pwd_to_pythonpath(current_dir: &PathBuf) -> bool {
    if !current_dir.exists() {
        error_println!(
            "Current directory not valid: {}",
            current_dir.display().to_string().bold()
        );
        false
    } else {
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

/// Processes additional environment variables from CLI arguments.
pub fn set_additional_env_var(
    additional_env_from_args: Vec<String>,
    quiet: bool,
) -> HashMap<String, String> {
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
