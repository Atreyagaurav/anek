use clap::Args;
use itertools::Itertools;
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
        .collect();
    Ok(lines)
}

pub fn read_inputs<'a>(
    enum_lines: &'a Vec<(usize, String)>,
    input_map: &mut HashMap<&'a str, &'a str>,
) -> Result<(), String> {
    for (_i, line) in enum_lines {
        if line.trim() != "" {
            let mut split_data = line.split("=");
            input_map.insert(
                split_data.next().unwrap_or(""),
                split_data.next().unwrap_or(""),
            );
        }
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
        let lines = input_lines(&file.path())?;
        input_values.push(
            lines
                .iter()
                .map(|l| {
                    (
                        file.file_name().to_str().unwrap().to_string(),
                        l.0,
                        l.1.clone(),
                    )
                })
                .collect(),
        );
    }

    let permutations = input_values.iter().multi_cartesian_product();
    let mut final_values: Vec<Vec<(String, usize, String)>> = Vec::new();
    for inputs in permutations {
        final_values.push(
            inputs
                .iter()
                .map(|&t| (t.0.clone(), t.1, t.2.clone())) // not good, find a way to safely transfer values
                .collect(),
        );
    }
    Ok(final_values)
}
