use anyhow::Error;
use clap::{Args, ValueHint};
use std::path::PathBuf;

use crate::dtypes::AnekDirectory;

#[derive(Args)]
pub struct CliArgs {
    #[arg(value_hint = ValueHint::DirPath)]
    path: Option<PathBuf>,
}

pub fn cmd(args: CliArgs, path: PathBuf) -> Result<(), Error> {
    let anek_dir = AnekDirectory::from(&args.path.unwrap_or(path))?;
    println!("{:?}", anek_dir.proj_root());
    Ok(())
}
