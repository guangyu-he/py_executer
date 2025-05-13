use crate::warning_println;
use anyhow::{anyhow, Result};
use colored::*;
use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub fn get_uv_path() -> Result<String> {
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

fn prepare_uv_project(project_path: &PathBuf, quiet: bool) -> Result<()> {
    let uv_path = get_uv_path()?;
    let output = Command::new(&uv_path)
        .args(["init", "--bare", project_path.to_str().unwrap()])
        .stdout(if quiet {
            Stdio::null()
        } else {
            Stdio::inherit()
        })
        .stderr(Stdio::piped())
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        if let Ok(stderr) = String::from_utf8(output.stderr) {
            if stderr.contains("Project is already initialized") {
                return Ok(());
            }
            eprintln!("{}", stderr);
        }
        Err(anyhow!("Failed to prepare uv project"))
    }
}

pub fn prepare_venv(venv_path: &PathBuf, quiet: bool) -> Result<()> {
    let uv_path = get_uv_path()?;
    let output = Command::new(&uv_path)
        .args(["venv", venv_path.to_str().unwrap()])
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
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(anyhow!("Failed to prepare venv"))
    }
}

fn install_requirements(
    venv_path: &PathBuf,
    requirements_path: &PathBuf,
    quiet: bool,
) -> Result<()> {
    if !requirements_path.exists() {
        if !quiet {
            warning_println!(
                "{} not exists, skipping",
                requirements_path.display().to_string().bold()
            );
        }
        return Ok(());
    }

    let uv_path = get_uv_path()?;
    let output = Command::new(&uv_path)
        .args([
            "add",
            "--quiet",
            "--directory",
            venv_path.to_str().unwrap(),
            "-r",
            venv_path.join(requirements_path).to_str().unwrap(),
        ])
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
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(anyhow!("Failed to install requirements"))
    }
}

fn use_uv_venv(
    quiet: bool,
    clean: bool,
    files_to_clean: &mut Vec<PathBuf>,
) -> Result<(PathBuf, Vec<PathBuf>)> {
    let current_dir = env::current_dir()?;

    prepare_uv_project(&current_dir, quiet)?;
    let venv_path = current_dir.join(".venv");
    if !venv_path.exists() {
        prepare_venv(&venv_path, quiet)?;
    }
    install_requirements(&venv_path, &current_dir.join("requirements.txt"), quiet)?;

    if clean {
        let original_venv = current_dir.join(".venv");
        if !original_venv.exists() {
            // means originally .venv does not exist
            files_to_clean.push(original_venv);
        }

        let pyproject_toml_path = current_dir.join("pyproject.toml");
        if !pyproject_toml_path.exists() {
            // means originally pyproject.toml does not exist
            files_to_clean.push(pyproject_toml_path);
        }

        let uv_lock_path = current_dir.join("uv.lock");
        if !uv_lock_path.exists() {
            // means originally uv.lock does not exist
            files_to_clean.push(uv_lock_path);
        }
    }

    Ok((venv_path, files_to_clean.to_vec()))
}

pub fn venv(
    venv_path_from_arg: &PathBuf,
    quiet: bool,
    clean: bool,
) -> Result<(PathBuf, Vec<PathBuf>)> {
    let current_dir = env::current_dir()?;
    let mut files_to_clean: Vec<PathBuf> = Vec::new();

    if !venv_path_from_arg.exists() {
        if venv_path_from_arg.to_str().unwrap_or("") == ".venv" {
            // means using default .venv
            // then try alternative venv
            if !quiet {
                warning_println!("No .venv found in current directory, trying venv");
            }
            let venv_path_alternate = current_dir.join("venv");
            if venv_path_alternate.exists() {
                if clean && !quiet {
                    warning_println!("Clean mode is not activated when using existing venv");
                }
                if current_dir.join("requirements.txt").exists() && !quiet {
                    warning_println!(
                        "You are about to use an existing venv, it is not possible for now to install requirements.txt on it"
                    );
                }
                return Ok((venv_path_alternate, files_to_clean));
            }
        }
        if !quiet {
            warning_println!("No venv found in current directory, trying uv");
        }
        let (venv_path, files_to_clean) = use_uv_venv(quiet, clean, &mut files_to_clean)?;
        return Ok((venv_path, files_to_clean));
    }

    // provided venv exists (either default .venv or custom venv)
    let venv_path_from_arg_absolute = match venv_path_from_arg.canonicalize() {
        Ok(path) => path,
        Err(err) => {
            return Err(anyhow!(
                "Can not get absolute path of {} , {}",
                venv_path_from_arg.display().to_string().bold(),
                err
            ));
        }
    };

    if clean && !quiet {
        warning_println!("Clean mode is not activated when using existing venv");
    }
    if current_dir.join("requirements.txt").exists() {
        if !quiet {
            warning_println!(
                "You are about to use an existing venv, it is not possible for now to install requirements.txt on it"
            );
        }
    }
    Ok((venv_path_from_arg_absolute, files_to_clean))
}
