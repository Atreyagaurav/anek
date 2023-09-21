use clap::{ArgGroup, Args, ValueHint};
use colored::Colorize;
use itertools::Itertools;
use new_string_template::template::Template;
use number_range::NumberRangeOptions;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use subprocess::Exec;

use crate::dtypes::{AnekDirectory, AnekDirectoryType};
use crate::export;
use crate::variable;

#[derive(Args)]
#[command(group = ArgGroup::new("action").required(true).multiple(false))]
#[command(group = ArgGroup::new("variables").required(false).multiple(false))]
pub struct CliArgs {
    /// command to run (from .anek/commands/)
    ///
    /// The command file saved will have the command template inside
    /// it. And it'll run that command (filling it with variables'
    /// values if there are some).
    #[arg(short, long, group="action", value_hint = ValueHint::Other)]
    command: Option<String>,
    /// Command template to run
    ///
    /// Command templates are bash commands with variables enclosed in
    /// curly braces. e.g. --command-template 'echo {mean}', will run
    /// echo with value of `mean` as the first argument.
    #[arg(short = 'C', long, group="action", value_hint = ValueHint::CommandName)]
    command_template: Option<String>,
    /// Run a pipeline
    ///
    /// Pipeline has a list of command that are run one after another,
    /// those command list has to be relative path from
    /// .anek/commands/, use `anek list -c` for possible commands.
    #[arg(short, long, group="action", value_hint = ValueHint::Other)]
    pipeline: Option<String>,
    /// Render the given file like command templates
    ///
    /// Renders a file. Just write the file anywhere in your path with
    /// templates for input variables in the same way you write
    /// command templates. It's useful for markdown files, as the
    /// curly braces syntax won't be used for anything else that
    /// way. Do be careful about that. And the program will replace
    /// those templates with their values when you run it with inputs.
    #[arg(short, long, group="action", value_hint = ValueHint::FilePath)]
    render: Option<PathBuf>,
    /// Render the given template like command templates
    ///
    /// Renders a template. Same as render but you pass template instead of file.
    #[arg(short='R', long, group="action", value_hint = ValueHint::Other)]
    render_template: Option<String>,
    /// Export the given variables
    #[arg(short='e', long,  value_delimiter=',', group="action", value_hint = ValueHint::Other)]
    export: Vec<String>,
    /// Export according to the format specified (csv,json,plain)
    #[arg(short='E', long, default_value="csv", value_hint = ValueHint::Other)]
    export_format: String,
    /// Select subset to run, works in batch and loop only
    ///
    /// Since batch and loop are just a list of inputs to run, you can
    /// select a subset of them to run. The selection string needs to
    /// be comma separated values of possitive integers, you can have
    /// range of values like: 1-5 to represent values from 1 to 5.
    #[arg(short, long, value_hint = ValueHint::Other)]
    select_inputs: Option<String>,
    /// Run from batch
    ///
    /// Batch file are list of input files that are run one after
    /// another on the same command template. The list of input files
    /// need to be relative to .anek/inputs/, run `anek list -i` for
    /// possible input files. If you provide more than a single batch
    /// file, their inputs will be combined and run one after another.
    #[arg(short, long, group="variables", value_delimiter=',', value_hint = ValueHint::Other)]
    batch: Vec<String>,
    /// Run commands by looping for the inputs
    ///
    /// Loops though the values of the input variables in the loop
    /// config and run the command templates on the combinations of
    /// those different variables.
    #[arg(short, long, group="variables", value_hint = ValueHint::Other)]
    r#loop: Option<String>,
    /// Run from inputs
    ///
    /// Use the variables' values defined in the inputs file to fill
    /// the tempalte and run it. You can specify a directory, and
    /// it'll use all the files inside to fill the template, in this
    /// case duplicate variable names will have only the last read
    /// value (alphabetically, then outside to inside recursively). If
    /// you give multiple input files then it'll read them in order,
    /// if they're directory then each directory will be read in the
    /// aforementioned way.
    #[arg(short, long, group="variables", value_delimiter=',', value_hint = ValueHint::Other)]
    input: Vec<String>,
    /// Print commands only so you can pipe it, assumes --demo
    ///
    /// This one will only print the commands without executing them
    /// or printing any other informations. You can pipe it to bash or
    /// gnu parallel or any other commands to run it, or process it
    /// further.
    #[arg(short = 'P', long)]
    pipable: bool,
    /// Demo only, don't run the actual command
    ///
    /// Does everything but skips running the command, you can make
    /// sure it's what you want to run before running it.
    #[arg(short, long)]
    demo: bool,
    /// Overwrite input variables
    ///
    /// Provide variables to be overwritten in the input config. If
    /// you have no input config the it'll be used to fill the
    /// template. Otherwise it'll be added to that, it'll replace if
    /// there are same variables with different names. In case of
    /// batch file, it'll replace it for each input file, and in case
    /// of loop, it does the same, but in addition, if the loop has
    /// multiple values for the variable that you are overwriting,
    /// then it'll no longer use any of those values to loop.
    #[arg(short, long, value_delimiter=',', value_hint = ValueHint::Other)]
    overwrite: Vec<String>,
    #[arg(default_value = ".", value_hint = ValueHint::DirPath)]
    path: PathBuf,
    /// Arguments to pass to the action template as ARG<N>
    ///
    /// The arguments passed here can be accessed as ARG1,ARG2,etc in
    /// the template.
    #[arg(num_args(0..), last(true), requires="action")]
    command_args: Vec<String>,
}

