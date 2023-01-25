use core::slice::Iter;
use std::path::PathBuf;

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

impl AnekDirectory {
    pub fn from(wd: &PathBuf) -> Self {
        Self {
            root: wd.join(".anek"),
        }
    }

    pub fn exists(&self) -> bool {
        self.root.exists() && self.root.is_dir()
    }

    pub fn get_directory(&self, dirtype: &AnekDirectoryType) -> PathBuf {
        self.root.join(dirtype.dir_name())
    }

    pub fn get_file(&self, dirtype: &AnekDirectoryType, filename: &str) -> PathBuf {
        self.get_directory(dirtype).join(&filename)
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
