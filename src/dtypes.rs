use anyhow::Error;
use colored::Colorize;
use core::slice::Iter;
use itertools::Itertools;
use std::{collections::HashMap, fs, path::PathBuf};
use string_template_plus::{Render, RenderOptions, Template};
use subprocess::Exec;

use crate::variable;

#[derive(Clone)]
pub enum AnekDirectoryType {
    Variables,
    Inputs,
    Commands,
    Pipelines,
    Loops,
    Batch,
}

impl AnekDirectoryType {
    pub fn dir_name(&self) -> &'static str {
        match self {
            AnekDirectoryType::Variables => "variables",
            AnekDirectoryType::Inputs => "inputs",
            AnekDirectoryType::Commands => "commands",
            AnekDirectoryType::Pipelines => "pipelines",
            AnekDirectoryType::Loops => "loops",
            AnekDirectoryType::Batch => "batch",
        }
    }

    pub fn dir_description(&self) -> &'static str {
        match self {
            AnekDirectoryType::Variables => concat!(
                "Variable file contains the descriptions of the variables.\n\n",
                "These files are not required, but help with keeping it documented, ",
                "and to help with making it easy for the program to verify they are ",
                "not mispelled in the input files or commands."
            ),
            AnekDirectoryType::Inputs => concat!(
                "Input files are files containing the variables and their values.\n\n",
                "They can be used to render the commands, you can group input files ",
                "in a directory and pass that directory as input file to use all the ",
                "variables in it."
            ),
            AnekDirectoryType::Commands => concat!(
                "Command files have command templates in them.\n\n",
                "They are simple commands, or just call scripts with arguments. ",
                "They can have the variables in braces that'll be rendered with ",
                "the value of the variable from input file."
            ),
            AnekDirectoryType::Pipelines => concat!(
                "Pipelines are sequence of commands.\n\n",
                "Pipeline execution will run the sequence of commands one after ",
                "another in the given order using the same input variables to render."
            ),
            AnekDirectoryType::Loops => concat!(
                "Loops loop through mutiple variables' values.\n\n",
                "Loops are directories that have files for each variable. ",
                "Put multiple values for the variable if you want to loop ",
                "through those values, you can do that to multiple variables."
            ),
            AnekDirectoryType::Batch => concat!(
                "Batch file are list of inputs.\n\n",
                "You can put list of inputs in batch file, unlike grouping inputs ",
                "in a directory that'll process them as a single input with all the ",
                "variables, batch file will process each inputs with the commands in ",
                "a batch process."
            ),
        }
    }
}

pub struct AnekDirectory {
    pub root: PathBuf,
}

pub struct Command {
    name: String,
    cmd: String,
    templ: Template,
}

impl Command {
    pub fn new<T: ToString>(name: T, cmd: T) -> Result<Self, Error> {
        Ok(Self {
            name: name.to_string(),
            cmd: cmd.to_string(),
            templ: Template::parse_template(&cmd.to_string())?,
        })
    }

    pub fn template(&self) -> &Template {
        &self.templ
    }

    pub fn print(&self, rendered_cmd: &str) {
        eprint!("{} ({}): ", "Command".bright_green(), self.name);
        println!("{}", rendered_cmd);
        eprintln!("⇒");
    }

    pub fn render(&self, variables: HashMap<String, String>, wd: PathBuf) -> Result<String, Error> {
        let op = RenderOptions {
            wd: wd,
            variables,
            shell_commands: true,
        };
        self.templ.render(&op)
    }

    pub fn run(
        &self,
        variables: &HashMap<String, String>,
        wd: &PathBuf,
        demo: bool,
        print: bool,
    ) -> Result<(), Error> {
        let cmd = self.render(variables.clone(), wd.to_path_buf())?;
        if print {
            self.print(&cmd);
        }
        if !demo {
            Exec::shell(cmd).cwd(&wd).join()?;
        }
        Ok(())
    }
}

pub struct CommandInputs {
    index: usize,
    name: String,
    files: Vec<PathBuf>,
    variables: HashMap<String, String>,
}