pub fn exec_on_inputfile(
    cmd: &Template,
    filenames: &Vec<PathBuf>,
    wd: &PathBuf,
    overwrite: &HashMap<&str, &str>,
    demo: bool,
    pipable: bool,
    name: &str,
    print_extras: (&str, &str),
) -> Result<(), String> {
    let mut files: Vec<PathBuf> = Vec::new();
    for filename in filenames {
        if filename.is_dir() {
            files.extend(
                variable::list_filenames(&filename)?
                    .iter()
                    .map(|f| filename.join(f)),
            );
        } else if !filename.exists() || filename.is_file() {
            let dot_d = filename.with_file_name(format!(
                "{}.d",
                filename.file_name().unwrap().to_string_lossy()
            ));
            if dot_d.exists() {
                if dot_d.is_dir() {
                    files.extend(
                        variable::list_filenames(&dot_d)?
                            .iter()
                            .map(|f| dot_d.join(f)),
                    );
                } else if dot_d.is_file() {
                    files.push(dot_d);
                }
            }
            if filename.exists() {
                files.push(filename.clone());
            }
        } else {
            return Err(format!(
                "Path {:?} is neither a directory nor a file",
                filename
            ));
        }
    }
    let mut input_map: HashMap<&str, &str> = HashMap::new();
    let lines = files
        .iter()
        .map(|file| variable::input_lines(&file, None))
        .collect::<Result<Vec<Vec<(usize, String)>>, String>>()?;
    lines
        .iter()
        .map(|l| variable::read_inputs(&l, &mut input_map))
        .collect::<Result<(), String>>()?;
    // render the metavariables in the overwrite
    let overwrite_meta: Vec<(&str, String)> = overwrite
        .iter()
        .map(|(k, v)| -> Result<(&str, String), String> {
            variable::render_template(&Template::new(v.to_string()), &input_map, wd)
                .and_then(|s| Ok((*k, s)))
                .map_err(|e| e.to_string())
        })
        .collect::<Result<Vec<(&str, String)>, String>>()?;
    for (k, v) in &overwrite_meta {
        input_map.insert(k, &v);
    }

    let command = variable::render_template(cmd, &input_map, wd)?;
    if !pipable {
        eprint!("{} ({}): ", "Command".bright_green(), name);
    }
    println!("{}{}{}", print_extras.0, command.trim(), print_extras.1);
    if !pipable {
        eprintln!("⇒");
    }
    if !(demo || pipable) {
        Exec::shell(command).cwd(&wd).join().unwrap();
    }
    Ok(())
}

pub fn exec_pipeline_on_inputfile(
    commands: &Vec<(String, String, Template)>,
    input_file: &Vec<PathBuf>,
    wd: &PathBuf,
    overwrite: &HashMap<&str, &str>,
    demo: bool,
    pipable: bool,
    print_extras: (&str, &str),
) -> Result<(), String> {
    for (name, _, command) in commands {
        exec_on_inputfile(
            &command,
            &input_file,
            &wd,
            &overwrite,
            demo,
            pipable,
            name,
            print_extras,
        )?;
    }
    Ok(())
}

