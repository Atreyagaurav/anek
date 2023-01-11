use clap::{ArgGroup, Args, ValueHint};
use colored::Colorize;
use itertools::Itertools;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::{read_dir, File};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

#[derive(Args)]
#[command(group = ArgGroup::new("list_info").required(false).multiple(false))]
pub struct CliArgs {
    /// Scan the favorites files for input parameters
    ///
    /// If scanned inputs are not listed, then it'll add a empty file
    /// with that name inside .anek/inputs/
    #[arg(short, long, action)]
    scan: bool,
    /// List the inputs with short description
    #[arg(short, long, group = "list_info", action)]
    list: bool,
    /// List the inputs with long description (assumes --list)
    #[arg(short, long, group = "list_info", action)]
    details: bool,
    /// Gives long description of the input
    #[arg(short, long, group = "list_info", value_hint = ValueHint::Other, value_name="INPUT")]
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

fn print_input_info(name: &str, path: &PathBuf, details: bool) -> Result<(), String> {
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
    if args.scan {
        let files = list_files_sorted_recursive(&args.path.join(".anek/favorites"))?;
        let mut inputs: HashSet<&str> = HashSet::new();
        let lines = files
            .iter()
            .map(|file| input_lines(&file))
            .collect::<Result<Vec<Vec<(usize, String)>>, String>>()?;
        lines
            .iter()
            .map(|lns| read_inputs_set(lns, &mut inputs))
            .collect::<Result<(), String>>()?;
        for input in inputs {
            let input_file = args.path.join(".anek/inputs").join(input);
            if !input_file.exists() {
                println!("{}: {}", "New Input".red().bold(), input);
                File::create(input_file).map_err(|e| e.to_string())?;
            } else if !input_file.is_file() {
                return Err(format!("{:?} is not a file", input_file));
            }
        }
    }
    if args.list || args.details {
        let files = list_files_sorted(&args.path.join(".anek/inputs"))?;

        for file in files {
            let filename = file.file_name().unwrap().to_str().unwrap().to_string();
            print_input_info(&filename, &file, args.details)?;
        }
    } else if let Some(name) = args.info {
        print_input_info(&name, &args.path.join(".anek/inputs").join(&name), true)?;
    }
    Ok(())
}
