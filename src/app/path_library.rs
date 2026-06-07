use std::io::{Result, Error};
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use super::settings::{get_settings_dir, write_settings_file};

use crate::data_asset::StringLogger;
use crate::data_asset::Tokenizer;

pub struct PathLibrary {
    paths: HashMap<String, PathBuf>,
    last: Option<PathBuf>,
}

impl PathLibrary {
    const FILENAME: &str = "saved_paths.txt";

    pub fn new() -> Self {
        PathLibrary {
            paths: HashMap::new(),
            last: None,
        }
    }

    pub fn set<P: AsRef<Path>>(&mut self, name: &str, path: P) {
        let path = path.as_ref().to_path_buf();

        // save to <last>
        let last = self.last.get_or_insert_with(PathBuf::new);
        last.clear();
        last.push(path.as_path());

        // save to library
        self.paths.insert(name.to_owned(), path);
    }

    pub fn get(&mut self, name: &str) -> Option<PathBuf> {
        match self.paths.get(name) {
            Some(p) => {                // got path; save it to <last> and return
                let last = self.last.get_or_insert_with(PathBuf::new);
                last.clear();
                last.push(p.as_path());
                Some(p.clone())
            }
            None => self.last.clone()  // no path; return whatever we had in <last>
        }
    }

    fn load_from_file<P: AsRef<Path>>(&mut self, filename: P) -> Result<()> {
        let config = std::fs::read_to_string(&filename)?;
        let mut tok = Tokenizer::new(&config);
        loop {
            let t = tok.read()?;
            if t.is_eof() { break; }
            if let Some(name) = t.take_ident() {
                let t = tok.read()?;
                if ! t.is_punct('=') {
                    return Err(Error::other(format!("expected '=', found '{}' at line {}", t, t.pos.line)));
                }
                let t = tok.read()?;
                if let Some(dir) = t.get_string() {
                    if name == "__last" {
                        self.last = Some(dir.into());
                    } else {
                        self.paths.insert(name, dir.into());
                    }
                } else {
                    return Err(Error::other(format!("expected directory in quotes, found '{}' at line {}", t, t.pos.line)));
                }
            }
        }
        Ok(())
    }

    pub fn load(&mut self, logger: &mut StringLogger) {
        let settings_dir = match get_settings_dir() {
            Some(dir) => { dir }
            None => {
                logger.log("WARNING: can't find settings directory, directories won't be loaded");
                return;
            }
        };
        let filename = settings_dir.join(Self::FILENAME);
        logger.log(format!("Loading dirs from '{}'", filename.display()));
        if let Err(e) = self.load_from_file(filename) {
            logger.log(format!("ERROR loading dirs: {}", e));
        }
    }

    fn save_entry(config: &mut String, name: &str, path: Option<&PathBuf>) -> bool {
        if let Some(path) = path && let Some(dir) = path.to_str() && ! dir.contains('\n') {
            config.push_str(name);
            config.push_str(" = \"");
            config.push_str(&dir.replace("\\", "\\\\").replace("\"", "\\\""));
            config.push_str("\"\n");
            true
        } else {
            false
        }
    }

    pub fn save(&mut self, logger: &mut StringLogger) {
        let mut config = String::new();
        Self::save_entry(&mut config, "__last", self.last.as_ref());
        let mut names = Vec::from_iter(self.paths.keys().cloned());
        names.sort();
        for name in names {
            Self::save_entry(&mut config, &name, self.paths.get(&name));
        }
        if let Err(e) = write_settings_file(Self::FILENAME, &config) {
            logger.log(format!("ERROR writing path library: '{}'", e));
        }
    }
}
