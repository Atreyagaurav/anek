use clap::{ArgGroup, Args, ValueHint};
use colored::Colorize;
use itertools::Itertools;
use new_string_template::template::Template;
use std::collections::HashMap;
use std::path::PathBuf;
use subprocess::Exec;

use crate::dtypes::{AnekDirectory, AnekDirectoryType};
use crate::variable;

#[derive(Args)]
#[command(group = ArgGroup::new("action").required(true).multiple(false))]
pub struct CliArgs {
    /// command to run (from .anek/commands/)
    ///
    /// The command file saved will have the command template inside
    /// it
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
    /// .anek/commands/
    #[arg(short, long, group="action", value_hint = ValueHint::Other)]
    pipeline: Option<String>,
    /// Run from batch
    ///
    /// Batch file are list of input files that are run one after
    /// another on the same command template. The list of input files
    /// need to be relative to .anek/
    #[arg(short, long, value_hint = ValueHint::Other, conflicts_with="favorite")]
    batch: Option<String>,
    /// Run commands by looping for the inputs
    ///
    /// Loops though the values of the input variables in the loop
    /// config and run the command templates on the combinations of
    /// those different variables.
    #[arg(short, long, value_hint = ValueHint::Other)]
    r#loop: Option<String>,
    /// Run from favorites
    ///
    /// Use the input variables' values defined in the favorites file
    /// as input to fill the tempalte and run it. You can specify a
    /// directory, and it'll use all the files inside to fill the
    /// template, in this case duplicate variable names will have only
    /// the last read value (alphabetically, then outside to inside
    /// recursively)
    #[arg(short, long, value_hint = ValueHint::Other)]
    input: Option<String>,
    /// Print commands in pipable format, assumes --demo
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
    #[arg(short, long, value_hint = ValueHint::Other)]
    overwrite: Vec<String>,
    #[arg(default_value = ".", value_hint = ValueHint::DirPath)]
    path: PathBuf,
}

pub fn exec_on_inputfile(
    cmd: &Template,
    filename: &PathBuf,
    wd: &PathBuf,
    overwrite: &HashMap<&str, &str>,
    demo: bool,
    pipable: bool,
    name: &str,
) -> Result<(), String> {
    let files = if filename.is_dir() {
        variable::list_filenames(&filename)?
            .iter()
            .map(|f| filename.join(f))
            .collect()
    } else if filename.is_file() {
        vec![filename.clone()]
    } else {
        return Err(format!(
            "Path {:?} is neither a directory nor a file",
            filename
        ));
    };
    let mut input_map: HashMap<&str, &str> = HashMap::new();
    let lines = files
        .iter()
        .map(|file| variable::input_lines(&file))
        .collect::<Result<Vec<Vec<(usize, String)>>, String>>()?;
    lines
        .iter()
        .map(|l| variable::read_inputs(&l, &mut input_map))
        .collect::<Result<(), String>>()?;
    input_map.extend(overwrite);
    let command = match cmd.render(&input_map) {
        Ok(c) => c,
        Err(e) => return Err(e.to_string()),
    };
    if !pipable {
        print!("{} ({}): ", "Command".bright_green(), name);
    }
    println!("{}", command.trim());
    if !(demo || pipable) {
        Exec::shell(command).cwd(&wd).join().unwrap();
    }
    Ok(())
}

pub fn exec_pipeline_on_inputfile(
    commands: &Vec<(String, String, Template)>,
    input_file: &PathBuf,
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
        let cmd = match command.render(&inputs) {
            Ok(c) => c,
            Err(e) => return Err(e.to_string()),
        };
        if !pipable {
            print!("{} ({}): ", "Command".bright_green(), name);
        }
        println!("{}", cmd);
        if !(demo || pipable) {
            Exec::shell(cmd).cwd(&wd).join().unwrap();
        }
    }
    Ok(())
}

