use clap::Args;
use colored::Colorize;
use itertools::Itertools;
use new_string_template::template::Template;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::PathBuf;
use subprocess::Exec;

use crate::input;

#[derive(Args)]
pub struct CliArgs {
    /// command to run
    command: String,
    /// Run from batch
    #[arg(short, long)]
    batch: Option<String>,
    /// Run batch by looping for the inputs
    #[arg(short, long)]
    r#loop: Option<String>,
    /// Run from favorites
    #[arg(short, long)]
    favorite: Option<String>,
    /// Overwrite input variables
    #[arg(short, long)]
    overwrite: Vec<String>,
    #[arg(default_value = ".")]
    path: PathBuf,
}

pub fn exec_file(
    cmd: &Template,
    filename: &PathBuf,
    wd: &PathBuf,
    overwrite: &HashMap<&str, &str>,
) -> Result<(), String> {
    let mut input_map: HashMap<&str, &str> = HashMap::new();
    let lines = input::input_lines(&filename)?;
    input::read_inputs(&lines, &mut input_map)?;
    input_map.extend(overwrite);
    let command = match cmd.render(&input_map) {
        Ok(c) => c,
        Err(e) => return Err(e.to_string()),
    };
    println!("{}: {}", "Run".green(), command.trim());
    Exec::shell(command).cwd(&wd).join().unwrap();
    Ok(())
}

pub fn run_command(args: CliArgs) -> Result<(), String> {
    let cmd_content = match read_to_string(args.path.join(".anek/commands").join(args.command)) {
        Ok(s) => s,
        Err(e) => return Err(e.to_string()),
    };
    let cmd_template = Template::new(&cmd_content);

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
        exec_file(
            &cmd_template,
            &args.path.join(".anek/favorites").join(favorite),
            &args.path,
            &overwrite,
        )?;
    }
    if let Some(batch) = args.batch {
        let batch_lines = input::input_lines(&args.path.join(".anek/batch").join(batch))?;
        for (i, line) in batch_lines {
            println!("{}: {}", "Job".green(), i + 1);
            exec_file(
                &cmd_template,
                &args.path.join(".anek").join(line),
                &args.path,
                &overwrite,
            )?;
        }
    }
    if let Some(loop_name) = args.r#loop {
        let loop_inputs = input::loop_inputs(&args.path.join(".anek/loops").join(loop_name))?;

        let permutations = loop_inputs
            .iter()
            // filter only the inputs used in the command file
            .filter(|inps| cmd_content.contains(&format!("{{{{{}}}}}", inps[0].0)))
            .map(|inps| {
                if let Some(value) = overwrite.get(inps[0].0.as_str()) {
                    vec![(inps[0].0.clone(), 0, value.to_string())]
                } else {
                    inps.to_vec()
                }
            })
            .multi_cartesian_product();

        let mut loop_index = 0; // extra variables for loop template
        for inputs in permutations {
            let loop_index_str = loop_index.to_string();
            let mut input_map: HashMap<&str, &str> = HashMap::new();
            input_map.insert("loop_index", &loop_index_str);
            print!("{} [{}]: ", "Input".bright_blue().bold(), loop_index);
            for (var, i, val) in &inputs {
                input_map.insert(&var, &val);
                print!("{}[{}]={}; ", &var, i, &val);
            }
            println!("");

            let command = match cmd_template.render(&input_map) {
                Ok(c) => c,
                Err(e) => return Err(e.to_string()),
            };
            println!("{}: {}", "Run".green(), command.trim());
            Exec::shell(command).cwd(&args.path).join().unwrap();
            loop_index += 1;
        }
    }
    Ok(())
}
