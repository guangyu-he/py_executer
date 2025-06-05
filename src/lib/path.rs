use crate::warning_println;
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
    runtime_path: PathBuf,
    uv_path: String,
    python_native_path: String,
    quiet: bool,
    clean: bool,
    files_to_clean: &mut Vec<PathBuf>,
) -> PathBuf {
    let possible_venv_dir_names = ["venv", ".venv"];
    possible_venv_dir_names
        .iter()
        .map(|name| runtime_path.join(name))
        .find(|path| path.exists())
        .unwrap_or_else(|| {
            prepare_venv(
                quiet,
                &runtime_path,
                &uv_path,
                &python_native_path,
                clean,
                files_to_clean,
            )
        })
}

fn prepare_venv(
    quiet: bool,
    runtime_path: &PathBuf,
    uv_path: &String,
    python_native_path: &String,
    clean: bool,
    files_to_clean: &mut Vec<PathBuf>,
) -> PathBuf {
    if !quiet {
        warning_println!(
            "No venv found in {}, will generate one",
            runtime_path.display()
        );
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
}
