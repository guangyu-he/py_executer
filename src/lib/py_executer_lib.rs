pub mod cmd;
pub mod macros;
pub mod path;

use anyhow::anyhow;
use colored::*;
use std::collections::HashMap;
use std::process::Command;
use std::{env, path::PathBuf};

/// Append the current working directory to the `PYTHONPATH` environment variable.
///
/// The function takes a `PathBuf` as its argument, which represents the current working directory.
/// It returns a `HashMap` where the key is the name of the environment variable and the value is
/// the value of the environment variable.
fn append_pwd_to_pythonpath(runtime_path: &PathBuf) -> HashMap<String, String> {
    let mut path = env::var("PYTHONPATH").unwrap_or_default();
    if !path.contains(&runtime_path.to_string_lossy().to_string()) {
        if !path.is_empty() {
            path.push(':');
        }
        path.push_str(runtime_path.to_string_lossy().to_string().as_str());
    }
    HashMap::from([("PYTHONPATH".to_string(), path)])
}

/// Set additional environment variables from command line arguments.
///
/// The function takes a list of strings as its first argument, where each string is a key-value pair
/// separated by an '=' character. The function will parse each string and add the key-value pair to
/// the `HashMap` returned by this function.
///
/// If a key-value pair is malformed, the function will print a warning message and ignore the pair.
///
/// The function also adds the current working directory to the `PYTHONPATH` environment variable.
pub fn set_additional_env_var(
    additional_env_from_args: Vec<String>,
    runtime_path: &PathBuf,
    quiet: bool,
) -> HashMap<String, String> {
    let mut additional_env = HashMap::new();

    //add current dir to PYTHONPATH
    additional_env.extend(append_pwd_to_pythonpath(runtime_path));

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

/// Find the path of the `uv` command.
///
/// The function returns `Ok<String>` if `uv` is found, where the string is the path of
/// the `uv` command. If `uv` is not found, the function prints a hint to install it
/// and returns an `Err`.
///
/// The function uses `which` or `where` command to find the path of `uv`. If the command
/// is not successful, it means `uv` is not installed, so the function prints a hint to
/// install it and returns an `Err`.
///
/// # Errors
///
/// The function returns an `Err` if `uv` is not installed.
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

/// Returns the path of the Python executable within the given virtual environment.
///
/// For Unix-like systems (Linux, macOS), the Python executable is located in the `bin` directory.
///
/// For Windows, the Python executable is located in the `Scripts` directory, and has the `.exe` extension.
pub fn get_python_exec_path(venv_path: &PathBuf) -> PathBuf {
    PathBuf::from(if cfg!(target_os = "windows") {
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
    })
}