pub fn run_command(args: CliArgs) -> Result<(), String> {
    let anek_dir = AnekDirectory::from(&args.path);
    let pipeline_templates = if let Some(pipeline) = args.pipeline {
        variable::input_lines(&anek_dir.get_file(&AnekDirectoryType::Pipelines, &pipeline))?
            .iter()
            .map(
                |(_, command)| -> Result<(String, String, Template), String> {
                    match variable::read_file_full(
                        &anek_dir.get_file(&AnekDirectoryType::Commands, &command),
                    ) {
                        Ok(s) => Ok((command.clone(), s.clone(), Template::new(s.trim()))),
                        Err(e) => Err(e.to_string()),
                    }
                },
            )
            .collect::<Result<Vec<(String, String, Template)>, String>>()
            .unwrap()
    } else if let Some(command) = args.command {
        let command_content =
            variable::read_file_full(&anek_dir.get_file(&AnekDirectoryType::Commands, &command))?;
        vec![(
            command,
            command_content.clone(),
            Template::new(command_content.trim()),
        )]
    } else
    // same as: if let Some(template) = args.command_template
    // Since these 3 are in a group and there must be one of them
    {
        let template = args.command_template.unwrap();
        vec![("CMD".to_string(), template.clone(), Template::new(template))]
    };

    let mut overwrite: HashMap<&str, &str> = HashMap::new();
    if args.overwrite.len() > 0 {
        for vars in &args.overwrite {
            let mut split_data = vars.split(":");
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
        }
    }

    if let Some(input) = args.input {
        exec_pipeline_on_inputfile(
            &pipeline_templates,
            &anek_dir.get_file(&AnekDirectoryType::Inputs, &input),
            &args.path,
            &overwrite,
            args.demo,
            args.pipable,
        )?;
    } else if let Some(batch) = args.batch {
        let batch_lines =
            variable::input_lines(&anek_dir.get_file(&AnekDirectoryType::Batch, &batch))?;
        for (i, line) in batch_lines {
            if !args.pipable {
                println!("{} [{}]: {}", "Job".bright_purple().bold(), i, line);
            }
            exec_pipeline_on_inputfile(
                &pipeline_templates,
                &anek_dir.get_file(&AnekDirectoryType::Inputs, &line),
                &args.path,
                &overwrite,
                args.demo,
                args.pipable,
            )?;
        }
    } else if let Some(loop_name) = args.r#loop {
        let loop_inputs =
            variable::loop_inputs(&anek_dir.get_file(&AnekDirectoryType::Loops, &loop_name))?;

        let permutations = loop_inputs
            .iter()
            // filter only the inputs used in the command file
            .filter(|inps| {
                pipeline_templates
                    .iter()
                    .any(|(_, temp, _)| temp.contains(&format!("{{{}}}", inps[0].0)))
            })
            .map(|inps| {
                if let Some(value) = overwrite.get(inps[0].0.as_str()) {
                    vec![(inps[0].0.clone(), 0, value.to_string())]
                } else {
                    inps.to_vec()
                }
            })
            .multi_cartesian_product();

        let mut loop_index = 1; // extra variables for loop template
        for inputs in permutations {
            let loop_index_str = loop_index.to_string();
            let mut input_map: HashMap<&str, &str> = HashMap::new();
            input_map.insert("loop_index", &loop_index_str);
            if !args.pipable {
                print!("{} [{}]: ", "Input".bright_magenta().bold(), loop_index);
            }
            for (var, i, val) in &inputs {
                input_map.insert(&var, &val);
                if !args.pipable {
                    print!("{} [{}]={}; ", &var, i, &val);
                }
            }
            if !args.pipable {
                println!("");
            }

            exec_pipeline(
                &pipeline_templates,
                &args.path,
                &input_map,
                args.demo,
                args.pipable,
            )?;
            loop_index += 1;
        }
    } else {
        exec_pipeline(
            &pipeline_templates,
            &args.path,
            &overwrite,
            args.demo,
            args.pipable,
        )?;
    }
    Ok(())
}
