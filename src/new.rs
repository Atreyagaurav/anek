use clap::{Args, ValueHint};
use std::fs::File;
use std::path::PathBuf;

use crate::dtypes::{AnekDirectory, AnekDirectoryType};

#[derive(Args)]
pub struct CliArgs {
    /// Variables files to be constructed
    ///
    /// Will make empty files with names of the variables provided,
    /// use comma to separate multiple variable names. You can use the
    /// scan function later to add the variable files automatically
    /// once you have input files with some variables.
    #[arg(short, long, value_delimiter = ',')]
    variables: Vec<String>,
    #[arg(default_value = ".", value_hint=ValueHint::DirPath)]
    path: PathBuf,
}

pub fn new_config(args: CliArgs) -> Result<(), anyhow::Error> {
    let ad = AnekDirectory::new(&args.path)?;

    for var in args.variables {
        File::create(ad.get_file(&AnekDirectoryType::Variables, &var))?;
    }
    Ok(())
}
