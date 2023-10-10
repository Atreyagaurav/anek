use anyhow::{Context, Error};
use clap::{ArgGroup, Args, ValueHint};
use colored::Colorize;
use itertools::Itertools;
use number_range::NumberRangeOptions;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use string_template_plus::Template;
use subprocess::Exec;

use crate::dtypes::{AnekDirectory, AnekDirectoryType};
use crate::export;
use crate::run_utils;
use crate::variable;

#[derive(Args)]
#[command(group = ArgGroup::new("action").required(true).multiple(false))]
pub struct CliArgs {
    /// Render the given file like command templates
    ///
    /// Renders a file. Just write the file anywhere in your path with
    /// templates for input variables in the same way you write
    /// command templates. It's useful for markdown files, as the
    /// curly braces syntax won't be used for anything else that
    /// way. Do be careful about that. And the program will replace
    /// those templates with their values when you run it with inputs.
    #[arg(short, long, group="action", value_hint = ValueHint::FilePath)]
    file: Option<PathBuf>,
    /// Render the given template like command templates
    ///
    /// Renders a template. You pass template instead of file.
    #[arg(short, long, group="action", value_hint = ValueHint::Other)]
    template: Option<String>,
    inputs: run_utils::InputsArgs,
}

pub fn run_command(args: CliArgs) -> Result<(), Error> {
    let anek_dir = AnekDirectory::from(&args.inputs.path)?;
    let template = if let Some(ref filename) = args.file {
        let file_content = fs::read_to_string(&filename)?;
        Template::parse_template(&file_content.trim())?
    } else if let Some(template) = args.template {
        Template::parse_template(template.trim())?
    } else {
        panic!("clap should make sure one of the prev condition is met");
    };

    let cmd_args = run_utils::command_args(&args);
    let mut overwrite: HashMap<&str, &str> = run_utils::overwrite_vars(&args, &cmd_args)?;

    let input_files = run_utils::input_files(&args)?;
    for files in input_files {
        println!(
            "{}",
            run_utils::render_on_inputfile(&template, files, anek_dir.path, &overwrite)?
        );
    }
    Ok(())
}
