use core::slice::Iter;
use std::path::PathBuf;

pub enum AnekDirectoryType {
    Inputs,
    Favorites,
    Commands,
    Pipelines,
    Loop,
    Batch,
    History,
}

impl AnekDirectoryType {
    pub fn dir_name(&self) -> &'static str {
        match self {
            AnekDirectoryType::Inputs => "inputs",
            AnekDirectoryType::Favorites => "favorites",
            AnekDirectoryType::Commands => "commands",
            AnekDirectoryType::Pipelines => "pipelines",
            AnekDirectoryType::Loop => "loop",
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
        AnekDirectoryType::Inputs,
        AnekDirectoryType::Favorites,
        AnekDirectoryType::Commands,
        AnekDirectoryType::Pipelines,
        AnekDirectoryType::Loop,
        AnekDirectoryType::Batch,
        AnekDirectoryType::History,
    ]
    .iter()
}
