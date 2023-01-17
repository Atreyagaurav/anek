use core::slice::Iter;
use std::path::PathBuf;

pub enum AnekDirectoryType {
    Variables,
    Inputs,
    Commands,
    Pipelines,
    Loops,
    Batch,
    History,
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
            AnekDirectoryType::History => "history",
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
}

pub fn anekdirtype_iter() -> Iter<'static, AnekDirectoryType> {
    [
        AnekDirectoryType::Variables,
        AnekDirectoryType::Inputs,
        AnekDirectoryType::Commands,
        AnekDirectoryType::Pipelines,
        AnekDirectoryType::Loops,
        AnekDirectoryType::Batch,
        AnekDirectoryType::History,
    ]
    .iter()
}
