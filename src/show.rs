use anyhow::Error;
use clap::{Args, ValueHint};
use std::path::PathBuf;
use string_template_plus::{Render, Template};

use crate::dtypes::{AnekDirectory, AnekDirectoryType};

#[derive(Args)]
pub struct CliArgs {
    /// The file inside .anek
    ///
    /// Use relative path starting from .anek, the possible paths are
    /// the same ones from `anek list` command output
    #[arg(value_hint = ValueHint::Other)]
    anek_file: String,
    #[arg(value_hint=ValueHint::DirPath)]
    path: Option<PathBuf>,
}

pub fn show_file(args: CliArgs, path: PathBuf) -> Result<(), Error> {
    let filepath =
        AnekDirectory::from(&args.path.unwrap_or(path))?.get_file_global(&args.anek_file);

    let contents = std::fs::read_to_string(filepath)?;
    if args
        .anek_file
        .starts_with(AnekDirectoryType::Commands.dir_name())
    {
        let templ = Template::parse_template(&contents)?;
        templ.print();
        println!();
    } else {
        println!("{}", contents);
    }
    Ok(())
}
