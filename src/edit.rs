use clap::{Args, ValueHint};
use std::env;
use std::path::PathBuf;
use subprocess::Exec;

use crate::dtypes::AnekDirectory;

#[derive(Args)]
pub struct CliArgs {
    /// Show the contents of a file inside .anek
    #[arg(short, long)]
    echo: bool,
    /// The file inside .anek
    ///
    /// Use relative path starting from .anek, the possible paths are
    /// the same ones from `anek list` command output
    #[arg(value_hint = ValueHint::Other)]
    anek_file: String,
    #[arg(default_value = ".", value_hint=ValueHint::DirPath)]
    path: PathBuf,
}

pub fn edit_file(args: CliArgs) -> Result<(), String> {
    let filepath = AnekDirectory::from(&args.path)?.root.join(args.anek_file);
    if args.echo {
        let contents = match std::fs::read_to_string(filepath) {
            Ok(c) => c,
            Err(e) => return Err(e.to_string()),
        };
        print!("{}", contents);
    } else {
        let command = format!("{} {:?}", env::var("EDITOR").unwrap(), filepath);
        println!("{}", command);
        if let Err(e) = Exec::shell(command).join() {
            return Err(format!("Can not edit file:{:?}\n {}", filepath, e));
        };
    }
    Ok(())
}
