use std::io::Result;
use std::path::{Path, PathBuf};
use std::collections::HashSet;

use super::settings::{
    read_settings_file,
    write_settings_file,
};

use crate::data_asset::StringLogger;

pub struct RecentProjects {
    paths: Vec<PathBuf>,
}

impl RecentProjects {
    const MAX_PATHS_SAVED: usize = 5;
    const FILENAME: &str = "recent_projects.txt";

    pub fn new() -> Self {
        RecentProjects {
            paths: Vec::new(),
        }
    }

    fn hash(path: &Path) -> u64 {
        use std::hash::{Hash, Hasher};

        let mut hasher = std::hash::DefaultHasher::new();
        path.hash(&mut hasher);
        hasher.finish()
    }

    fn trim_paths(&mut self) {
        let mut seen = HashSet::new();
        self.paths.retain(|p| {
            let hash = Self::hash(p);
            if ! seen.contains(&hash) {
                seen.insert(hash);
                true
            } else {
                false
            }
        });
        while self.paths.len() > Self::MAX_PATHS_SAVED {
            self.paths.pop();
        }
    }

    pub fn add<P: AsRef<Path>>(&mut self, path: P) {
        self.paths.push(path.as_ref().to_path_buf());
        self.trim_paths();
    }

    fn load_settings_file(&mut self) -> Result<()> {
        let config = read_settings_file(Self::FILENAME)?;
        for line in config.lines() {
            if ! line.is_empty() && ! line.starts_with("#") {
                self.paths.push(std::path::PathBuf::from(line));
            }
        }
        self.trim_paths();
        Ok(())
    }

    pub fn load(&mut self, logger: &mut StringLogger) {
        if let Err(e) = self.load_settings_file() {
            logger.log(format!("ERROR loading recent projects:\n{}", e));
        }
    }

    pub fn save(&mut self, logger: &mut StringLogger) {
        let mut config = String::new();
        for path in self.paths.iter() {
            config.push_str(&path.to_string_lossy());
            config.push('\n');
        }
        if let Err(e) = write_settings_file(Self::FILENAME, &config) {
            logger.log(format!("ERROR writing recent projects: '{}'", e));
        }
    }

    pub fn num_files(&self) -> usize {
        self.paths.len()
    }

    pub fn files(&self) -> impl Iterator<Item = &PathBuf> {
        self.paths.iter()
    }

    pub fn file(&self, index: usize) -> Option<&PathBuf> {
        self.paths.get(index)
    }
}
