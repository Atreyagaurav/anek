use anyhow::Error;
use clap::{ArgGroup, Args, ValueHint};
use colored::Colorize;
use std::path::PathBuf;

use crate::dtypes::AnekDirectoryType;
use crate::{
    dtypes::{anekdirtype_iter, AnekDirectory},
    variable,
};

#[derive(Args)]
#[command(group = ArgGroup::new("anek-type").required(false).multiple(false))]
pub struct CliArgs {
    /// Filter list to those matching pattern
    #[arg(short = 'F', long, value_delimiter = ',', requires = "anek-type", value_hint = ValueHint::Other)]
    filter: Vec<String>,
    /// Search for the content inside the matched files
    ///
    /// Prints the line number and the line with the matched content
    #[arg(short, long, requires = "anek-type", value_hint = ValueHint::Other)]
    search: Vec<String>,
    /// Variables
    ///
    /// List the variables in the variables directory
    #[arg(short, long, group = "anek-type", action)]
    variables: bool,
    /// Inputs
    ///
    /// List the files you've saved as inputs
    #[arg(short, long, group = "anek-type", action)]
    inputs: bool,
    /// Commands
    ///
    ///
    #[arg(short, long, group = "anek-type", action)]
    command: bool,
    /// Pipelines
    ///
    /// List the pipelines files.
    #[arg(short, long, group = "anek-type", action)]
    pipeline: bool,
    /// Templates
    ///
    /// List the templates files.
    #[arg(short, long, group = "anek-type", action)]
    template: bool,
    /// Loops
    ///
    /// List the files in loop directory,
    #[arg(short, long, group = "anek-type", action)]
    loops: bool,
    /// Batch
    ///
    /// List the batch files,
    #[arg(short, long, group = "anek-type", action)]
    batch: bool,
    /// All files
    ///
    /// Do not skip files inside .d directories
    #[arg(short, long, action)]
    all: bool,
    #[arg(default_value = ".", value_hint = ValueHint::DirPath)]
    path: PathBuf,
}

pub fn list_options(args: CliArgs) -> Result<(), Error> {
    let anek_dir = AnekDirectory::from(&args.path)?;
    let list_func = if args.all {
        variable::list_filenames
    } else {
        variable::list_anek_filenames
    };
    let anek_type = if args.variables {
        Some(AnekDirectoryType::Variables)
    } else if args.inputs {
        Some(AnekDirectoryType::Inputs)
    } else if args.batch {
        Some(AnekDirectoryType::Batch)
    } else if args.loops {
        Some(AnekDirectoryType::Loops)
    } else if args.command {
        Some(AnekDirectoryType::Commands)
    } else if args.pipeline {
        Some(AnekDirectoryType::Pipelines)
    } else if args.template {
        Some(AnekDirectoryType::Templates)
    } else {
        None
    };
    let anek_types: Vec<(&str, &AnekDirectoryType)> = if let Some(ref at) = anek_type {
        vec![("", at)]
    } else {
        anekdirtype_iter().map(|d| (d.dir_name(), d)).collect()
    };
    let mut paths: Vec<String> = anek_types
        .iter()
        .map(|adt| -> Result<Vec<String>, Error> {
            list_func(&anek_dir.get_directory(adt.1)).map(|d| {
                d.iter()
                    .map(|f| {
                        if adt.0.is_empty() {
                            f.to_string()
                        } else {
                            format!("{}/{}", adt.0, f)
                        }
                    })
                    .collect::<Vec<String>>()
            })
        })
        .collect::<Result<Vec<Vec<String>>, Error>>()?
        .into_iter()
        .flatten()
        .collect();
    if !args.filter.is_empty() {
        paths = paths
            .into_iter()
            .filter_map(|p| {
                if args.filter.iter().all(|fs| p.contains(fs)) {
                    Some(p)
                } else {
                    None
                }
            })
            .collect();
    }
    for p in paths {
        match p.rsplit_once("/") {
            Some((dir, file)) => println!("{}/{}", dir.truecolor(100, 100, 100), file),
            None => println!("{}", p),
        }
        if !args.search.is_empty() {
            let fpath = if let Some(ref at) = anek_type {
                anek_dir.get_file(&at, &p)
            } else {
                anek_dir.root.join(p)
            };
            let matching_lines = variable::matching_lines(&fpath, &args.search, false)?;
            for (ln, line) in matching_lines {
                println!("{ln}: {line}",);
            }
        }
    }
    Ok(())
}
