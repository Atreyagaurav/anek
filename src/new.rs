use clap::{Args, ValueHint};
use std::fs::{create_dir, File};
use std::path::PathBuf;

#[derive(Args)]
pub struct CliArgs {
    /// Input files to be constructed
    #[arg(short, long, value_delimiter = ',')]
    inputs: Vec<String>,
    #[arg(default_value = ".", value_hint=ValueHint::DirPath)]
    path: PathBuf,
}

pub fn new_config(args: CliArgs) -> Result<(), String> {
    let filepath = args.path.join(".anek");
    if filepath.exists() {
        if filepath.is_dir() {
            return Err(format!("{:?} already exists", filepath));
        } else {
            return Err(format!("{:?} file exists which is not a anek configuration, pleae remove that before continuing", filepath));
        }
    } else {
        create_dir(&filepath).map_err(|e| e.to_string())?;
        // todo check spellings : make a enum for this
        create_dir(filepath.join("inputs")).map_err(|e| e.to_string())?;
        create_dir(filepath.join("favorites")).map_err(|e| e.to_string())?;
        create_dir(filepath.join("batch")).map_err(|e| e.to_string())?;
        create_dir(filepath.join("loop")).map_err(|e| e.to_string())?;
        create_dir(filepath.join("commands")).map_err(|e| e.to_string())?;
        create_dir(filepath.join("pipelines")).map_err(|e| e.to_string())?;
        create_dir(filepath.join("history")).map_err(|e| e.to_string())?;
        for inp in args.inputs {
            File::create(filepath.join("inputs").join(inp)).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
