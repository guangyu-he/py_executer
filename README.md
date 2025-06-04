# py_executer

A Rust-based command-line tool to execute Python scripts with automatic virtual environment and dependency management.
`py_executer` streamlines running Python code by handling environment setup, dependency installation, and environment
variables, making your workflow faster and more reliable.

## Features

- **Automatic Virtual Environment Management:** Manage a Python virtual environment
  using [uv](https://github.com/astral-sh/uv), if uv not installed, the native python will be used.
- **Dependency Installation:** Installs dependencies from `requirements.txt` automatically.
- **.env File Support:** Loads environment variables from a `.env` file or from CLI.
- **Setup `PYTHONVENV`:** Automatically set python path based on the path where the CLI is executed.
- **Custom Environment Variables:** Pass additional environment variables via CLI.
- **Flexible Python Arguments:** Pass extra arguments to the Python script.
- **Clean Mode:** Clean the created .venv after execution, if there was no venv created before.
- **Cross-platform:** Works on Unix-like systems and Windows.
- **Simple UV wrapper:** Can be used to run uv commands

## Installation

make sure you have [rust](https://www.rust-lang.org) installed. [uv](https://github.com/astral-sh/uv) is optional but
recommended.

### from source

1. Clone this repository:
   ```sh
   git clone https://github.com/guangyu-he/py_executer
   cd py_executer
   ```
2. Install the binary:
    ```sh
    cargo install --path .
    ```

### from Github

```sh
cargo install --git https://github.com/guangyu-he/py_executer
```

## from crates.io

```sh
cargo install py_executer
```

## Usage

### running python script

```sh
py_executer run <SCRIPT_PATH> [OPTIONS]
```

#### Arguments

- `<SCRIPT_PATH>`: Path to the Python script to execute.

#### Options

- `-p`, `--project <PROJECT_PATH>`: Specify the project directory (default: current directory).
- `-v`, `--venv <VENV_PATH>`: Specify a custom virtual environment path (default: `.venv` or `venv`). If a valid venv is
  present, this venv will be used directly (not managed by uv), requirements.txt will not be installed, and clean mode
  will be ignored.
- `-E`, `--env <KEY=VALUE>`: Additional environment variables in the format KEY=VALUE. Can be used multiple times.
- `-e`, `--env-file <ENV_FILE>`: Path to a .env file (default: `.env` in the current directory).
- `--quiet`: Suppress output from the CLI.
- `--clean`: Clean the created uv-managed .venv and config files after execution. Pre-existing files are not deleted.
- `-A`, `--py-args <ARGs>`: Additional arguments to pass to the Python script. Must be placed as the last argument(s)
  and
  will be passed directly to Python.

### running uv command

```sh
py_executer uv <COMMAND> [OPTIONS]
```

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
py_executer run my_script.py
```

this will be equivalent to:

```sh
uv venv
uv pip install -r requirements.txt
# or uv sync --project xxx # if it is an uv project
export $(grep -v '^#' .env | xargs)  # if .env exists
PYTHONPATH=$PYTHONPATH:$(pwd)
uv run --project xxx my_script.py
```

or if no uv installed:

```sh
which python3
python3 -m venv .venv
source .venv/bin/activate
python3 -m pip install -r requirements.txt
export $(grep -v '^#' .env | xargs)
PYTHONPATH=$PYTHONPATH:$(pwd)
python3 my_script.py
```

#### more customized options

```sh
py_executer run my_script.py -v venv -E DEBUG=true -A --input data.txt
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

## License

MIT