use anyhow::Error;
use clap::{Args, ValueHint};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use string_template_plus::{Render, RenderOptions, Template};

use crate::dtypes::AnekDirectory;
use crate::run_utils;

#[derive(Args)]
pub struct CliArgs {
    /// Render the given template like command templates
    ///
    /// Renders a template. You pass template instead of file.
    #[arg(short, long, group="action", value_hint = ValueHint::Other)]
    template: bool,
    /// Render the given file like command templates
    ///
    /// Renders a file. Just write the file anywhere in your path with
    /// templates for input variables in the same way you write
    /// command templates. It's useful for markdown files, as the
    /// curly braces syntax won't be used for anything else that
    /// way. Do be careful about that. And the program will replace
    /// those templates with their values when you run it with inputs.
    #[arg(value_hint = ValueHint::FilePath)]
    file: String,

    #[command(subcommand)]
    inputs: run_utils::Inputs,
}

pub fn run_command(args: CliArgs) -> Result<(), Error> {
    let anek_dir = AnekDirectory::from_pwd()?;
    let template = if args.template {
        Template::parse_template(args.file.trim())?
    } else {
        let path = PathBuf::from(args.file);
        let file_content = fs::read_to_string(&path)?;
        Template::parse_template(&file_content.trim())?
    };

    let cmd_args = run_utils::command_args(&args.inputs);
    let overwrite: HashMap<String, String> = run_utils::overwrite_vars(&args.inputs, &cmd_args)?;

    let input_files = run_utils::inputs(&anek_dir, &args.inputs, &HashSet::new())?;
    let total = input_files.len();
    for (i, input) in input_files.iter().enumerate() {
        input.eprint_job(i + 1, total);
        let variables = run_utils::variables_from_input(input, &overwrite)?;
        let renderops = RenderOptions {
            variables,
            wd: PathBuf::default(),
            shell_commands: true,
        };
        println!("{}", template.render(&renderops)?);
    }
    Ok(())
}
