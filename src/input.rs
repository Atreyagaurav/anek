use clap::Args;
use std::collections::HashMap;
use std::fs::{read_dir, File, ReadDir};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

#[derive(Args)]
pub struct CliArgs {
    #[arg(default_value = ".")]
    path: PathBuf,
}

pub fn input_lines(filename: &PathBuf) -> Result<Vec<(usize, String)>, String> {
    let file = match File::open(&filename) {
        Ok(f) => f,
        Err(e) => {
            return Err(format!(
                "Couldn't open input file: {:?}\n{:?}",
                &filename, e
            ))
        }
    };
    let reader_lines = BufReader::new(file).lines();
    let lines = reader_lines
        .enumerate()
        .map(|il| (il.0, il.1.unwrap()))
        .filter(|il| il.1.trim() != "" && !il.1.trim().starts_with("#"))
        .collect();
    Ok(lines)
}

pub fn read_inputs<'a>(
    enum_lines: &'a Vec<(usize, String)>,
    input_map: &mut HashMap<&'a str, &'a str>,
) -> Result<(), String> {
    for (i, line) in enum_lines {
        let mut split_data = line.split("=");
        input_map.insert(
            match split_data.next() {
                Some(d) => d,
                None => return Err(format!("Invalid Line# {}: \"{}\"", i, line)),
            },
            match split_data.next() {
                Some(d) => d,
                None => return Err(format!("Invalid Line# {}: \"{}\"", i, line)),
            },
        );
    }
    Ok(())
}

pub fn list_files(filename: &PathBuf) -> Result<ReadDir, String> {
    let files = match read_dir(&filename) {
        Ok(l) => l,
        Err(e) => return Err(format!("Couldn't open directory: {:?}\n{:?}", &filename, e)),
    };
    Ok(files)
}

pub fn loop_inputs(dirname: &PathBuf) -> Result<Vec<Vec<(String, usize, String)>>, String> {
    let input_files = list_files(&dirname)?;
    let mut input_values: Vec<Vec<(String, usize, String)>> = Vec::new();
    for file in input_files {
        let file = file.unwrap();
        let filename = file.file_name().to_str().unwrap().to_string();
        let lines = input_lines(&file.path())?;
        input_values.push(
            lines
                .iter()
                .enumerate()
                .map(|(i, l)| (filename.clone(), i, l.1.clone()))
                .collect(),
        );
    }
    Ok(input_values)
}
