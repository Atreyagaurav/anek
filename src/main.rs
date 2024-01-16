use anyhow::Error;
use chrono::Local;
use clap::{CommandFactory, Parser, Subcommand};
use colored::Colorize;
use std::time::Instant;

mod completions;
mod dtypes;
mod edit;
mod editor;
mod export;
mod graph;
mod gui;
mod list;
mod new;
mod render;
mod report;
mod run;
mod run_utils;
mod show;
mod variable;
mod view;

#[derive(Parser)]
#[command(name = "anek", author, about, version)]
struct Cli {
    /// No other outputs
    ///
    /// Don't print the Time taken part. Not applicable for subcommand
    /// outputs.
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
    /// Open the GUI for command run
    Gui,
    /// Open the Editor GUI
    Editor,
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
    /// All the valid paths can be listed using ~anek list~.
    /// Completion will help you there by proving them.
    Edit(edit::CliArgs),
    Export(export::CliArgs),
    /// run the file
    ///
    /// Main command to run/print the commands or pipelines.
    Run(run::CliArgs),
    /// render the file
    ///
    /// Main command print the commands or pipelines.
    Render(render::CliArgs),
    /// Print completions for different shells.
    ///
    /// If you use bash, use the edited one included in the repo.
    Completions(completions::CliArgs),
    /// Generate a markdown with anek configurations
    ///
    /// This one will generate a markdown with details on what
    /// configuration files there are, which variables they use, what
    /// input values they have and such. It's just a compilation of
    /// the .anek files with their contents, so that it'll be some
    /// sort of documentation for people without anek program, or just
    /// for you to look at, instead of having to browse the files.
    Report(report::CliArgs),
    /// Show the current Anek Directory
    View(view::CliArgs),
    /// Show the Anek file
    Show(show::CliArgs),
    /// Generate a graph in DOT syntax of the current anek variables
    Graph(graph::CliArgs),
}

fn main() {
    let g_args = Cli::parse();

    let start_time = Local::now().format("%Y-%m-%d %H:%M:%S");
    let start = Instant::now();
    let action_result: Result<(), Error> = match g_args.action {
        Action::Gui => gui::run(),
        Action::Editor => editor::run(),
        Action::New(args) => new::new_config(args),
        Action::Variable(args) => variable::run_command(args),
        Action::List(args) => list::list_options(args),
        Action::Edit(args) => edit::edit_file(args),
        Action::Export(args) => export::run_command(args),
        Action::Run(args) => run::run_command(args),
        Action::Render(args) => render::run_command(args),
        Action::Completions(args) => {
            let mut clap_app = Cli::command();
            completions::print_completions(args, &mut clap_app)
        }
        Action::Report(args) => report::save_report(args),
        Action::View(args) => view::cmd(args),
        Action::Show(args) => show::show_file(args),
        Action::Graph(args) => graph::print_dot(args),
    };
    let duration = start.elapsed();

    if let Err(e) = action_result {
        eprintln!("{}: {}", "Error".bright_red(), e);
    }
    if g_args.quiet {
        return;
    }
    eprintln!("{:12}: {}", "Started at".bright_blue().bold(), start_time);
    eprintln!("{:12}: {:?}", "Time Elapsed".bright_blue().bold(), duration);
}
