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
    #[arg(default_value = ".", value_hint=ValueHint::DirPath)]
    path: PathBuf,
}

pub fn edit_file(args: CliArgs) -> Result<(), Error> {
    let filepath = AnekDirectory::from(&args.path)?.root.join(args.anek_file);
    let command = format!("{} {:?}", env::var("EDITOR").unwrap(), filepath);
    println!("{}", command);
    Exec::shell(command).join()?;
    Ok(())
}
