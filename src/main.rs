mod python;
mod uv;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process;

use python::python;
use uv::uv;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run script mode
    Run {
        /// Script path
        #[clap(index = 1)]
        script: PathBuf,

        /// Project path
        #[clap(short, long, default_value = ".")]
        project: PathBuf,

        /// Additional environment variables in the format KEY=VALUE (can be used multiple times)
        #[clap(short = 'E', long)]
        env: Vec<String>,

        /// .env file path, if provided, it will be loaded.
        /// If a .env file is found under --project path, it will be loaded automatically
        #[clap(short = 'e', long)]
        env_file: Option<PathBuf>,

        /// Suppress output
        #[clap(long, default_value_t = false)]
        quiet: bool,

        /// Clean mode
        /// if specified, it will clean the created uv .venv and configs
        /// if those files originally exist, they will not be deleted
        #[clap(long, default_value_t = false)]
        clean: bool,

        /// Python arguments, must be placed as the last argument
        #[arg(short = 'A', long = "py_arg", num_args = 1.., value_delimiter = ' ')]
        py_args: Vec<String>,
    },
    /// UV mode - pass all arguments to uv command
    Uv {
        /// Arguments to pass to uv command
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

fn main() -> process::ExitCode {
    let args = Args::parse();

    match args.command {
        Commands::Run {
            script,
            project,
            env,
            env_file,
            quiet,
            clean,
            py_args,
        } => python(script, project, env, env_file, quiet, clean, py_args),
        Commands::Uv { args } => uv(args),
    }
}
