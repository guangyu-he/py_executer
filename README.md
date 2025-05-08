# py_executer

A Rust-based command-line tool to execute Python scripts with automatic virtual environment and dependency management.
`py_executer` streamlines running Python code by handling environment setup, dependency installation, and environment
variables, making your workflow faster and more reliable.

## Features

- **Automatic Virtual Environment Management:** Detects or creates a Python virtual environment
  using [uv](https://github.com/astral-sh/uv).
- **Dependency Installation:** Installs dependencies from `requirements.txt` automatically.
- **.env File Support:** Loads environment variables from a `.env` file.
- **Custom Environment Variables:** Pass additional environment variables via CLI.
- **Flexible Python Arguments:** Pass extra arguments to the Python script.
- **Cross-platform:** Works on Unix-like systems and Windows.

## Installation

1. Install [Rust](https://www.rust-lang.org/tools/install)
   and [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html).
2. Clone this repository:
   ```sh
   git clone <your-repo-url>
   cd py_executer
   ```
3. Build the project:
   ```sh
   cargo build --release
   ```
4. (Optional) Install [uv](https://github.com/astral-sh/uv) if not already present. The tool will attempt to install it
   if missing.

## Usage

```sh
py_executer <SCRIPT_PATH> [OPTIONS]
```

### Arguments

- `<SCRIPT_PATH>`: Path to the Python script to execute.

### Options

- `-v`, `--venv <VENV_PATH>`: Specify a custom virtual environment path.
- `-E`, `--env <KEY=VALUE>`: Additional environment variables (can be used multiple times).
- `-e`, `--env-file <ENV_FILE>`: Path to a .env file (default: `.env` under the script directory).
- `--quiet`: Suppress output.
- `-A`, `--py-arg <ARG>`: Additional arguments to pass to the Python script.

### Example

```sh
py_executer my_script.py -E DEBUG=true -A --input -A data.txt
# this will be equivalent to:
# python3 my_script.py --input data.txt
```

## Project Structure

- `src/main.rs`: Main CLI logic and environment setup.
- `src/lib/`: Internal modules for utilities, macros, and uv integration.

## Contributing

Pull requests are welcome! For major changes, please open an issue first to discuss what you would like to change.

