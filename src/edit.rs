use anyhow::Error;
use clap::{Args, ValueHint};
use std::env;
use std::path::PathBuf;
use subprocess::Exec;

use crate::dtypes::AnekDirectory;

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

pub fn edit_file(args: CliArgs, path: PathBuf) -> Result<(), Error> {
    let anek_dir = AnekDirectory::from(&args.path.unwrap_or(path))?;
    let filepath = anek_dir.get_file_global(&args.anek_file);
    let command = format!("{} {:?}", env::var("EDITOR").unwrap(), filepath);
    println!("{}", command);
    Exec::shell(command).cwd(anek_dir.proj_root()).join()?;
    Ok(())
}
