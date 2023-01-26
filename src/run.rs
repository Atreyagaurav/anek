use clap::{ArgGroup, Args, ValueHint};
use colored::Colorize;
use itertools::Itertools;
use new_string_template::template::Template;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use subprocess::Exec;

use crate::dtypes::{AnekDirectory, AnekDirectoryType};
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
}

pub fn exec_on_inputfile(
    cmd: &Template,
    filenames: &Vec<PathBuf>,
    wd: &PathBuf,
    overwrite: &HashMap<&str, &str>,
    demo: bool,
    pipable: bool,
    name: &str,
) -> Result<(), String> {
    let mut files: Vec<PathBuf> = Vec::new();
    for filename in filenames {
        if filename.is_dir() {
            files.extend(
                variable::list_filenames(&filename)?
                    .iter()
                    .map(|f| filename.join(f)),
            );
        } else if filename.is_file() {
            files.push(filename.clone());
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
            variable::render_template(&Template::new(v.to_string()), &input_map)
                .and_then(|s| Ok((*k, s)))
                .map_err(|e| e.to_string())
        })
        .collect::<Result<Vec<(&str, String)>, String>>()?;
    for (k, v) in &overwrite_meta {
        input_map.insert(k, &v);
    }

    let command = variable::render_template(cmd, &input_map)?;
    if !pipable {
        eprint!("{} ({}): ", "Command".bright_green(), name);
    }
    println!("{}", command.trim());
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
) -> Result<(), String> {
    for (name, _, command) in commands {
        exec_on_inputfile(&command, &input_file, &wd, &overwrite, demo, pipable, name)?;
    }
    Ok(())
}

pub fn exec_pipeline(
    commands: &Vec<(String, String, Template)>,
    wd: &PathBuf,
    inputs: &HashMap<&str, &str>,
    demo: bool,
    pipable: bool,
) -> Result<(), String> {
    for (name, _, command) in commands {
        let cmd = variable::render_template(command, &inputs)?;
        if !pipable {
            eprint!("{} ({}): ", "Command".bright_green(), name);
        }
        println!("{}", cmd);
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
    let anek_dir = AnekDirectory::from(&args.path);
    let pipable = args.pipable || args.render.is_some() || args.render_template.is_some();

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
    } else
    // same as: if let Some(template) = args.command_template Since
    // these 4 are in a group and there must be one of them, inforced
    // by the argument parser
    {
        let template = args.command_template.unwrap();
        vec![("-T-".to_string(), template.clone(), Template::new(template))]
    };

    let mut overwrite: HashMap<&str, &str> = HashMap::new();
    overwrite.insert("", ""); // to replace {} as empty string.
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

    if args.input.len() > 0 {
        exec_pipeline_on_inputfile(
            &pipeline_templates,
            &anek_dir.get_files(&AnekDirectoryType::Inputs, &args.input),
            &args.path,
            &overwrite,
            args.demo,
            pipable,
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
            .collect();
        let total = batch_lines.len();
        for (i, line) in batch_lines {
            if !args.pipable {
                eprintln!(
                    "{} [{} of {}]: {}",
                    "Job".bright_purple().bold(),
                    i,
                    total,
                    line
                );
            }
            let input_files: Vec<&str> = line.split(",").collect();
            exec_pipeline_on_inputfile(
                &pipeline_templates,
                &anek_dir.get_files(&AnekDirectoryType::Inputs, &input_files),
                &args.path,
                &overwrite,
                args.demo,
                pipable,
            )?;
        }
    } else if let Some(loop_name) = args.r#loop {
        let loop_inputs =
            variable::loop_inputs(&anek_dir.get_file(&AnekDirectoryType::Loops, &loop_name))?;

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

        let mut loop_index = 1; // extra variable for loop template
        let loop_total = permutations.clone().count();
        for inputs in permutations {
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

            exec_pipeline(
                &pipeline_templates,
                &args.path,
                &input_map,
                args.demo,
                pipable,
            )?;
            loop_index += 1;
        }
    } else {
        exec_pipeline(
            &pipeline_templates,
            &args.path,
            &overwrite,
            args.demo,
            pipable,
        )?;
    }
    Ok(())
}