impl CommandInputs {
    pub fn from_files(index: usize, name: String, files: Vec<PathBuf>) -> Self {
        Self {
            index,
            name,
            files,
            variables: HashMap::new(),
        }
    }

    pub fn read_files(mut self) -> Result<Self, Error> {
        let lines = variable::compact_lines_from_anek_file(self.files())?;
        variable::read_inputs(&lines, &mut self.variables)?;
        Ok(self)
    }

    pub fn files(&self) -> &Vec<PathBuf> {
        &self.files
    }

    pub fn variables(&self) -> &HashMap<String, String> {
        &self.variables
    }

    pub fn eprint_job(&self, job: usize, total: usize) {
        eprintln!(
            "{} {} [{} of {}]: {}",
            "Job".bright_purple().bold(),
            self.index,
            job,
            total,
            self.name
        );
    }
}

impl AnekDirectory {
    pub fn from(wd: &PathBuf) -> Result<Self, Error> {
        let root = wd.join(".anek");
        if root.exists() {
            if root.is_dir() {
                Ok(Self { root })
            } else {
                Err(Error::msg(format!("{:?} is not a directory", root)))
            }
        } else {
            let wd = wd.canonicalize()?;
            if let Some(p) = wd.parent() {
                AnekDirectory::from(&p.to_path_buf())
            } else {
                Err(Error::msg("No .anek configuration in the current path"))
            }
        }
    }

    pub fn new(wd: &PathBuf) -> Result<Self, Error> {
        let root = wd.join(".anek");
        if root.exists() {
            if root.is_dir() {
                Err(Error::msg(format!(
                    "{:?} already has anek configuration",
                    root
                )))
            } else {
                Err(Error::msg(format!(
                    "{:?} file exists, that is not anek configuration",
                    root
                )))
            }
        } else {
            fs::create_dir(&root)?;
            let anek = AnekDirectory::from(&wd)?;
            for adt in anekdirtype_iter() {
                fs::create_dir(anek.get_directory(adt))?;
            }
            Ok(anek)
        }
    }

    pub fn get_directory(&self, dirtype: &AnekDirectoryType) -> PathBuf {
        self.root.join(dirtype.dir_name())
    }

    pub fn get_file(&self, dirtype: &AnekDirectoryType, filename: &str) -> PathBuf {
        self.get_directory(dirtype).join(&filename)
    }

    pub fn url_to_path(&self, dirtype: &AnekDirectoryType, filename: &str) -> String {
        let mut file = self.get_file(dirtype, filename);
        if !file.exists() {
            let file_d = self.get_file(dirtype, &format!("{filename}.d"));
            if file_d.exists() {
                file = file_d;
            }
        }
        format!("{}", file.to_str().unwrap())
    }

    pub fn get_files<T: ToString>(
        &self,
        dirtype: &AnekDirectoryType,
        filenames: &Vec<T>,
    ) -> Vec<PathBuf> {
        filenames
            .iter()
            .map(|f| self.get_file(&dirtype, &f.to_string()))
            .collect()
    }

    pub fn command(&self, cmd: &str) -> Result<Command, Error> {
        let s = fs::read_to_string(self.get_file(&AnekDirectoryType::Commands, &cmd))?;
        let templ = Template::parse_template(s.trim())?;
        Ok(Command {
            name: cmd.to_string(),
            cmd: s,
            templ,
        })
    }

    pub fn inputs<T: ToString>(&self, index: usize, files: &Vec<T>) -> CommandInputs {
        CommandInputs::from_files(
            index,
            files.iter().map(|s| s.to_string()).join(","),
            self.get_files(&AnekDirectoryType::Inputs, &files),
        )
    }
}

pub fn anekdirtype_iter() -> Iter<'static, AnekDirectoryType> {
    [
        AnekDirectoryType::Variables,
        AnekDirectoryType::Inputs,
        AnekDirectoryType::Commands,
        AnekDirectoryType::Pipelines,
        AnekDirectoryType::Loops,
        AnekDirectoryType::Batch,
    ]
    .iter()
}
