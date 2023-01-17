use clap::{CommandFactory, Parser, Subcommand};
use colored::Colorize;
use std::time::Instant;

mod completions;
mod dtypes;
mod edit;
mod list;
mod new;
mod run;
mod variable;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// No other outputs
    ///
    /// Don't print the Time taken part.
    #[arg(short, long)]
    quiet: bool,
    /// Command to run
    ///
    /// Any command that you want to run, all the args after this will
    /// be passed to that command.
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand)]
enum Action {
    /// make a new anek configuration
    ///
    /// Make a new configuration setup (directories) in current
    /// directory, or the path specified. if you provide input
    /// arguments, it'll also add those to input directory.
    New(new::CliArgs),
    /// Variable related commands
    ///
    /// Commands related to variable files. Variable files are not necessary
    /// to run any of the commands, as long as their values are
    /// provided in the terminal or the favorites files.
    ///
    /// Variable files are helpful for using the completion features and
    /// to maintain the documentations about the inputs. You can write
    /// variable files with the short description in the first line, and
    /// then full description.
    Variable(variable::CliArgs),
    /// list things
    ///
    /// List available things like favorites, batches, commands,
    /// pipelines etc. It is mostly used for generating
    /// autocomplete. But users can also use it to list the available
    /// options for them.
    List(list::CliArgs),
    /// Edit .anek files
    ///
    /// Edit or view files inside ~.anek~. It basically just calls
    /// your editor, so consider this a shortcut for calling your
    /// ~EDITOR~. You shouldn't specify the full path, but relative
    /// path from inside ~.anek~.
    ///
    /// All the valid paths can be listed using ~anek list
    /// -a~. Completion will help you there by proving them.
    Edit(edit::CliArgs),
    /// run the file
    ///
    /// Main command to run/print the commands or pipelines.
    Run(run::CliArgs),
    /// Print completions
    Completions(completions::CliArgs),
}

fn main() {
    let args = Cli::parse();

    let start = Instant::now();
    let action_result = match args.action {
        Action::New(args) => new::new_config(args),
        Action::Variable(args) => variable::run_command(args),
        Action::List(args) => list::list_options(args),
        Action::Edit(args) => edit::edit_file(args),
        Action::Run(args) => run::run_command(args),
        Action::Completions(args) => {
            let mut clap_app = Cli::command();
            completions::print_completions(args, &mut clap_app)
        }
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
