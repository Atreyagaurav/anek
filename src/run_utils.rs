use anyhow::{Context, Error};
use clap::{ArgGroup, Args, Subcommand, ValueHint};
use itertools::Itertools;
use number_range::NumberRangeOptions;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use string_template_plus::{Render, RenderOptions, Template};

use crate::dtypes::{AnekDirectory, AnekDirectoryType, Command, CommandInputs, LoopVariable};
use crate::variable;

#[derive(Subcommand)]
pub enum Inputs {
    /// Use these inputs
    On(InputsArgs),
}

impl Default for Inputs {
    fn default() -> Self {
        Self::On(InputsArgs::default())
    }
}

impl Inputs {
    pub fn on(&self) -> &InputsArgs {
        match self {
            Inputs::On(a) => a,
        }
    }
}

#[derive(Args, Clone, Default)]
#[command(group = ArgGroup::new("variables").required(false).multiple(false))]
pub struct InputsArgs {
    /// Select subset to run, works in batch and loop only
    ///
    /// Since batch and loop are just a list of inputs to run, you can
    /// select a subset of them to run. The selection string needs to
    /// be comma separated values of possitive integers, you can have
    /// range of values like: 1-5 to represent values from 1 to 5.
    #[arg(short, long, default_value="", value_hint = ValueHint::Other, value_parser=filter_index)]
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
    /// Arguments to pass to the action template as ARG<N>
    ///
    /// The arguments passed here can be accessed as ARG1,ARG2,etc in
    /// the template.
    #[arg(num_args(0..), last(true), requires="action")]
    command_args: Vec<String>,
}

impl InputsArgs {
    pub fn batch(batch: Vec<String>) -> Self {
        Self {
            batch,
            ..Default::default()
        }
    }

    pub fn input(input: Vec<String>) -> Self {
        Self {
            input,
            ..Default::default()
        }
    }

    pub fn r#loop(r#loop: String) -> Self {
        Self {
            r#loop: Some(r#loop),
            ..Default::default()
        }
    }
}

fn filter_index(inputs: &str) -> Result<HashSet<usize>, Error> {
    Ok(NumberRangeOptions::default()
        .with_list_sep(',')
        .with_range_sep('-')
        .parse(inputs)?
        .collect())
}

pub fn cmd_from_pipeline(anek_dir: &AnekDirectory, pipeline: &str) -> Result<Vec<Command>, Error> {
    variable::input_lines(
        &anek_dir.get_file(&AnekDirectoryType::Pipelines, pipeline),
        None,
    )?
    .iter()
    .map(|(_, cmd)| -> Result<Command, Error> { anek_dir.command(cmd) })
    .collect()
}

pub fn command_args(args: &Inputs) -> Vec<(String, String)> {
    let args = args.on();
    args.command_args
        .iter()
        .enumerate()
        .map(|(i, a)| (format!("ARG{}", i + 1), a.to_string()))
        .collect()
}

pub fn overwrite_vars(
    args: &Inputs,
    command_args: &[(String, String)],
) -> Result<HashMap<String, String>, Error> {
    let args = args.on();
    let mut overwrite: HashMap<String, String> = HashMap::new();
    command_args.iter().for_each(|(k, v)| {
        overwrite.insert(k.to_string(), v.to_string());
    });
    if !args.overwrite.is_empty() {
        for vars in &args.overwrite {
            let mut split_data = vars.split(':').flat_map(|s| s.split('='));
            overwrite.insert(
                split_data
                    .next()
                    .context(format!("Invalid Variable in overwrite: {}", vars))?
                    .to_string(),
                split_data
                    .next()
                    .context(format!("Invalid Value in overwrite: {}", vars))?
                    .to_string(),
            );
            for d in split_data {
                eprintln!("Unused data from --overwrite: {}", d);
            }
        }
    }
    Ok(overwrite)
}

