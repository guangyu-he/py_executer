use crate::warning_println;
use anyhow::{Result, anyhow};
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

pub fn venv(
    venv_path_from_arg: Option<PathBuf>,
    script_parent_path: &PathBuf,
    quiet: bool,
) -> Result<PathBuf> {
    let mut venv_path = PathBuf::new();

    if let Some(venv_path_from_arg) = venv_path_from_arg {
        if venv_path_from_arg.exists() {
            // if venv_path input from arg is exist, return it as a string
            venv_path = venv_path_from_arg;
        } else {
            // if venv_path input from arg is not exist, continue
            if !quiet {
                warning_println!(
                    "Venv provided {} does not exist. Will use uv to manage venv",
                    venv_path_from_arg.display()
                );
            }
        }
    }

    // try using PYTHON_VENV_PATH from env
    let env_venv_path = env::var("PYTHON_VENV_PATH").unwrap_or("".to_string());
    if !env_venv_path.is_empty() {
        // if it is not empty, wrap it as a PathBuf and check if it exists
        let env_venv_pathbuf = PathBuf::from(env_venv_path);
        if env_venv_pathbuf.exists() {
            // if it exists, return it as a string
            venv_path = env_venv_pathbuf;
        } else {
            // if it does not exist, continue and using default venv path
            if !quiet {
                warning_println!(
                    "Venv provided from PYTHON_VENV_PATH {} does not exist. Will use uv to manage venv",
                    env_venv_pathbuf.display()
                );
            }
        }
    }

    if !venv_path.exists() {
        prepare_uv_project(&script_parent_path, quiet)?;
        venv_path = script_parent_path.join(".venv");
        if !venv_path.exists() {
            prepare_venv(&venv_path, quiet)?;
        }
        install_requirements(
            &venv_path,
            &script_parent_path.join("requirements.txt"),
            quiet,
        )?;
    } else {
        prepare_uv_project(&venv_path.parent().unwrap().to_path_buf(), quiet)?;
        install_requirements(
            &venv_path,
            &script_parent_path.join("requirements.txt"),
            quiet,
        )?;
    }

    Ok(venv_path)
}
