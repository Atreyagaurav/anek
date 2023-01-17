use clap::{ArgGroup, Args, ValueHint};
use colored::Colorize;
use itertools::Itertools;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::{read_dir, File};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use crate::dtypes::{AnekDirectory, AnekDirectoryType};

#[derive(Args)]
#[command(group = ArgGroup::new("list_info").required(false).multiple(false))]
pub struct CliArgs {
    /// Scan the input files for variables
    ///
    /// If scanned variables are not listed, then it'll add a empty file
    /// with that name inside .anek/variables/
    #[arg(short, long, action)]
    scan: bool,
    /// List the variables with short description
    #[arg(short, long, group = "list_info", action)]
    list: bool,
    /// List the variables with long description (assumes --list)
    #[arg(short, long, group = "list_info", action)]
    details: bool,
    /// Gives long description of the variable
    #[arg(short, long, group = "list_info", value_hint = ValueHint::Other, value_name="VAR")]
    info: Option<String>,
    #[arg(default_value = ".", value_hint=ValueHint::DirPath)]
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
        .map(|(i, l)| (i + 1, l.unwrap()))
        .filter(|(_, l)| l.trim() != "" && !l.trim().starts_with("#"))
        .collect();
    Ok(lines)
}

// todo: make it into generic function?
pub fn read_inputs_set<'a>(
    enum_lines: &'a Vec<(usize, String)>,
    input_map: &mut HashSet<&'a str>,
) -> Result<(), String> {
    for (i, line) in enum_lines {
        let mut split_data = line.split("=");
        input_map.insert(match split_data.next() {
            Some(d) => d,
            None => return Err(format!("Invalid Line# {}: \"{}\"", i, line)),
        });
    }
    Ok(())
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

pub fn read_file_full(path: &PathBuf) -> Result<String, String> {
    let lines = input_lines(&path)?;
    let content = lines
        .iter()
        .map(|(_, line)| line.to_string())
        .collect::<Vec<String>>()
        .join("\n");
    return Ok(content);
}

pub fn list_files_sorted(filename: &PathBuf) -> Result<std::vec::IntoIter<PathBuf>, String> {
    let files = match read_dir(&filename) {
        Ok(l) => l,
        Err(e) => return Err(format!("Couldn't open directory: {:?}\n{:?}", &filename, e)),
    };
    return Ok(files
        .map(|f| -> PathBuf { f.unwrap().path().to_owned() })
        .sorted());
}

pub fn list_files_sorted_recursive(filename: &PathBuf) -> Result<Vec<PathBuf>, String> {
    let mut file_list: Vec<PathBuf> = Vec::new();
    let mut list_dir: VecDeque<PathBuf> = VecDeque::from(vec![filename.clone()]);

    while !list_dir.is_empty() {
        let file = list_dir.pop_front().unwrap();
        if file.is_file() {
            file_list.push(file.clone());
        } else if file.is_dir() {
            let files = list_files_sorted(&file)?;
            list_dir.extend(files);
        }
    }
    return Ok(file_list);
}

pub fn list_filenames(dirpath: &PathBuf) -> Result<Vec<String>, String> {
    let dirpath_full = dirpath.to_str().unwrap();
    let filenames: Vec<String> = list_files_sorted_recursive(dirpath)?
        .iter()
        .map(|file| {
            let fullpath = file.to_str().unwrap().to_string();
            let relpath = fullpath
                .strip_prefix(dirpath_full)
                .unwrap()
                .trim_start_matches("/");
            relpath.to_string()
        })
        .collect();
    Ok(filenames)
}

pub fn loop_inputs(dirname: &PathBuf) -> Result<Vec<Vec<(String, usize, String)>>, String> {
    let input_files = list_files_sorted(&dirname)?;
    let mut input_values: Vec<Vec<(String, usize, String)>> = Vec::new();
    for file in input_files {
        let filename = file.file_name().unwrap().to_str().unwrap().to_string();
        let lines = input_lines(&file)?;
        if lines.len() == 0 {
            continue;
        }
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

fn print_variable_info(name: &str, path: &PathBuf, details: bool) -> Result<(), String> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => return Err(format!("Couldn't open input file: {:?}\n{:?}", &path, e)),
    };
    print!("{} {:10}: ", "⇒".bright_blue(), name.green());

    let mut reader_lines = BufReader::new(file).lines();
    if let Some(l) = reader_lines.next() {
        println!("{}", l.unwrap());
        if details {
            for line in reader_lines {
                println!("    {}", line.unwrap());
            }
        }
    } else {
        println!("");
    }

    Ok(())
}

pub fn run_command(args: CliArgs) -> Result<(), String> {
    let anek_dir = AnekDirectory::from(&args.path);
    if args.scan {
        let files =
            list_files_sorted_recursive(&anek_dir.get_directory(&AnekDirectoryType::Inputs))?;
        let mut vars: HashSet<&str> = HashSet::new();
        let lines = files
            .iter()
            .map(|file| input_lines(&file))
            .collect::<Result<Vec<Vec<(usize, String)>>, String>>()?;
        lines
            .iter()
            .map(|lns| read_inputs_set(lns, &mut vars))
            .collect::<Result<(), String>>()?;
        for var in vars {
            let var_file = anek_dir.get_file(&AnekDirectoryType::Variables, &var);
            if !var_file.exists() {
                println!("{}: {}", "New Input".red().bold(), var);
                File::create(var_file).map_err(|e| e.to_string())?;
            } else if !var_file.is_file() {
                return Err(format!("{:?} is not a file", var_file));
            }
        }
    }
    if args.list || args.details {
        let files = list_files_sorted(&anek_dir.get_directory(&AnekDirectoryType::Variables))?;

        for file in files {
            if file.is_file() {
                let filename = file.file_name().unwrap().to_str().unwrap().to_string();
                print_variable_info(&filename, &file, args.details)?;
            }
        }
    } else if let Some(name) = args.info {
        print_variable_info(
            &name,
            &anek_dir.get_file(&AnekDirectoryType::Variables, &name),
            true,
        )?;
    }
    Ok(())
}