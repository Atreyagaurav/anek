use clap::{Args, ValueHint};
use std::path::PathBuf;

use crate::dtypes::AnekDirectory;

#[derive(Args)]
pub struct CliArgs {
    #[arg(default_value = ".", value_hint = ValueHint::DirPath)]
    path: PathBuf,
}

pub fn cmd(args: CliArgs) -> Result<(), String> {
    let anek_dir = AnekDirectory::from(&args.path)?;
    println!("{:?}", anek_dir.root);
    Ok(())
}
