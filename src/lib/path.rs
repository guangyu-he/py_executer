use crate::{get_python_exec_path, warning_println};
use anyhow::anyhow;
use std::path::PathBuf;
use std::process::{Command, Stdio};

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
