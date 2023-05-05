use clap::{ArgGroup, Args, ValueHint};
use colored::Colorize;
use itertools::Itertools;
use lazy_static::lazy_static;
use new_string_template::template::Template;
use regex::Regex;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::{read_dir, File};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

use crate::dtypes::{AnekDirectory, AnekDirectoryType};

#[derive(Args)]
#[command(group = ArgGroup::new("list_info").required(false).multiple(false))]
#[command(group = ArgGroup::new("scan").required(false).multiple(true))]
pub struct CliArgs {
    /// Scan the input files for variables
    ///
    /// Scan the input files for any variables that are not in the
    /// variables directory
    #[arg(short, long, group = "scan", action)]
    scan_inputs: bool,
    /// Scan the command files for variables
    ///
    /// Scan the command files for any variables that are not in the
    /// variables directory
    #[arg(short = 'S', long, group = "scan", action)]
    scan_commands: bool,
    /// Add files for the new variables detected by scan
    ///
    /// It'll add a empty file with that name inside .anek/variables/
    #[arg(short, long, action, requires = "scan")]
    add: bool,
    /// List the variables with short description
    #[arg(short, long, group = "list_info", action)]
    list: bool,
    /// List the variables with long description
    #[arg(short, long, group = "list_info", action)]
    details: bool,
    /// Gives long description of the variable
    #[arg(short, long, group = "list_info", value_hint = ValueHint::Other, value_name="VAR")]
    info: Option<String>,
    /// Update variables read from stdin in the given file
    ///
    /// Use it for generated files as it'll remove the comments
    #[arg(short, long, value_hint = ValueHint::FilePath)]
    update: Option<PathBuf>,
    #[arg(default_value = ".", value_hint=ValueHint::DirPath)]
    path: PathBuf,
}

pub static OPTIONAL_RENDER_CHAR: char = '?';
pub static LITERAL_VALUE_QUOTE_CHAR: char = '"';

pub fn input_lines(
    filename: &PathBuf,
    renumber: Option<usize>,
) -> Result<Vec<(usize, String)>, String> {
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
    let lines = if let Some(num) = renumber {
        reader_lines
            .map(|l| l.unwrap().trim().to_string())
            .filter(|l| l != "" && !l.starts_with("#"))
            .enumerate()
            .map(|(i, l)| (i + num, l))
            .collect()
    } else {
        reader_lines
            .enumerate()
            .map(|(i, l)| (i + 1, l.unwrap().trim().to_string()))
            .filter(|(_, l)| l != "" && !l.starts_with("#"))
            .collect()
    };
    Ok(lines)
}

// todo: make it into generic function?
pub fn read_inputs_set<'a>(
    enum_lines: &'a Vec<(usize, String)>,
    input_map: &mut HashSet<&'a str>,
) -> Result<(), String> {
    for (i, line) in enum_lines {
        let split_data = match line.split_once("=") {
            Some(d) => d,
            None => return Err(format!("Invalid Line# {}: \"{}\"", i, line)),
        };
        input_map.insert(split_data.0);
    }
    Ok(())
}

pub fn render_template(
    templ: &Template,
    input_map: &HashMap<&str, &str>,
) -> Result<String, String> {
    let rend = match templ.render(&input_map) {
        Ok(c) => c,
        Err(_) => {
            let template_str: String = templ.render_nofail(&input_map);
            lazy_static! {
                static ref CMD_RE: Regex = Regex::new(&format!(
                    "{}{}{}{}{}",
                    r"\{((?:[a-zA-Z0-9_",
                    LITERAL_VALUE_QUOTE_CHAR.to_string(),
                    "-]+[",
                    OPTIONAL_RENDER_CHAR.to_string(),
                    r"]?)+)\}"
                ))
                .unwrap();
            }
            let mut new_map = input_map.clone();

            for cap in CMD_RE.captures_iter(&template_str) {
                let cg = cap.get(1).unwrap();
                let cap_slice = &template_str[cg.start()..cg.end()];
                for csg in cap_slice.split(OPTIONAL_RENDER_CHAR) {
                    // replace the template with ? with first valid
                    // value or first literal string
                    if let Some(val) = input_map.get(csg) {
                        new_map.insert(cap_slice, val);
                        break;
                    } else if csg.starts_with(LITERAL_VALUE_QUOTE_CHAR)
                        && csg.ends_with(LITERAL_VALUE_QUOTE_CHAR)
                    {
                        new_map.insert(cap_slice, &csg[1..(csg.len() - 1)]);
                        break;
                    }
                }
            }

            match templ.render(&new_map) {
                Ok(c) => c,
                Err(e) => return Err(e.to_string()),
            }
        }
    };
    Ok(rend)
}

pub fn read_inputs_set_from_commands<'a>(
    enum_lines: &'a Vec<(usize, String)>,
    input_map: &mut HashSet<&'a str>,
) -> Result<(), String> {
    lazy_static! {
        static ref CMD_RE: Regex = Regex::new(r"\{((?:\w+[|]?)+)\}").unwrap();
    }
    for (_, line) in enum_lines {
        for cap in CMD_RE.captures_iter(line) {
            let cg = cap.get(1).unwrap();
            let cap_slice = &line[cg.start()..cg.end()];
            cap_slice.split(OPTIONAL_RENDER_CHAR).for_each(|cg| {
                input_map.insert(cg);
            })
        }
    }
    Ok(())
}