pub fn exec_pipeline(
    commands: &Vec<(String, String, Template)>,
    wd: &PathBuf,
    inputs: &HashMap<&str, &str>,
    demo: bool,
    pipable: bool,
    print_extras: (&str, &str),
) -> Result<(), String> {
    for (name, _, command) in commands {
        let cmd = variable::render_template(command, &inputs, wd)?;
        if !pipable {
            eprint!("{} ({}): ", "Command".bright_green(), name);
        }
        println!("{}{}{}", print_extras.0, cmd, print_extras.1);
        if !pipable {
            eprintln!("⇒");
        }
        if !(demo || pipable) {
            Exec::shell(cmd)
                .cwd(&wd)
                .join()
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub fn run_command(args: CliArgs) -> Result<(), String> {
    let anek_dir = AnekDirectory::from(&args.path)?;
    let pipable = args.pipable
        || args.render.is_some()
        || args.render_template.is_some()
        || args.export.len() > 0;
    let filter_index: HashSet<usize> = if let Some(f) = args.select_inputs {
        NumberRangeOptions::default()
            .with_list_sep(',')
            .with_range_sep('-')
            .parse(&f)
            .map_err(|s| s.to_string())?
            .collect()
    } else {
        HashSet::new()
    };

    let pipeline_templates = if let Some(pipeline) = args.pipeline {
        variable::input_lines(
            &anek_dir.get_file(&AnekDirectoryType::Pipelines, &pipeline),
            None,
        )?
        .iter()
        .map(
            |(_, command)| -> Result<(String, String, Template), String> {
                match fs::read_to_string(&anek_dir.get_file(&AnekDirectoryType::Commands, &command))
                {
                    Ok(s) => Ok((command.clone(), s.clone(), Template::new(s.trim()))),
                    Err(e) => Err(e.to_string()),
                }
            },
        )
        .collect::<Result<Vec<(String, String, Template)>, String>>()
        .unwrap()
    } else if let Some(command) = args.command {
        let command_content =
            fs::read_to_string(&anek_dir.get_file(&AnekDirectoryType::Commands, &command))
                .map_err(|e| e.to_string())?;
        vec![(
            command,
            command_content.clone(),
            Template::new(command_content.trim()),
        )]
    } else if let Some(ref filename) = args.render {
        let file_content = fs::read_to_string(&filename).map_err(|e| e.to_string())?;
        vec![(
            "-R-".to_string(),
            file_content.clone(),
            Template::new(file_content.trim()),
        )]
    } else if let Some(ref template) = args.render_template {
        vec![(
            "-R-".to_string(),
            template.clone(),
            Template::new(template.trim()),
        )]
    } else if args.export.len() > 0 {
        vec![(
            "-E-".to_string(),
            args.export_format.clone(),
            Template::new(export::body_template(&args.export, &args.export_format)),
        )]
    } else
    // same as: if let Some(template) = args.command_template Since
    // these 4 are in a group and there must be one of them, inforced
    // by the argument parser
    {
        let template = args.command_template.unwrap();
        vec![("-T-".to_string(), template.clone(), Template::new(template))]
    };

    let mut overwrite: HashMap<&str, &str> = HashMap::new();
    let command_args: Vec<(String, &str)> = args
        .command_args
        .iter()
        .enumerate()
        .map(|(i, a)| (format!("ARG{}", i + 1), a.as_str()))
        .collect();
    command_args.iter().for_each(|(k, v)| {
        overwrite.insert(k.as_str(), *v);
    });
    if args.overwrite.len() > 0 {
        for vars in &args.overwrite {
            let mut split_data = vars.split(":").map(|s| s.split("=")).flatten();
            overwrite.insert(
                match split_data.next() {
                    Some(d) => d,
                    None => return Err(format!("Invalid Variable in overwrite: {}", vars)),
                },
                match split_data.next() {
                    Some(d) => d,
                    None => return Err(format!("Invalid Value in overwrite: {}", vars)),
                },
            );
            while let Some(d) = split_data.next() {
                eprintln!("Unused data from --overwrite: {}", d);
            }
        }
    }
    let (print_pre, print_pre_line, print_post_line, print_connector, print_post) =
        if args.export.len() > 0 {
            export::pre_post_templates(&args.export, &args.export_format)
        } else {
            export::pre_post_templates(&vec![], "")
        };
    print!("{}", print_pre);
    if args.input.len() > 0 {
        exec_pipeline_on_inputfile(
            &pipeline_templates,
            &anek_dir.get_files(&AnekDirectoryType::Inputs, &args.input),
            &args.path,
            &overwrite,
            args.demo,
            pipable,
            (print_pre_line, print_post_line),
        )?;
    } else if args.batch.len() > 0 {
        let batch_lines: Vec<(usize, String)> = args
            .batch
            .iter()
            .map(|b| variable::input_lines(&anek_dir.get_file(&AnekDirectoryType::Batch, &b), None))
            .flatten_ok()
            .collect::<Result<Vec<(usize, String)>, String>>()?
            .into_iter()
            .enumerate()
            .map(|(i, (_, s))| (i + 1, s))
            .filter(|f| {
                if filter_index.len() == 0 {
                    true
                } else {
                    filter_index.contains(&f.0)
                }
            })
            .collect();
        let mut current = 0;
        let total = batch_lines.len();
        for (i, line) in batch_lines {
            current += 1;
            if !args.pipable {
                eprintln!(
                    "{} {} [{} of {}]: {}",
                    "Job".bright_purple().bold(),
                    i,
                    current,
                    total,
                    line
                );
            }
            let input_files: Vec<&str> = line.split(",").collect();
            let post: String = if current < total {
                format!("{}{}", print_post_line, print_connector)
            } else {
                print_post_line.to_string()
            };
            exec_pipeline_on_inputfile(
                &pipeline_templates,
                &anek_dir.get_files(&AnekDirectoryType::Inputs, &input_files),
                &args.path,
                &overwrite,
                args.demo,
                pipable,
                (print_pre_line, &post),
            )?;
        }
    } else if let Some(loop_name) = args.r#loop {
        let loop_inputs = variable::loop_inputs(
            &anek_dir.get_file(&AnekDirectoryType::Loops, &format!("{}.d", loop_name)),
        )?;

        let permutations = loop_inputs
            .into_iter()
            // filter only the inputs used in the command file
            .filter(|inps| {
                pipeline_templates
                    .iter()
                    .any(|(_, temp, _)| temp.contains(&format!("{{{}}}", inps[0].0)))
            })
            .map(|inps| {
                if let Some(value) = overwrite.get(inps[0].0.as_str()) {
                    // TODO, make overwrite metavariables work with loop too.
                    vec![(inps[0].0.clone(), 0, value.to_string())]
                } else {
                    inps.to_vec()
                }
            })
            .multi_cartesian_product();

        let loop_total = permutations.clone().count();

        for (li, inputs) in permutations.enumerate() {
            let loop_index = li + 1;

            if filter_index.len() > 0 && !filter_index.contains(&loop_index) {
                continue;
            }
            let loop_index_str = loop_index.to_string();
            let mut input_map: HashMap<&str, &str> = HashMap::new();
            input_map.insert("LOOP_INDEX", &loop_index_str);
            if !args.pipable {
                eprint!(
                    "{} [{} of {}]: ",
                    "Input".bright_magenta().bold(),
                    loop_index,
                    loop_total
                );
            }
            for (var, i, val) in &inputs {
                input_map.insert(&var, &val);
                if !args.pipable {
                    eprint!("{} [{}]={}; ", &var, i, &val);
                }
            }
            if !args.pipable {
                eprintln!("");
            }
            let post: String = if loop_index < loop_total {
                format!("{}{}", print_post_line, print_connector)
            } else {
                print_post_line.to_string()
            };
            exec_pipeline(
                &pipeline_templates,
                &args.path,
                &input_map,
                args.demo,
                pipable,
                (print_pre_line, &post),
            )?;
        }
    } else {
        exec_pipeline(
            &pipeline_templates,
            &args.path,
            &overwrite,
            args.demo,
            pipable,
            (print_pre_line, print_post_line),
        )?;
    }
    print!("{}", print_post);
    Ok(())
}
