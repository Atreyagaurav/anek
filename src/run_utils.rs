use anyhow::{Context, Error};
use clap::{ArgGroup, Args, ValueHint};
use colored::Colorize;
use itertools::Itertools;
use number_range::NumberRangeOptions;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use string_template_plus::Template;
use subprocess::Exec;

use crate::dtypes::{AnekDirectory, AnekDirectoryType, Command};
use crate::variable;

#[derive(Args)]
#[command(group = ArgGroup::new("variables").required(false).multiple(false))]
pub struct InputsArgs {
    /// Select subset to run, works in batch and loop only
    ///
    /// Since batch and loop are just a list of inputs to run, you can
    /// select a subset of them to run. The selection string needs to
    /// be comma separated values of possitive integers, you can have
    /// range of values like: 1-5 to represent values from 1 to 5.
    #[arg(short, long, value_hint = ValueHint::Other, value_parser=filter_index)]
    select_inputs: HashSet<usize>,
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

fn filter_index(inputs: &str) -> Result<HashSet<usize>, Error> {
    Ok(NumberRangeOptions::default()
        .with_list_sep(',')
        .with_range_sep('-')
        .parse(&inputs)?
        .collect())
}

pub fn cmd_from_pipeline(anek_dir: &AnekDirectory, pipeline: &str) -> Result<Vec<Command>, Error> {
    variable::input_lines(
        &anek_dir.get_file(&AnekDirectoryType::Pipelines, &pipeline),
        None,
    )?
    .iter()
    .map(|(_, cmd)| -> Result<Command, Error> { anek_dir.command(cmd) })
    .collect()
}

pub fn command_args(args: &InputsArgs) -> Vec<(String, &str)> {
    args.command_args
        .iter()
        .enumerate()
        .map(|(i, a)| (format!("ARG{}", i + 1), a.as_str()))
        .collect()
}

pub fn overwrite_vars<'a>(
    args: &'a InputsArgs,
    command_args: &'a Vec<(String, &str)>,
) -> Result<HashMap<&'a str, &'a str>, Error> {
    let mut overwrite: HashMap<&str, &str> = HashMap::new();
    command_args.iter().for_each(|(k, v)| {
        overwrite.insert(k.as_str(), *v);
    });
    if args.overwrite.len() > 0 {
        for vars in &args.overwrite {
            let mut split_data = vars.split(":").map(|s| s.split("=")).flatten();
            overwrite.insert(
                split_data
                    .next()
                    .context(format!("Invalid Variable in overwrite: {}", vars))?,
                split_data
                    .next()
                    .context(format!("Invalid Value in overwrite: {}", vars))?,
            );
            while let Some(d) = split_data.next() {
                eprintln!("Unused data from --overwrite: {}", d);
            }
        }
    }
    Ok(overwrite)
}

pub fn input_files(
    anek_dir: &AnekDirectory,
    args: &InputsArgs,
    filter_index: &HashSet<usize>,
) -> Result<Vec<Vec<PathBuf>>, Error> {
    if args.input.len() > 0 {
        Ok(vec![
            anek_dir.get_files(&AnekDirectoryType::Inputs, &args.input)
        ])
    } else if args.batch.len() > 0 {
        let batch_lines: Vec<(usize, String)> = args
            .batch
            .iter()
            .map(|b| variable::input_lines(&anek_dir.get_file(&AnekDirectoryType::Batch, &b), None))
            .flatten_ok()
            .collect::<Result<Vec<(usize, String)>, Error>>()?
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
        batch_lines
            .iter()
            .map(|(i, line)| {
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
                let files: Vec<&str> = line.split(",").collect();
                Ok(anek_dir.get_files(&AnekDirectoryType::Inputs, &files))
            })
            .collect()
    } else {
        Ok(vec![])
    }
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
) -> Result<(), Error> {
    let command = render_on_inputfile(cmd, filenames, wd, overwrite)?;
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

pub fn render_on_inputfile(
    templ: &Template,
    filenames: &Vec<PathBuf>,
    wd: &PathBuf,
    overwrite: &HashMap<&str, &str>,
) -> Result<String, Error> {
    let mut input_map: HashMap<&str, &str> = HashMap::new();
    let lines = variable::compact_lines_from_anek_file(filenames)?;
    variable::read_inputs(&lines, &mut input_map)?;
    // render the metavariables in the overwrite
    let overwrite_meta: Vec<(&str, String)> = overwrite
        .iter()
        .map(|(k, v)| -> Result<(&str, String), Error> {
            variable::render_template(&Template::parse_template(v)?, &input_map, wd)
                .and_then(|s| Ok((*k, s)))
        })
        .collect::<Result<Vec<(&str, String)>, Error>>()?;
    for (k, v) in &overwrite_meta {
        input_map.insert(k, &v);
    }
    variable::render_template(templ, &input_map, wd)
}

pub fn exec_pipeline_on_inputfile(
    commands: &Vec<(String, String, Template)>,
    input_file: &Vec<PathBuf>,
    wd: &PathBuf,
    overwrite: &HashMap<&str, &str>,
    demo: bool,
    pipable: bool,
    print_extras: (&str, &str),
) -> Result<(), Error> {
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
) -> Result<(), Error> {
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
            Exec::shell(cmd).cwd(&wd).join()?;
        }
    }
    Ok(())
}
