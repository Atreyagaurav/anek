use clap::{Args, ValueHint};
use std::fs::{create_dir, File};
use std::path::PathBuf;

use crate::dtypes::{anekdirtype_iter, AnekDirectory, AnekDirectoryType};

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

pub fn new_config(args: CliArgs) -> Result<(), String> {
    let filepath = AnekDirectory::from(&args.path);
    if filepath.root.exists() {
        if filepath.root.is_dir() {
            return Err(format!("{:?} already exists", filepath.root));
        } else {
            return Err(
                format!("{:?} file exists which is not a anek configuration, pleae remove that before continuing", filepath.root),
            );
        }
    } else {
        create_dir(&filepath.root).map_err(|e| e.to_string())?;

        for adt in anekdirtype_iter() {
            create_dir(filepath.get_directory(adt)).map_err(|e| e.to_string())?;
        }
        for var in args.variables {
            File::create(filepath.get_file(&AnekDirectoryType::Variables, &var))
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
