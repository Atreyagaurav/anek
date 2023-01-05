use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;
use std::time::Instant;

mod input;
mod list;
mod run;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Command to run
    #[command(subcommand)]
    action: Action,

    ///Working Directory
    ///
    /// The path should have .anek directory with anek configurations
    #[arg(default_value = ".")]
    path: PathBuf,
}

#[derive(Subcommand)]
enum Action {
    // /// make a new file
    // New(PathBuf),
    // /// pull a file from anek file to current path
    // Pull(PathBuf),
    // /// push a file from current path to anek file
    // Push(PathBuf),
    // /// edit a file in the anek file
    // Edit(PathBuf),
    // /// edit a command in the anek file
    // Command(String),
    // /// favorite
    // Favorite(String),
    /// Input related commands
    Input(input::CliArgs),
    /// list things
    List(list::CliArgs),
    /// run the file
    Run(run::CliArgs),
}

fn main() {
    let args = Cli::parse();

    let start = Instant::now();
    let action_result = match args.action {
        Action::Input(args) => input::run_command(args),
        Action::List(args) => list::list_options(args),
        Action::Run(args) => run::run_command(args),
    };
    let duration = start.elapsed();
    match action_result {
        Ok(_) => (),
        Err(e) => eprintln!("{}: {}", "Error".bright_red(), e),
    }
    eprintln!("{}: {:?}", "Time Elapsed".bright_blue().bold(), duration);
}
