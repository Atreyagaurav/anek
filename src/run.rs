use anyhow::Error;
use clap::{Args, ValueHint};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::dtypes::{AnekDirectory, Command};
use crate::run_utils;

#[derive(Args)]
pub struct CliArgs {
    /// Command is a template to run
    ///
    /// Command templates are bash commands with variables enclosed in
    /// curly braces. e.g. --template 'echo {mean}', will run
    /// echo with value of `mean` as the first argument.
    #[arg(short, long)]
    template: bool,
    /// Print commands only so you can pipe it, assumes --demo
    ///
    /// This one will only print the commands without executing them
    /// or printing any other informations. You can pipe it to bash or
    /// gnu parallel or any other commands to run it, or process it
    /// further.
    #[arg(short = 'P', long)]
    pipable: bool,
    /// Demo only, don't run the actual command
    ///
    /// Does everything but skips running the command, you can make
    /// sure it's what you want to run before running it.
    #[arg(short, long)]
    demo: bool,
    /// command to run (from .anek/commands/)
    ///
    /// The command file saved will have the command template inside
    /// it. And it'll run that command (filling it with variables'
    /// values if there are some).
    #[arg(value_hint = ValueHint::Other)]
    command: String,
    #[arg(default_value = ".", value_hint = ValueHint::DirPath)]
    path: PathBuf,

    #[command(subcommand)]
    inputs: run_utils::Inputs,
}

pub fn run_command(args: CliArgs) -> Result<(), Error> {
    let anek_dir = AnekDirectory::from(&args.path)?;
    let command = if args.template {
        Command::new("-T-", &args.command)?
    } else {
        anek_dir.command(&args.command)?
    };

    let cmd_args = run_utils::command_args(&args.inputs);
    let overwrite: HashMap<String, String> = run_utils::overwrite_vars(&args.inputs, &cmd_args)?;

    let input_files = run_utils::inputs(&anek_dir, &args.inputs)?;
    let total = input_files.len();
    for (i, input) in input_files.iter().enumerate() {
        input.eprint_job(i + 1, total);
        let variables = run_utils::variables_from_input(input, &args.path, &overwrite)?;
        command.run(&variables, &args.path, args.demo, !args.pipable)?;
    }
    Ok(())
}
