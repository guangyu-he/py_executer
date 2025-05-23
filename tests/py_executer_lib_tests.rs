use py_executer_lib::validate_to_absolute_path;
use std::path::PathBuf;

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
