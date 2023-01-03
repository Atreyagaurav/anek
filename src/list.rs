use clap::Args;
use std::path::PathBuf;

use crate::input;

#[derive(Args)]
pub struct CliArgs {
    /// Default values
    #[arg(short, long, action)]
    default: bool,
    /// Favorites
    #[arg(short, long, action)]
    favorite: bool,
    /// Commands
    #[arg(short, long, action)]
    command: bool,
    /// Loops
    #[arg(short, long, action)]
    loops: bool,
    /// Batch
    #[arg(short, long, action)]
    batch: bool,
    #[arg(default_value = ".")]
    path: PathBuf,
}

pub fn list_options(args: CliArgs) -> Result<(), String> {
    let paths = if args.favorite {
        input::list_files(&args.path.join("favorites"))?
    } else if args.batch {
        input::list_files(&args.path.join("batch"))?
    } else if args.loops {
        input::list_files(&args.path.join("loops"))?
    } else if args.command {
        input::list_files(&args.path.join("commands"))?
    } else {
        input::list_files(&args.path.join("history"))?
    };
    for p in paths {
        println!("{}", p.unwrap().file_name().into_string().unwrap());
    }
    Ok(())
}
