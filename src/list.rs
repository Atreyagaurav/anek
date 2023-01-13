use clap::{Args, ValueHint};
use colored::Colorize;
use std::{collections::HashMap, path::PathBuf};

use crate::dtypes::AnekDirectoryType;
use crate::{
    dtypes::{anekdirtype_iter, AnekDirectory},
    input,
};

#[derive(Args)]
pub struct CliArgs {
    /// Filter list to those containing values
    #[arg(short = 'F', long, value_delimiter = ',', value_hint = ValueHint::Other)]
    filter: Vec<String>,
    /// All
    #[arg(short, long, action, conflicts_with = "filter")]
    all: bool,
    /// Inputs
    #[arg(short, long, action)]
    input: bool,
    /// Favorites
    ///
    /// List the files you've saved as favorites, the --filter command
    /// will filter the files including the variable value as
    /// non-empty
    #[arg(short, long, action)]
    favorite: bool,
    /// Commands
    #[arg(short, long, action)]
    command: bool,
    /// Pipelines
    #[arg(short, long, action)]
    pipeline: bool,
    /// Loops
    #[arg(short, long, action)]
    loops: bool,
    /// Batch
    #[arg(short, long, action)]
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
    let paths = if args.all {
        let all_dirs = anekdirtype_iter()
            .map(|adt| input::list_filenames(&anek_dir.get_directory(adt)))
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
    } else if args.input {
        let lst = input::list_filenames(&anek_dir.get_directory(&AnekDirectoryType::Inputs))?;
        if args.filter.is_empty() {
            lst
        } else {
            lst.iter()
                .filter(|fnm| {
                    let lines =
                        input::input_lines(&anek_dir.get_file(&AnekDirectoryType::Inputs, &fnm))
                            .unwrap();
                    args.filter
                        .iter()
                        .all(|f| lines.iter().any(|(_, l)| l.contains(f.as_str())))
                })
                .map(|s| s.to_string())
                .collect()
        }
    } else if args.favorite {
        let lst = input::list_filenames(&anek_dir.get_directory(&AnekDirectoryType::Favorites))?;
        if args.filter.is_empty() {
            lst
        } else {
            lst.iter()
                .filter(|f| {
                    let lines =
                        input::input_lines(&anek_dir.get_file(&AnekDirectoryType::Favorites, &f))
                            .unwrap();
                    let mut input_map: HashMap<&str, &str> = HashMap::new();
                    input::read_inputs(&lines, &mut input_map).unwrap();
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
        let lst = input::list_filenames(&anek_dir.get_directory(&AnekDirectoryType::Batch))?;
        if args.filter.is_empty() {
            lst
        } else {
            lst.iter()
                .filter(|fnm| {
                    let lines =
                        input::input_lines(&anek_dir.get_file(&AnekDirectoryType::Batch, &fnm))
                            .unwrap();
                    args.filter
                        .iter()
                        .all(|f| lines.iter().any(|(_, l)| l.contains(f.as_str())))
                })
                .map(|s| s.to_string())
                .collect()
        }
    } else if args.loops {
        input::list_filenames(&anek_dir.get_directory(&AnekDirectoryType::Loop))?
    } else if args.command {
        let lst = input::list_filenames(&anek_dir.get_directory(&AnekDirectoryType::Commands))?;
        if args.filter.is_empty() {
            lst
        } else {
            lst.iter()
                .filter(|fnm| {
                    let cmd_content = std::fs::read_to_string(
                        &anek_dir.get_file(&AnekDirectoryType::Commands, &fnm),
                    )
                    .unwrap();
                    args.filter
                        .iter()
                        .all(|f| cmd_content.contains(&format!("{{{}}}", f)))
                })
                .map(|s| s.to_string())
                .collect()
        }
    } else if args.pipeline {
        let lst = input::list_filenames(&anek_dir.get_directory(&AnekDirectoryType::Pipelines))?;
        if args.filter.is_empty() {
            lst
        } else {
            lst.iter()
                .filter(|fnm| {
                    let lines =
                        input::input_lines(&anek_dir.get_file(&AnekDirectoryType::Pipelines, &fnm))
                            .unwrap();
                    args.filter
                        .iter()
                        .all(|f| lines.iter().any(|(_, l)| l.contains(f.as_str())))
                })
                .map(|s| s.to_string())
                .collect()
        }
    } else {
        input::list_filenames(&anek_dir.get_directory(&AnekDirectoryType::History))?
    };
    for p in paths {
        match p.rsplit_once("/") {
            Some((dir, file)) => println!("{}/{}", dir.truecolor(100, 100, 100), file),
            None => println!("{}", p),
        }
    }
    Ok(())
}