pub fn read_inputs<'a>(
    enum_lines: &'a Vec<(usize, String)>,
    input_map: &mut HashMap<&'a str, &'a str>,
) -> Result<(), String> {
    for (i, line) in enum_lines {
        let split_data = match line.split_once("=") {
            Some(d) => d,
            None => return Err(format!("Invalid Line# {}: \"{}\"", i, line)),
        };
        input_map.insert(split_data.0, split_data.1);
    }
    Ok(())
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

pub fn list_anek_filenames(dirpath: &PathBuf) -> Result<Vec<String>, String> {
    let dirpath_full = dirpath.to_str().unwrap();
    let mut file_list: HashSet<PathBuf> = HashSet::new();
    let mut list_dir: VecDeque<PathBuf> = VecDeque::from(vec![dirpath.clone()]);

    while !list_dir.is_empty() {
        let file = list_dir.pop_front().unwrap();
        if file.is_file() {
            file_list.insert(file.clone());
        } else if file.is_dir() {
            if file.file_name().unwrap().to_string_lossy().ends_with(".d") {
                file_list.insert(file.with_extension(""));
            } else {
                let files = list_files_sorted(&file)?;
                list_dir.extend(files);
            }
        }
    }
    let filenames: Vec<String> = file_list
        .into_iter()
        .map(|file| {
            let fullpath = file.to_str().unwrap().to_string();
            let relpath = fullpath
                .strip_prefix(dirpath_full)
                .unwrap()
                .trim_start_matches("/");
            relpath.to_string()
        })
        .sorted()
        .collect();
    Ok(filenames)
}

pub fn loop_inputs(dirname: &PathBuf) -> Result<Vec<Vec<(String, usize, String)>>, String> {
    let input_files = list_files_sorted(&dirname)?;
    let mut input_values: Vec<Vec<(String, usize, String)>> = Vec::new();
    for file in input_files {
        let filename = file.file_name().unwrap().to_str().unwrap().to_string();
        let lines = input_lines(&file, Some(1))?;
        if lines.len() == 0 {
            continue;
        }
        input_values.push(
            lines
                .into_iter()
                .map(|(i, l)| (filename.clone(), i, l))
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

fn update_from_stdin(file: &PathBuf) -> Result<(), String> {
    let mut variables: HashMap<String, String> = if !file.exists() {
        HashMap::new()
    } else if file.is_file() {
        let lines = input_lines(&file, None)?;
        let mut vars: HashMap<&str, &str> = HashMap::new();
        read_inputs(&lines, &mut vars)?;
        vars.iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    } else {
        return Err("File is not an anek file".into());
    };

    for (k, v) in variables.iter() {
        eprintln!("{}={}", k, v);
    }
    eprintln!("Waiting for input...");

    let mut input = String::new();
    let mut modified = false;
    loop {
        input.clear();
        let n = io::stdin()
            .read_line(&mut input)
            .map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        if let Some((k, v)) = input.split_once("=") {
            let (k, v) = (k.trim(), v.trim());
            if variables.contains_key(k) {
                if variables[k] == v {
                    continue;
                }
                println!("{}: {} -> {}", k, variables[k], v);
            }
            variables.insert(k.to_string(), v.to_string());
            modified = true;
        }
    }
    if modified {
        let fp = std::fs::File::create(&file).map_err(|e| e.to_string())?;
        let mut writer = BufWriter::new(fp);
        for (k, v) in variables {
            writeln!(writer, "{}={}", k, v).map_err(|e| e.to_string())?;
        }
        eprintln!("Updated {:?}", file)
    }
    Ok(())
}

pub fn run_command(args: CliArgs) -> Result<(), String> {
    let anek_dir = AnekDirectory::from(&args.path)?;
    let mut vars: HashSet<&str> = HashSet::new();
    let inp_lines: Vec<Vec<(usize, String)>>;
    let cmd_lines: Vec<Vec<(usize, String)>>;
    if args.scan_inputs {
        let files =
            list_files_sorted_recursive(&anek_dir.get_directory(&AnekDirectoryType::Inputs))?;

        inp_lines = files
            .iter()
            .map(|file| input_lines(&file, None))
            .collect::<Result<Vec<Vec<(usize, String)>>, String>>()?;
        inp_lines
            .iter()
            .map(|lns| read_inputs_set(lns, &mut vars))
            .collect::<Result<(), String>>()?;
    }

    if args.scan_commands {
        let files =
            list_files_sorted_recursive(&anek_dir.get_directory(&AnekDirectoryType::Commands))?;
        cmd_lines = files
            .iter()
            .map(|file| input_lines(&file, None))
            .collect::<Result<Vec<Vec<(usize, String)>>, String>>()?;
        cmd_lines
            .iter()
            .map(|lines| read_inputs_set_from_commands(lines, &mut vars))
            .collect::<Result<(), String>>()?;
    }
    for var in vars {
        let var_file = anek_dir.get_file(&AnekDirectoryType::Variables, &var);
        if !var_file.exists() {
            println!("{}: {}", "New".red().bold(), var);
            if args.add {
                File::create(var_file).map_err(|e| e.to_string())?;
            }
        } else if !var_file.is_file() {
            return Err(format!("{:?} is not a file", var_file));
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

    if let Some(file) = args.update {
        update_from_stdin(&file)?;
    }
    Ok(())
}
