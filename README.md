# py_executer

A Rust-based command-line tool to execute Python scripts with automatic virtual environment and dependency management.
`py_executer` streamlines running Python code by handling environment setup, dependency installation, and environment
variables, making your workflow faster and more reliable.

## Features

- **Automatic Virtual Environment Management:** Manage a Python virtual environment
  using [uv](https://github.com/astral-sh/uv).
- **Dependency Installation:** Installs dependencies from `requirements.txt` automatically.
- **.env File Support:** Loads environment variables from a `.env` file or from CLI.
- **Setup `PYTHONVENV`:** Automatically set python path based on the path where the CLI is executed.
- **Custom Environment Variables:** Pass additional environment variables via CLI.
- **Flexible Python Arguments:** Pass extra arguments to the Python script.
- **Clean Mode:** Clean the created uv-managed .venv and config files after execution, sensorless to execute a python
  script.
- **Cross-platform:** Works on Unix-like systems and Windows.

## Installation

make sure you have [rust](https://www.rust-lang.org) and [uv](https://github.com/astral-sh/uv) installed

### from source

1. Clone this repository:
   ```sh
   git clone https://github.com/guangyu-he/py_executer
   cd py_executer
   ```
2. Build the project and try:
   ```sh
   cargo build --release
   ```
3. Or install the binary:
    ```sh
    cargo install --path .
    ```

## from crates.io

```sh
cargo install py_executer
```

## Usage

```sh
py_executer <SCRIPT_PATH> [OPTIONS]
```

### Arguments

- `<SCRIPT_PATH>`: Path to the Python script to execute.

### Options

- `-v`, `--venv <VENV_PATH>`: Specify a custom virtual environment path (default: `.venv` or `venv`). If a valid venv is
  present, this venv will be used directly (not managed by uv), requirements.txt will not be installed, and clean mode
  will be ignored.
- `-E`, `--env <KEY=VALUE>`: Additional environment variables in the format KEY=VALUE. Can be used multiple times.
- `-e`, `--env-file <ENV_FILE>`: Path to a .env file (default: `.env` in the current directory).
- `--quiet`: Suppress output from the CLI.
- `--clean`: Clean the created uv-managed .venv and config files after execution. Pre-existing files are not deleted.
- `-A`, `--py-arg <ARGs>`: Additional arguments to pass to the Python script. Must be placed as the last argument(s) and
  will be passed directly to Python.

### Example

#### minimum usage

assume there is a project like this:

```
project/
├── myscript.py
├── requirements.txt
└── .env
```

```sh
py_executer my_script.py
```

this will be equivalent to:

```sh
uv init --bare
uv venv
uv add -r requirements.txt
export $(grep -v '^#' .env | xargs)  # if .env exists
PYTHONPATH=$PYTHONPATH:$(pwd)
.venv/bin/python my_script.py
```

after the execution, the script will create uv project files

```
project/
├── myscript.py
├── requirements.txt
├── .env
├── uv.lock
├── .venv/
└── pyproject.toml
```

to clean up generated files afterward, you can add `--clean` in the argument

#### more customized options

```sh
py_executer my_script.py -v venv -E DEBUG=true -A --input data.txt
```

this will be equivalent to:

```sh
export $(grep -v '^#' .env | xargs)
PYTHONPATH=$PYTHONPATH:$(pwd)
DEBUG=true
venv/bin/python3 my_script.py --input data.txt
```

## Project Structure

- `src/main.rs`: Main CLI logic and environment setup.
- `src/lib/`: Internal modules for utilities, macros, and uv integration.

## Contributing

Pull requests are welcome! For major changes, please open an issue first to discuss what you would like to change.
