use anyhow::Error;
use clap::{Args, ValueHint};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use string_template_plus::{Render, RenderOptions, Template};

use crate::dtypes::AnekDirectory;
use itertools::Itertools;

use crate::run_utils;

#[derive(Args, Default)]
pub struct CliArgs {
    /// Export according to the format specified (csv,json,plain)
    #[arg(short, long, default_value="csv", value_hint = ValueHint::Other)]
    format: String,
    /// Output file for Export
    #[arg(short, long, value_hint = ValueHint::Other)]
    output: Option<PathBuf>,
    /// Variables to export
    #[arg(short, long,  value_delimiter=',', value_hint = ValueHint::Other)]
    variables: Vec<String>,

    #[command(subcommand)]
    inputs: run_utils::Inputs,
}

impl CliArgs {
    pub fn from_gui(
        format: String,
        variables: Vec<String>,
        inputs: run_utils::Inputs,
        output: Option<PathBuf>,
    ) -> Self {
        Self {
            format,
            variables,
            inputs,
            output,
        }
    }
}

struct ExportWrapers {
    vars_templ: Template,
    start: String,
    start_line: String,
    connector: String,
    end_line: String,
    end: String,
}

impl ExportWrapers {
    pub fn new<T: ToString, U: ToString, V: ToString, W: ToString, X: ToString>(
        vars_templ: Template,
        start: T,
        start_line: U,
        connector: V,
        end_line: W,
        end: X,
    ) -> Self {
        Self {
            vars_templ,
            start: start.to_string(),
            start_line: start_line.to_string(),
            connector: connector.to_string(),
            end_line: end_line.to_string(),
            end: end.to_string(),
        }
    }

    pub fn from_name(name: &str, vars: &Vec<String>) -> Result<Self, Error> {
        match name {
            "csv" => Self::csv(vars),
            "json" => Self::json(vars),
            &_ => Self::plain(vars),
        }
    }

    pub fn plain(vars: &[String]) -> Result<Self, Error> {
        Ok(Self::new(
            Template::parse_template(
                &vars
                    .iter()
                    .map(|v| format!("{}={{{}}}", v.trim_end_matches('?'), v))
                    .collect::<Vec<String>>()
                    .join("\n"),
            )?,
            "",
            "",
            "\n",
            "",
            "",
        ))
    }

    pub fn csv(vars: &[String]) -> Result<Self, Error> {
        Ok(Self::new(
            Template::parse_template(&format!("{{{}}}", vars.join("},{")))?,
            format!(
                "{}\n",
                vars.iter().map(|v| v.trim_end_matches('?')).join(",")
            ),
            "",
            "\n",
            "",
            "",
        ))
    }

    pub fn json(vars: &[String]) -> Result<Self, Error> {
        Ok(Self::new(
            Template::parse_template(
                &vars
                    .iter()
                    .map(|v| format!("\"{}\":\"{{{}}}\"", v.trim_end_matches('?'), v))
                    .collect::<Vec<String>>()
                    .join(","),
            )?,
            "[\n",
            "  {",
            ",\n",
            "}",
            "\n]",
        ))
    }
}

pub fn run_command(args: CliArgs, anek_dir: AnekDirectory) -> Result<(), Error> {
    let wrappers = ExportWrapers::from_name(&args.format, &args.variables)?;
    let cmd_args = run_utils::command_args(&args.inputs);
    let overwrite: HashMap<String, String> = run_utils::overwrite_vars(&args.inputs, &cmd_args)?;
    let input_files = run_utils::inputs(&anek_dir, &args.inputs, &HashSet::new())?;

    let mut renderop = RenderOptions {
        wd: PathBuf::from("."),
        variables: HashMap::new(),
        shell_commands: false,
    };

    let file = args.output.map(|f| File::create(f).unwrap());

    let mut writer: Box<dyn Write> = match file {
        Some(f) => Box::new(BufWriter::new(f)),
        None => Box::new(std::io::stdout()),
    };

    write!(writer, "{}", wrappers.start)?;
    let total = input_files.len();
    for (i, input) in input_files.iter().enumerate() {
        let i = i + 1;
        write!(writer, "{}", wrappers.start_line)?;
        input.eprint_job(i, total);
        renderop.variables = run_utils::variables_from_input(input, &overwrite)?;
        write!(writer, "{}", wrappers.vars_templ.render(&renderop)?)?;
        write!(writer, "{}", wrappers.end_line)?;
        if i < total {
            write!(writer, "{}", wrappers.connector)?;
        }
    }
    writeln!(writer, "{}", wrappers.end)?;
    Ok(())
}
