pub mod macros;

use anyhow::anyhow;
use colored::*;
use std::collections::HashMap;
use std::process::Command;
use std::{env, path::PathBuf};

/// Appends the current directory to PYTHONPATH if it is valid.
fn append_pwd_to_pythonpath(current_dir: PathBuf) -> HashMap<String, String> {
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
            if !quiet {
                warning_println!(
                    "Warning: Ignoring malformed environment variable: {}",
                    env_var.bold()
                );
            }
        }
    }
    additional_env
}

/// Parse and validate a script path.
///
/// The function takes a `PathBuf` as argument and checks if the path exists.
/// If the path exists, it returns a tuple of `(PathBuf, PathBuf)`, where the first element
/// is the absolute path of the script and the second element is the parent directory of
/// the script.
///
/// # Errors
///
/// The function returns an `Err` if the path does not exist or if the parent directory
/// cannot be obtained.
pub fn validate_to_absolute_path(script_path: &PathBuf) -> anyhow::Result<PathBuf> {
    match script_path.canonicalize() {
        Ok(path) => {
            if !path.exists() {
                return Err(anyhow!("{} not exists", path.display().to_string().bold()));
            }
            Ok(path)
        }
        Err(err) => Err(anyhow!("Failed to get absolute path of script: {}", err)),
    }
}

pub fn get_uv_path() -> anyhow::Result<String> {
    // For Unix-like systems (Linux, macOS)
    #[cfg(not(target_os = "windows"))]
    let find_executable = "which";

    // For Windows
    #[cfg(target_os = "windows")]
    let find_executable = "where";

    let output = Command::new(find_executable).arg("uv").output()?;
    if output.status.success() {
        // found uv
        let path = String::from_utf8(output.stdout)?.trim().to_string();
        Ok(path)
    } else {
        // not found uv, hint to install it

        // for unix, run wget -qO- https://astral.sh/uv/install.sh | sh
        eprintln!("Please run the following command to install uv:");
        #[cfg(not(target_os = "windows"))]
        eprintln!("wget -qO- https://astral.sh/uv/install.sh | sh");

        // for windows, run powershell -ExecutionPolicy ByPass -c "irm https://astral.sh/uv/install.ps1 | iex"
        #[cfg(target_os = "windows")]
        eprintln!(
            "powershell -ExecutionPolicy ByPass -c \"irm https://astral.sh/uv/install.ps1 | iex\""
        );
        Err(anyhow!("uv not installed"))
    }
}

#[test]
fn test_validate_to_absolute_path() {
    let script_path = PathBuf::from("test.py");
    let result = validate_to_absolute_path(&script_path);
    assert!(result.is_ok());
    println!("Script path: {}", result.unwrap().display().to_string());

    let non_existent_path = PathBuf::from("");
    let result = validate_to_absolute_path(&non_existent_path);
    assert!(result.is_err());
    println!("Error: {}", result.unwrap_err());
}
