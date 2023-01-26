use clap::{ArgGroup, Args, ValueHint};
use colored::Colorize;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use crate::dtypes::AnekDirectoryType;
use crate::{
    dtypes::{anekdirtype_iter, AnekDirectory},
    variable,
};

#[derive(Args)]
#[command(group = ArgGroup::new("anek-type").required(false).multiple(false))]
pub struct CliArgs {
    /// Filter list to those containing values
    #[arg(short = 'F', long, value_delimiter = ',', requires = "anek-type", value_hint = ValueHint::Other)]
    filter: Vec<String>,
    /// Variables
    ///
    /// List the variables in the variables directory, if you use
    /// --filter, it'll list only the variables with matchig text in
    /// their description
    #[arg(short, long, group = "anek-type", action)]
    variables: bool,
    /// Inputs
    ///
    /// List the files you've saved as inputs, the --filter command
    /// will filter the files including the variable value as
    /// non-empty
    #[arg(short, long, group = "anek-type", action)]
    inputs: bool,
    /// Commands
    ///
    ///
    #[arg(short, long, group = "anek-type", action)]
    command: bool,
    /// Pipelines
    ///
    /// List the pipelines files. The --filter will filter the
    /// pipelines containing the given keyword as a command
    #[arg(short, long, group = "anek-type", action)]
    pipeline: bool,
    /// Loops
    ///
    /// List the files in loop directory, --filter will only list the
    /// files that have the filter keyword in their path
    #[arg(short, long, group = "anek-type", action)]
    loops: bool,
    /// Batch
    ///
    /// List the batch files, --filter will filter the files
    /// containing the inputs in the file
    #[arg(short, long, group = "anek-type", action)]
    batch: bool,
    #[arg(default_value = ".", value_hint = ValueHint::DirPath)]
    path: PathBuf,
}

pub fn list_options(args: CliArgs) -> Result<(), String> {
    let anek_dir = AnekDirectory::from(&args.path);
    if !anek_dir.exists() {
        return Err(format!(
            "Directory [{:?}] doesn't have .anek config.",
            args.path
        ));
    }
    let paths = if args.variables {
        let lst = variable::list_filenames(&anek_dir.get_directory(&AnekDirectoryType::Variables))?;
        if args.filter.is_empty() {
            lst
        } else {
            lst.iter()
                .filter(|fnm| {
                    let lines = variable::input_lines(
                        &anek_dir.get_file(&AnekDirectoryType::Variables, &fnm),
                        None,
                    )
                    .unwrap();
                    args.filter
                        .iter()
                        .all(|f| lines.iter().any(|(_, l)| l.contains(f.as_str())))
                })
                .map(|s| s.to_string())
                .collect()
        }
    } else if args.inputs {
        let lst = variable::list_filenames(&anek_dir.get_directory(&AnekDirectoryType::Inputs))?;
        if args.filter.is_empty() {
            lst
        } else {
            lst.iter()
                .filter(|f| {
                    let lines = variable::input_lines(
                        &anek_dir.get_file(&AnekDirectoryType::Inputs, &f),
                        None,
                    )
                    .unwrap();
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
    } else if args.batch {
        let lst = variable::list_filenames(&anek_dir.get_directory(&AnekDirectoryType::Batch))?;
        if args.filter.is_empty() {
            lst
        } else {
            lst.iter()
                .filter(|fnm| {
                    let lines = variable::input_lines(
                        &anek_dir.get_file(&AnekDirectoryType::Batch, &fnm),
                        None,
                    )
                    .unwrap();
                    args.filter
                        .iter()
                        .all(|f| lines.iter().any(|(_, l)| l == f))
                })
                .map(|s| s.to_string())
                .collect()
        }
    } else if args.loops {
        let lst = variable::list_filenames(&anek_dir.get_directory(&AnekDirectoryType::Loops))?;
        if args.filter.is_empty() {
            lst
        } else {
            lst.iter()
                .filter(|name| args.filter.iter().all(|f| name.contains(f)))
                .map(|s| s.to_string())
                .collect()
        }
    } else if args.command {
        let lst = variable::list_filenames(&anek_dir.get_directory(&AnekDirectoryType::Commands))?;
        if args.filter.is_empty() {
            lst
        } else {
            lst.iter()
                .filter(|fnm| {
                    let cmd_content = variable::input_lines(
                        &anek_dir.get_file(&AnekDirectoryType::Commands, &fnm),
                        None,
                    )
                    .unwrap();
                    let mut cmd_variables: HashSet<&str> = HashSet::new();
                    variable::read_inputs_set_from_commands(&cmd_content, &mut cmd_variables)
                        .unwrap();

                    args.filter
                        .iter()
                        .all(|f| cmd_variables.contains(f.as_str()))
                })
                .map(|s| s.to_string())
                .collect()
        }
    } else if args.pipeline {
        let lst = variable::list_filenames(&anek_dir.get_directory(&AnekDirectoryType::Pipelines))?;
        if args.filter.is_empty() {
            lst
        } else {
            lst.iter()
                .filter(|fnm| {
                    let lines = variable::input_lines(
                        &anek_dir.get_file(&AnekDirectoryType::Pipelines, &fnm),
                        None,
                    )
                    .unwrap();
                    args.filter
                        .iter()
                        .all(|f| lines.iter().any(|(_, l)| l == f))
                })
                .map(|s| s.to_string())
                .collect()
        }
    } else {
        let all_dirs = anekdirtype_iter()
            .map(|adt| variable::list_filenames(&anek_dir.get_directory(adt)))
            .collect::<Result<Vec<Vec<String>>, String>>()?;
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
    }
    Ok(())
}
