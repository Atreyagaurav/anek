use clap::Args;
use colored::Colorize;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::PathBuf;
use string_template::Template;
use subprocess::Exec;

use crate::input;

#[derive(Args)]
pub struct CliArgs {
    /// command to run
    command: String,
    /// Run with default values
    #[arg(short, long, action)]
    default: bool,
    /// Run from batch
    #[arg(short, long)]
    batch: Option<String>,
    /// Run batch by looping for the inputs
    #[arg(short, long)]
    r#loop: Option<String>,
    /// Run from favorites
    #[arg(short, long)]
    favorite: Option<String>,
    #[arg(default_value = ".")]
    path: PathBuf,
}

pub fn exec_file(cmd: &Template, filename: &PathBuf, wd: &PathBuf) -> Result<(), String> {
    let mut input_map: HashMap<&str, &str> = HashMap::new();
    let lines = input::input_lines(&filename)?;
    input::read_inputs(&lines, &mut input_map)?;
    let command = cmd.render(&input_map);
    println!("{}: {}", "Run".green(), command.trim());
    Exec::shell(command).cwd(&wd).join().unwrap();
    Ok(())
}

pub fn run_command(args: CliArgs) -> Result<(), String> {
    let cmd_content = match read_to_string(args.path.join("commands").join(args.command)) {
        Ok(s) => s,
        Err(e) => return Err(e.to_string()),
    };
    let cmd_template = Template::new(&cmd_content);
    if let Some(favorite) = args.favorite {
        exec_file(
            &cmd_template,
            &args.path.join("favorites").join(favorite),
            &args.path.join("contents"),
        )?;
    }
    if let Some(batch) = args.batch {
        let batch_lines = input::input_lines(&args.path.join("batch").join(batch))?;
        for (i, line) in batch_lines {
            println!("{}: {}", "Job".green(), i + 1);
            exec_file(
                &cmd_template,
                &args.path.join(line),
                &args.path.join("contents"),
            )?;
        }
    }
    if let Some(loop_name) = args.r#loop {
        let loop_inputs = input::loop_inputs(&args.path.join("loops").join(loop_name))?;

        let mut loop_index = 0; // extra variables for loop template
        for inputs in loop_inputs {
            let loop_index_str = loop_index.to_string();
            let mut input_map: HashMap<&str, &str> = HashMap::new();
            input_map.insert("loop_index", &loop_index_str);
            print!("{}: ", "Input".bright_blue().bold());
            for (var, i, val) in &inputs {
                input_map.insert(&var, &val);
                print!("{}[{}]={}; ", &var, i, &val);
            }
            println!("");

            let command = cmd_template.render(&input_map);
            println!("{}: {}", "Run".green(), command.trim());
            Exec::shell(command)
                .cwd(&args.path.join("contents"))
                .join()
                .unwrap();
            loop_index += 1;
        }
    }
    Ok(())
}
