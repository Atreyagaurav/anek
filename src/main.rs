use clap::{CommandFactory, Parser, Subcommand};
use clap_complete;
use colored::Colorize;
use std::io;
use std::time::Instant;

mod input;
mod list;
mod run;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// No other outputs
    #[arg(short, long)]
    quiet: bool,
    /// Command to run
    #[command(subcommand)]
    action: Action,
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
    /// Print completions
    Completions,
}

fn print_completions() -> Result<(), String> {
    let shell = clap_complete::Shell::Bash;
    let mut clap_app = Cli::command();
    let app_name = clap_app.get_name().to_string();
    clap_complete::generate(shell, &mut clap_app, app_name, &mut io::stdout());
    Ok(())
}

fn main() {
    let args = Cli::parse();

    let start = Instant::now();
    let action_result = match args.action {
        Action::Input(args) => input::run_command(args),
        Action::List(args) => list::list_options(args),
        Action::Run(args) => run::run_command(args),
        Action::Completions => print_completions(),
    };
    let duration = start.elapsed();

    if args.quiet {
        return;
    }
    if let Err(e) = action_result {
        eprintln!("{}: {}", "Error".bright_red(), e);
    }
    eprintln!("{}: {:?}", "Time Elapsed".bright_blue().bold(), duration);
}
