use anyhow::{anyhow, Result};
use colored::*;
use std::path::PathBuf;

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
pub fn parse_and_validate_script_path(script_path: &PathBuf) -> Result<PathBuf> {
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
