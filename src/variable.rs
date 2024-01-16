use anyhow::{Context, Error};
use clap::{ArgGroup, Args, ValueHint};
use colored::Colorize;
use itertools::Itertools;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::{read_dir, File};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use string_template_plus::{Render, RenderOptions, Template, TemplatePart};

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
    /// Use it for generated files as it'll remove the comments. You
    /// can make a syntax "filename::variable=value" work, by using a
    /// template string instead of a file. The template can have
    /// placeholders {1},{2}, etc for the part that are separated by
    /// `::` and are before the `variable=value` pair. In the example
    /// "filename::variable=value", the {1} will be replaced with
    /// filename.
    #[arg(short, long, value_hint = ValueHint::FilePath, value_parser=Template::parse_template)]
    update: Option<Template>,
    #[arg(default_value = ".", value_hint=ValueHint::DirPath)]
    path: PathBuf,
}

pub fn input_lines(
    filename: &PathBuf,
    renumber: Option<usize>,
) -> Result<Vec<(usize, String)>, Error> {
    let file = match File::open(&filename) {
        Ok(f) => f,
        Err(e) => {
            return Err(Error::msg(format!(
                "Couldn't open input file: {:?}\n{:?}",
                &filename, e
            )))
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

pub fn matching_lines(
    filename: &PathBuf,
    patterns: &Vec<String>,
    any: bool,
) -> Result<Vec<(usize, String)>, Error> {
    let lines = input_lines(filename, None)?;
    let mut matching_lines: Vec<(usize, String)> = Vec::new();
    for (i, line) in &lines {
        if any {
            for pat in patterns {
                if line.contains(pat) {
                    let line = line.replace(&*pat, &pat.reversed().to_string());
                    matching_lines.push((*i, line));
                }
            }
        } else {
            if patterns.iter().all(|p| line.contains(&*p)) {
                let mut line = line.clone();
                patterns.iter().for_each(|p| {
                    line = line.replace(&*p, &p.reversed().to_string());
                });

                matching_lines.push((*i, line));
            }
        }
    }
    Ok(matching_lines)
}

// todo: make it into generic function?
pub fn read_inputs_set<'a>(
    enum_lines: &'a Vec<(usize, String)>,
    input_map: &mut HashSet<&'a str>,
) -> Result<(), Error> {
    for (i, line) in enum_lines {
        let split_data = line
            .split_once("=")
            .context(format!("Invalid Line# {}: \"{}\"", i, line))?;
        input_map.insert(split_data.0);
    }
    Ok(())
}

pub fn read_inputs_set_from_commands<'a>(
    enum_lines: &'a Vec<(usize, String)>,
    input_map: &mut HashSet<&'a str>,
) -> Result<(), Error> {
    for (_, line) in enum_lines {
        let templ = Template::parse_template(line)?;
        templ
            .parts()
            .into_iter()
            .filter_map(|p| {
                if let TemplatePart::Var(v, _) = p {
                    Some(v)
                } else {
                    None
                }
            })
            .for_each(|v| {
                let ind = line.find(v).expect("Variable should be in the template");
                input_map.insert(&line[ind..(ind + v.len())]);
            });
    }
    Ok(())
}

pub fn read_inputs(
    enum_lines: &Vec<(usize, String)>,
    input_map: &mut HashMap<String, String>,
) -> Result<(), Error> {
    for (i, line) in enum_lines {
        let split_data = line
            .split_once("=")
            .context(format!("Invalid Line# {}: \"{}\"", i, line))?;
        input_map.insert(split_data.0.to_string(), split_data.1.to_string());
    }
    Ok(())
}

pub fn list_files_sorted(filename: &PathBuf) -> Result<std::vec::IntoIter<PathBuf>, Error> {
    let files = read_dir(&filename)?;
    return Ok(files
        .map(|f| -> PathBuf { f.unwrap().path().to_owned() })
        .sorted());
}

pub fn list_files_sorted_recursive(filename: &PathBuf) -> Result<Vec<PathBuf>, Error> {
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

pub fn list_filenames(dirpath: &PathBuf) -> Result<Vec<String>, Error> {
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

pub fn list_anek_filenames(dirpath: &PathBuf) -> Result<Vec<String>, Error> {
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

pub fn loop_inputs(dirname: &PathBuf) -> Result<Vec<Vec<(String, usize, String)>>, Error> {
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

fn print_variable_info(name: &str, path: &PathBuf, details: bool) -> Result<(), Error> {
    let file = File::open(path)?;
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

fn print_update(filename: &str, variable: &str, old: Option<&str>, new: &str) {
    if let Some(old) = old {
        if old == new {
            return;
        }
        let mut i: usize = 0;
        for (o, n) in old.chars().zip(new.chars()) {
            if o != n {
                break;
            }
            i += 1;
        }

        println!(
            "{0}:: {1}: {2}{3} -> {2}{4}",
            filename,
            variable,
            &old[..i],
            &old[i..].on_red(),
            &new[i..].on_green()
        );
    } else {
        println!("{}:: {}: {}", filename, variable, new);
    }
}

fn update_file(file_s: &str, var_line: &str) -> Result<(), Error> {
    let file = PathBuf::from(file_s);
    if let Some((k, v)) = var_line.split_once("=") {
        let mut variables: HashMap<String, String> = if !file.exists() {
            HashMap::new()
        } else if file.is_file() {
            let lines = input_lines(&file, None)?;
            let mut vars: HashMap<String, String> = HashMap::new();
            read_inputs(&lines, &mut vars)?;
            vars.iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect()
        } else {
            return Err(Error::msg("File is not an anek file"));
        };

        let (k, v) = (k.trim(), v.trim());
        print_update(file_s, k, variables.get(k).map(|s| s.as_str()), v);
        variables.insert(k.to_string(), v.to_string());
        let fp = std::fs::File::create(&file)?;
        let mut writer = BufWriter::new(fp);
        for k in variables.keys().sorted() {
            writeln!(writer, "{}={}", k, variables[k])?;
        }
    }

    Ok(())
}

fn update_from_stdin(file_templ: &Template) -> Result<(), Error> {
    eprintln!("Waiting for input...");

    let mut input = String::new();
    let mut filename = String::new();
    let file_notempl = file_templ.lit();
    let mut filerenderops = RenderOptions::default();
    loop {
        input.clear();
        let n = io::stdin().read_line(&mut input)?;
        if n == 0 {
            break;
        }
        if let Some((fstr, vars)) = input.rsplit_once("::") {
            if file_notempl.is_some() {
                return Err(Error::msg("The file provided is not a template"));
            }
            filerenderops.variables.clear();
            fstr.split("::").enumerate().for_each(|(i, v)| {
                filerenderops
                    .variables
                    .insert((i + 1).to_string(), v.to_string());
            });
            filename = file_templ.render(&filerenderops)?;
            update_file(&filename, vars)?;
        } else {
            if let Some(file) = &file_notempl {
                update_file(file, &input)?;
            } else if !filename.is_empty() {
                update_file(&filename, &input)?;
            } else {
                return Err(Error::msg("No arguments for the file template"));
            }
        }
    }
    Ok(())
}

pub fn compact_lines_from_anek_file(
    filenames: &Vec<PathBuf>,
) -> Result<Vec<(usize, String)>, anyhow::Error> {
    let mut files: Vec<PathBuf> = Vec::new();
    for filename in filenames {
        if filename.is_dir() {
            files.extend(list_filenames(&filename)?.iter().map(|f| filename.join(f)));
        } else if !filename.exists() || filename.is_file() {
            let dot_d = filename.with_file_name(format!(
                "{}.d",
                filename.file_name().unwrap().to_string_lossy()
            ));
            if dot_d.exists() {
                if dot_d.is_dir() {
                    files.extend(list_filenames(&dot_d)?.iter().map(|f| dot_d.join(f)));
                } else if dot_d.is_file() {
                    files.push(dot_d);
                }
            }
            if filename.exists() {
                files.push(filename.clone());
            }
        } else {
            return Err(Error::msg(format!(
                "Path {:?} is neither a directory nor a file",
                filename
            )));
        }
    }
    let lines = files
        .iter()
        .map(|file| input_lines(&file, None))
        .collect::<Result<Vec<Vec<(usize, String)>>, Error>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<(usize, String)>>();
    Ok(lines)
}

pub fn run_command(args: CliArgs, anek_dir: AnekDirectory) -> Result<(), Error> {
    let mut vars: HashSet<&str> = HashSet::new();
    let inp_lines: Vec<Vec<(usize, String)>>;
    let cmd_lines: Vec<Vec<(usize, String)>>;
    if args.scan_inputs {
        let files =
            list_files_sorted_recursive(&anek_dir.get_directory(&AnekDirectoryType::Inputs))?;

        inp_lines = files
            .iter()
            .map(|file| input_lines(&file, None))
            .collect::<Result<Vec<Vec<(usize, String)>>, Error>>()?;
        inp_lines
            .iter()
            .map(|lns| read_inputs_set(lns, &mut vars))
            .collect::<Result<(), Error>>()?;
    }

    if args.scan_commands {
        let files =
            list_files_sorted_recursive(&anek_dir.get_directory(&AnekDirectoryType::Commands))?;
        cmd_lines = files
            .iter()
            .map(|file| input_lines(&file, None))
            .collect::<Result<Vec<Vec<(usize, String)>>, Error>>()?;
        cmd_lines
            .iter()
            .map(|lines| read_inputs_set_from_commands(lines, &mut vars))
            .collect::<Result<(), Error>>()?;
    }
    for var in vars {
        let var_file = anek_dir.get_file(&AnekDirectoryType::Variables, &var);
        if !var_file.exists() {
            println!("{}: {}", "New".red().bold(), var);
            if args.add {
                File::create(var_file)?;
            }
        } else if !var_file.is_file() {
            return Err(Error::msg(format!("{:?} is not a file", var_file)));
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
