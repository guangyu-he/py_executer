use crate::{get_python_exec_path, warning_println};
use anyhow::anyhow;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Finds the native Python executable path.
///
/// If `uv_path` is empty, it uses `which` or `where` command to find the native Python executable.
/// If the command is successful, it returns the path of the Python executable.
/// If the command is not successful, it returns an empty string.
///
/// If `uv_path` is not empty, it returns an empty string.
///
/// # Platform-specific
///
/// On Unix-like systems, it uses `which` command.
///
/// On Windows, it uses `where` command.
pub fn get_python_native_path(uv_path: &String) -> String {
    if uv_path.is_empty() {
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
    }
}

/// Validates a venv path.
///
/// # Errors
///
/// The function returns an `Err` if the venv path does not exist or if the Python executable
/// under the venv path does not exist.
fn validate_venv(venv_path: PathBuf) -> anyhow::Result<PathBuf> {
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

/// Finds a virtual environment path.
///
/// # Errors
///
/// The function returns an `Err` if the provided venv path does not exist or if the Python executable
/// under the venv path does not exist.
///
/// # Platform-specific
///
/// On Unix-like systems, it uses `which` command.
///
/// On Windows, it uses `where` command.
///
/// # Arguments
///
/// * `venv`: The venv path provided by the user.
/// * `runtime_path`: The runtime path of the current directory.
/// * `uv_path`: The path of the uv executable.
/// * `python_native_path`: The path of the native Python executable.
/// * `quiet`: If `true`, suppresses warnings and errors.
/// * `clean`: If `true`, will clean the created uv-managed .venv and config files after execution.
/// * `files_to_clean`: A vector of paths to clean.
///
/// # Returns
///
/// The path of the found virtual environment.
pub fn get_venv_path(
    venv: PathBuf,
    runtime_path: PathBuf,
    uv_path: String,
    python_native_path: String,
    quiet: bool,
    clean: bool,
    files_to_clean: &mut Vec<PathBuf>,
) -> PathBuf {
    match validate_venv(venv) {
        Ok(venv) => venv,
        Err(e) => {
            if !quiet {
                warning_println!(
                    "Failed to validate  provided venv: {}, looking for a possible one under current directory",
                    e
                );
            }
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
                    if clean {
                        files_to_clean.push(new_venv_path.clone());
                    }
                    new_venv_path
                })
        }
    }
}
