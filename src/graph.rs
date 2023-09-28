use anyhow::Error;
use clap::{Args, ValueHint};
use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::dtypes::{anekdirtype_iter, AnekDirectory, AnekDirectoryType};
use crate::variable;

#[derive(Args)]
pub struct CliArgs {
    /// Remove orphaned items. TODO
    #[arg(short, long, action)]
    remove_orphans: bool,
    /// Only include these pipeline
    #[arg(short, long, value_delimiter = ',')]
    pipelines: Vec<String>,
    /// Only include these batch files
    #[arg(short, long, value_delimiter = ',')]
    batches: Vec<String>,
    #[arg(default_value = ".", value_hint=ValueHint::DirPath)]
    path: PathBuf,
}

lazy_static! {
    static ref NODE_COLORS: HashMap<&'static str, &'static str> = HashMap::from([
        (AnekDirectoryType::Inputs.dir_name(), "red"),
        (AnekDirectoryType::Pipelines.dir_name(), "blue"),
        (AnekDirectoryType::Batch.dir_name(), "orange"),
        (AnekDirectoryType::Variables.dir_name(), "yellow"),
        (AnekDirectoryType::Commands.dir_name(), "brown"),
    ]);
}

pub fn print_dot(args: CliArgs) -> Result<(), Error> {
    let filepath = AnekDirectory::from(&args.path)?;
    println!("digraph anek{{");
    println!("rank=LR;");
    println!("overlap=prism; overlap_scaling=-3.5;");
    println!("node [penwidth=2, shape=rectangle];");

    for dt in anekdirtype_iter() {
        println!("subgraph cluster_{} {{", dt.dir_name());
        println!("label = {};", dt.dir_name());
        let color = NODE_COLORS.get(dt.dir_name()).unwrap_or(&"gray");
        for file in variable::list_anek_filenames(&filepath.get_directory(dt))? {
            println!("\"{}\" [color={}]", file, color);
        }
        println!("}}");
    }
    // pipelines as sequence of commands
    let dir = filepath.get_directory(&AnekDirectoryType::Pipelines);
    let pipelines = if args.pipelines.is_empty() {
        variable::list_filenames(&dir)?
    } else {
        args.pipelines
    };
    for file in pipelines {
        let lines = variable::input_lines(&dir.join(&file), None)?;
        let conn = lines
            .iter()
            .map(|(_, s)| format!("\"{s}\""))
            .collect::<Vec<String>>()
            .join(" -> ");
        println!("{conn} -> \"{file}\" [color=blue]");
    }
    // commands as collection of variables
    let dir = filepath.get_directory(&AnekDirectoryType::Commands);
    for file in variable::list_filenames(&dir)? {
        let mut inputs: HashSet<&str> = HashSet::new();
        let lines = variable::input_lines(&dir.join(&file), None)?;
        variable::read_inputs_set_from_commands(&lines, &mut inputs)?;
        for input in inputs {
            println!("\"{input}\" -> \"{file}\"  [color=yellow]");
        }
    }
    // inputs as collection of variables
    let dir = filepath.get_directory(&AnekDirectoryType::Inputs);
    for file in variable::list_anek_filenames(&dir)? {
        let mut inputs: HashSet<&str> = HashSet::new();
        let lines = variable::compact_lines_from_anek_file(&[dir.join(&file)].to_vec())?;
        variable::read_inputs_set(&lines, &mut inputs)?;
        for input in inputs {
            println!("\"{input}\" -> \"{file}\"  [color=pink]");
        }
    }
    // batch as sequence of inputs
    let dir = filepath.get_directory(&AnekDirectoryType::Batch);
    let batches = if args.batches.is_empty() {
        variable::list_filenames(&dir)?
    } else {
        args.batches
    };
    for file in batches {
        let lines = variable::input_lines(&dir.join(&file), None)?;
        let conn = lines
            .iter()
            .map(|(_, s)| format!("\"{s}\""))
            .collect::<Vec<String>>()
            .join(" -> ");
        println!("{conn} -> \"{file}\" [color=orange]");
    }
    println!("}}");
    Ok(())
}