pub fn inputs(
    anek_dir: &AnekDirectory,
    args: &Inputs,
    variables: &HashSet<&str>,
) -> Result<Vec<CommandInputs>, Error> {
    if !args.on().batch.is_empty() {
        input_files(anek_dir, &args.on().batch, &args.on().select_inputs)
    } else if let Some(l) = &args.on().r#loop {
        let overwrite = overwrite_vars(args, &command_args(args))?;
        loop_inputs(anek_dir, l, &args.on().select_inputs, variables, &overwrite)
    } else {
        Ok(vec![anek_dir.inputs(1, &args.on().input).read_files()?])
    }
}

pub fn loop_inputs(
    anek_dir: &AnekDirectory,
    loop_file: &str,
    selection: &HashSet<usize>,
    variables: &HashSet<&str>,
    overwrite: &HashMap<String, String>,
) -> Result<Vec<CommandInputs>, Error> {
    let loop_dir = anek_dir
        .get_directory(&AnekDirectoryType::Loops)
        .join(format!("{loop_file}.d"));
    let loop_vars = variable::loop_inputs(&loop_dir)?;

    let permutations = loop_vars
        .into_iter()
        // filter only the inputs used in the command file
        .filter(|inps| variables.contains(&inps[0].name.as_str()))
        .map(|inps| {
            if let Some(value) = overwrite.get(inps[0].name.as_str()) {
                // TODO, make overwrite metavariables work with loop too.
                vec![LoopVariable::new(
                    inps[0].name.clone(),
                    0,
                    value.to_string(),
                )]
            } else {
                inps.to_vec()
            }
        })
        .multi_cartesian_product();
    let mut cmd_inputs = Vec::new();
    for (li, inputs) in permutations.enumerate() {
        let loop_index = li + 1;

        if !selection.is_empty() && !selection.contains(&loop_index) {
            continue;
        }
        let mut variables: HashMap<String, String> = HashMap::new();
        variables.insert("LOOP_INDEX".to_string(), loop_index.to_string());
        let mut name = String::new();
        for LoopVariable {
            name: var,
            index,
            value,
        } in &inputs
        {
            variables.insert(var.to_string(), value.to_string());
            name.push_str(&format!("{} [{}]={}; ", &var, index, &value));
        }

        let inp = CommandInputs::from_variables(loop_index, name, variables);
        cmd_inputs.push(inp);
    }
    Ok(cmd_inputs)
}

pub fn input_files(
    anek_dir: &AnekDirectory,
    batch_files: &[String],
    selection: &HashSet<usize>,
) -> Result<Vec<CommandInputs>, Error> {
    batch_files
        .iter()
        .map(|b| variable::input_lines(&anek_dir.get_file(&AnekDirectoryType::Batch, b), None))
        .flatten_ok()
        .collect::<Result<Vec<(usize, String)>, Error>>()?
        .into_iter()
        .enumerate()
        .map(|(i, (_, s))| (i + 1, s))
        .filter(|f| {
            if selection.is_empty() {
                true
            } else {
                selection.contains(&f.0)
            }
        })
        .map(|(i, line)| {
            let files: Vec<&str> = line.split(',').collect();
            let inp = anek_dir.inputs(i, &files);
            inp.read_files()
        })
        .collect()
}

pub fn variables_from_input(
    input: &CommandInputs,
    overwrite: &HashMap<String, String>,
) -> Result<HashMap<String, String>, Error> {
    let mut input_map = input.variables().clone();
    // render the metavariables in the overwrite
    let renderop = RenderOptions {
        variables: input_map.clone(),
        wd: PathBuf::default(),
        shell_commands: true,
    };
    let overwrite_meta: Vec<(String, String)> = overwrite
        .iter()
        .map(|(k, v)| -> Result<(String, String), Error> {
            Template::parse_template(v)?
                .render(&renderop)
                .map(|s| (k.to_string(), s))
        })
        .collect::<Result<Vec<(String, String)>, Error>>()?;
    for (k, v) in overwrite_meta {
        input_map.insert(k.to_string(), v);
    }
    Ok(input_map)
}
