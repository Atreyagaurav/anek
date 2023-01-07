use clap::{Args, ValueHint};
use colored::Colorize;
use std::{collections::HashMap, path::PathBuf};

use crate::input;

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
    if !args.path.join(".anek").is_dir() {
        return Err(format!(
            "Directory [{:?}] doesn't have .anek config.",
            args.path
        ));
    }
    let paths = if args.all {
        input::list_filenames(&args.path.join(".anek/inputs"))?
            .iter()
            .map(|i| format!("inputs/{}", i))
            .chain(
                input::list_filenames(&args.path.join(".anek/favorites"))?
                    .iter()
                    .map(|f| format!("favorites/{}", f)),
            )
            .chain(
                input::list_filenames(&args.path.join(".anek/batch"))?
                    .iter()
                    .map(|f| format!("batch/{}", f)),
            )
            .chain(
                input::list_filenames(&args.path.join(".anek/loops"))?
                    .iter()
                    .map(|f| format!("loops/{}", f)),
            )
            .chain(
                input::list_filenames(&args.path.join(".anek/commands"))?
                    .iter()
                    .map(|f| format!("commands/{}", f)),
            )
            .chain(
                input::list_filenames(&args.path.join(".anek/pipelines"))?
                    .iter()
                    .map(|f| format!("pipelines/{}", f)),
            )
            .collect()
    } else if args.input {
        let lst = input::list_filenames(&args.path.join(".anek/inputs"))?;
        if args.filter.is_empty() {
            lst
        } else {
            lst.iter()
                .filter(|fnm| {
                    let lines =
                        input::input_lines(&args.path.join(".anek/inputs").join(fnm)).unwrap();
                    args.filter
                        .iter()
                        .all(|f| lines.iter().any(|(_, l)| l.contains(f.as_str())))
                })
                .map(|s| s.to_string())
                .collect()
        }
    } else if args.favorite {
        let lst = input::list_filenames(&args.path.join(".anek/favorites"))?;
        if args.filter.is_empty() {
            lst
        } else {
            lst.iter()
                .filter(|f| {
                    let lines =
                        input::input_lines(&args.path.join(".anek/favorites").join(f)).unwrap();
                    let mut input_map: HashMap<&str, &str> = HashMap::new();
                    input::read_inputs(&lines, &mut input_map).unwrap();
                    args.filter
                        .iter()
                        .all(|f| input_map.contains_key(f.as_str()))
                })
                .map(|s| s.to_string())
                .collect()
        }
    } else if args.batch {
        let lst = input::list_filenames(&args.path.join(".anek/batch"))?;
        if args.filter.is_empty() {
            lst
        } else {
            lst.iter()
                .filter(|fnm| {
                    let lines =
                        input::input_lines(&args.path.join(".anek/batch").join(fnm)).unwrap();
                    args.filter
                        .iter()
                        .all(|f| lines.iter().any(|(_, l)| l.contains(f.as_str())))
                })
                .map(|s| s.to_string())
                .collect()
        }
    } else if args.loops {
        input::list_filenames(&args.path.join(".anek/loops"))?
    } else if args.command {
        let lst = input::list_filenames(&args.path.join(".anek/commands"))?;
        if args.filter.is_empty() {
            lst
        } else {
            lst.iter()
                .filter(|fnm| {
                    let cmd_content =
                        std::fs::read_to_string(args.path.join(".anek/commands").join(fnm))
                            .unwrap();
                    args.filter
                        .iter()
                        .all(|f| cmd_content.contains(&format!("{{{}}}", f)))
                })
                .map(|s| s.to_string())
                .collect()
        }
    } else if args.pipeline {
        let lst = input::list_filenames(&args.path.join(".anek/pipelines"))?;
        if args.filter.is_empty() {
            lst
        } else {
            lst.iter()
                .filter(|fnm| {
                    let lines =
                        input::input_lines(&args.path.join(".anek/pipelines").join(fnm)).unwrap();
                    args.filter
                        .iter()
                        .all(|f| lines.iter().any(|(_, l)| l.contains(f.as_str())))
                })
                .map(|s| s.to_string())
                .collect()
        }
    } else {
        input::list_filenames(&args.path.join(".anek/history"))?
    };
    for p in paths {
        match p.rsplit_once("/") {
            Some((dir, file)) => println!("{}/{}", dir.truecolor(100, 100, 100), file),
            None => println!("{}", p),
        }
    }
    Ok(())
}
