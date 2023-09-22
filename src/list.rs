use anyhow::Error;
use clap::{ArgGroup, Args, ValueHint};
use colored::Colorize;
use std::{collections::HashMap, path::PathBuf};

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
    } else {
        None
    };
    let paths = if let Some(ref at) = anek_type {
        let lst = list_func(&anek_dir.get_directory(&at))?;
        if args.filter.is_empty() {
            lst
        } else {
            lst.iter()
                .filter(|f| {
                    let lines = variable::input_lines(&anek_dir.get_file(&at, &f), None).unwrap();
                    let mut input_map: HashMap<&str, &str> = HashMap::new();
                    variable::read_inputs(&lines, &mut input_map).unwrap();
                    args.filter.iter().all(|f| {
                        if let Some(key) = input_map.get(f.as_str()) {
                            !key.is_empty()
                        } else {
                            false
                        }
                    })
                })
                .map(|s| s.to_string())
                .collect()
        }
    } else {
        let all_dirs = anekdirtype_iter()
            .map(|adt| list_func(&anek_dir.get_directory(adt)))
            .collect::<Result<Vec<Vec<String>>, Error>>()?;
        anekdirtype_iter()
            .zip(all_dirs)
            .map(|(adt, fnames)| {
                fnames
                    .iter()
                    .map(|f| format!("{}/{}", adt.dir_name(), f))
                    .collect::<Vec<String>>()
            })
            .flatten()
            .collect()
    };

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
