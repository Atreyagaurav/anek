use anyhow::Error;
use clap::{Args, ValueHint};
use std::collections::{HashMap, HashSet};

use crate::dtypes::{AnekDirectory, Command};
use crate::run_utils;

#[derive(Args, Default)]
pub struct CliArgs {
    /// Command is a template to run
    ///
    /// Command templates are bash commands with variables enclosed in
    /// curly braces. e.g. --template 'echo {mean}', will run
    /// echo with value of `mean` as the first argument.
    #[arg(short, long)]
    template: bool,
    /// The following command is a pipeline
    #[arg(short, long)]
    pipeline: bool,
    /// Don't Print commands
    #[arg(short, long)]
    /// Don't print the other info except commands
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
    #[command(subcommand)]
    inputs: run_utils::Inputs,
}

impl CliArgs {
    pub fn from_gui(pipeline: bool, command: String, inputs: run_utils::Inputs) -> Self {
        Self {
            pipeline,
            command,
            inputs,
            ..Default::default()
        }
    }
}

pub fn run_command(args: CliArgs, anek_dir: AnekDirectory) -> Result<(), Error> {
    let commands = if args.template {
        vec![Command::new("-T-", &args.command)?]
    } else if args.pipeline {
        run_utils::cmd_from_pipeline(&anek_dir, &args.command)?
    } else {
        vec![anek_dir.command(&args.command)?]
    };

    let mut variables: HashSet<&str> = HashSet::new();
    for cmd in &commands {
        cmd.template().parts().iter().for_each(|p| {
            p.variables().iter().for_each(|v| {
                variables.insert(v);
            })
        });
    }

    let cmd_args = run_utils::command_args(&args.inputs);
    let overwrite: HashMap<String, String> = run_utils::overwrite_vars(&args.inputs, &cmd_args)?;

    let input_files = run_utils::inputs(&anek_dir, &args.inputs, &variables)?;
    let total = input_files.len();
    for (i, input) in input_files.iter().enumerate() {
        if !args.pipable {
            input.eprint_job(i + 1, total);
        }
        let variables = run_utils::variables_from_input(input, &overwrite)?;
        for cmd in &commands {
            cmd.run(&variables, args.demo, args.pipable, anek_dir.proj_root())?;
        }
    }
    Ok(())
}
