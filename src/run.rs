use clap::{ArgGroup, Args, ValueHint};
use colored::Colorize;
use itertools::Itertools;
use new_string_template::template::Template;
use std::collections::HashMap;
use std::path::PathBuf;
use subprocess::Exec;

use crate::input;

#[derive(Args)]
#[command(group = ArgGroup::new("action").required(true).multiple(false))]
pub struct CliArgs {
    /// command to run (from .anek/commands/)
    #[arg(short, long, group="action", value_hint = ValueHint::Other)]
    command: Option<String>,
    /// Command template to run
    #[arg(short = 'C', long, group="action", value_hint = ValueHint::CommandName)]
    command_template: Option<String>,
    /// Run a pipeline
    #[arg(short, long, group="action", value_hint = ValueHint::Other)]
    pipeline: Option<String>,
    /// Run from batch
    #[arg(short, long, value_hint = ValueHint::Other, conflicts_with="favorite")]
    batch: Option<String>,
    /// Run batch by looping for the inputs
    #[arg(short, long, value_hint = ValueHint::Other)]
    r#loop: Option<String>,
    /// Run from favorites
    #[arg(short, long, value_hint = ValueHint::Other)]
    favorite: Option<String>,
    /// Demo
    #[arg(short, long)]
    demo: bool,
    /// Overwrite input variables
    #[arg(short, long, value_hint = ValueHint::Other)]
    overwrite: Vec<String>,
    #[arg(default_value = ".", value_hint = ValueHint::DirPath)]
    path: PathBuf,
}

pub fn exec_file(
    cmd: &Template,
    filename: &PathBuf,
    wd: &PathBuf,
    overwrite: &HashMap<&str, &str>,
    demo: bool,
    name: &str,
) -> Result<(), String> {
    let mut input_map: HashMap<&str, &str> = HashMap::new();
    let lines = input::input_lines(&filename)?;
    input::read_inputs(&lines, &mut input_map)?;
    input_map.extend(overwrite);
    let command = match cmd.render(&input_map) {
        Ok(c) => c,
        Err(e) => return Err(e.to_string()),
    };
    println!(
        "{} ({}): {}",
        "Command".bright_green(),
        name,
        command.trim()
    );
    if !demo {
        Exec::shell(command).cwd(&wd).join().unwrap();
    }
    Ok(())
}

pub fn exec_pipeline_on_file(
    commands: &Vec<(String, String, Template)>,
    input_file: &PathBuf,
    wd: &PathBuf,
    overwrite: &HashMap<&str, &str>,
    demo: bool,
) -> Result<(), String> {
    for (name, _, command) in commands {
        exec_file(&command, &input_file, &wd, &overwrite, demo, name)?;
    }
    Ok(())
}

pub fn exec_pipeline(
    commands: &Vec<(String, String, Template)>,
    wd: &PathBuf,
    inputs: &HashMap<&str, &str>,
    demo: bool,
) -> Result<(), String> {
    for (name, _, command) in commands {
        let cmd = match command.render(&inputs) {
            Ok(c) => c,
            Err(e) => return Err(e.to_string()),
        };
        println!("{} ({}): {}", "Command".bright_green(), name, cmd);
        if !demo {
            Exec::shell(cmd).cwd(&wd).join().unwrap();
        }
    }
    Ok(())
}

pub fn run_command(args: CliArgs) -> Result<(), String> {
    let pipeline_templates = if let Some(pipeline) = args.pipeline {
        input::input_lines(&args.path.join(".anek/pipelines").join(&pipeline))?
            .iter()
            .map(
                |(_, command)| -> Result<(String, String, Template), String> {
                    match input::read_file_full(&args.path.join(".anek/commands").join(&command)) {
                        Ok(s) => Ok((command.clone(), s.clone(), Template::new(s.trim()))),
                        Err(e) => Err(e.to_string()),
                    }
                },
            )
            .collect::<Result<Vec<(String, String, Template)>, String>>()
            .unwrap()
    } else if let Some(command) = args.command {
        let command_content =
            input::read_file_full(&args.path.join(".anek/commands").join(&command))?;
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

    if let Some(favorite) = args.favorite {
        exec_pipeline_on_file(
            &pipeline_templates,
            &args.path.join(".anek/favorites").join(favorite),
            &args.path,
            &overwrite,
            args.demo,
        )?;
    }
    if let Some(batch) = args.batch {
        let batch_lines = input::input_lines(&args.path.join(".anek/batch").join(batch))?;
        for (i, line) in batch_lines {
            println!("{} [{}]: {}", "Job".bright_purple().bold(), i, line);
            exec_pipeline_on_file(
                &pipeline_templates,
                &args.path.join(".anek").join(line),
                &args.path,
                &overwrite,
                args.demo,
            )?;
        }
    }
    if let Some(loop_name) = args.r#loop {
        let loop_inputs = input::loop_inputs(&args.path.join(".anek/loops").join(loop_name))?;

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
            print!("{} [{}]: ", "Input".bright_magenta().bold(), loop_index);
            for (var, i, val) in &inputs {
                input_map.insert(&var, &val);
                print!("{} [{}]={}; ", &var, i, &val);
            }
            println!("");

            exec_pipeline(&pipeline_templates, &args.path, &input_map, args.demo)?;
            loop_index += 1;
        }
    }
    Ok(())
}
