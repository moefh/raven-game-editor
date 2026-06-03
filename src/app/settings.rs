use std::io::{Result, Error};
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use crate::data_asset::StringLogger;
use crate::data_asset::{Tokenizer, Token, TokenData};

const APP_ID: &str = "raven-game-editor";

fn get_settings_dir() -> Option<PathBuf> {
    use egui::os::OperatingSystem as OS;
    match OS::from_target_os() {
        OS::Nix => std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| std::env::home_dir().map(|p| p.join(".local").join("share")))
            .map(|p| { p.join(APP_ID) }),
        OS::Mac => std::env::home_dir().map(|p| {
            p.join("Library").join("Application Support").join(APP_ID)
        }),
        OS::Windows => std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .map(|p| p.join(APP_ID)),
        _ => None,
    }
}

fn write_settings_file<P: AsRef<Path>>(filename: P, content: &str) -> Result<()> {
    let dir = get_settings_dir().ok_or(Error::other("can't figure out config directory"))?;
    std::fs::create_dir_all(&dir)?;
    let filename = dir.join(filename.as_ref());
    std::fs::write(&filename, content)
}

pub struct AppSettings {
    pub theme: String,
    pub zoom: u32,
    pub start_maximized: bool,
    pub image_bg_color: egui::Color32,
    pub map_bg_color: egui::Color32,
    pub color_picker_bg_color: egui::Color32,
    pub image_grid_color: egui::Color32,
    pub map_grid_color: egui::Color32,
    pub marching_ants_delay: u32,
    pub marching_ants_dash_size: u32,
    pub marching_ants_thickness: u32,
    pub marching_ants_color1: egui::Color32,
    pub marching_ants_color2: egui::Color32,
}

impl AppSettings {
    const FILENAME: &str = "settings.txt";

    pub fn new() -> Self {
        AppSettings {
            theme: String::from("system"),
            zoom: 100,
            start_maximized: false,
            image_bg_color: egui::Color32::from_rgb(0xe0, 0xff, 0xff),
            map_bg_color: egui::Color32::from_rgb(0x80, 0x20, 0x80),
            color_picker_bg_color: egui::Color32::from_rgb(0xe0, 0xe0, 0xe0),
            image_grid_color: egui::Color32::BLACK,
            map_grid_color: egui::Color32::BLACK,
            marching_ants_delay: 100,
            marching_ants_dash_size: 5,
            marching_ants_thickness: 3,
            marching_ants_color1: egui::Color32::BLACK,
            marching_ants_color2: egui::Color32::WHITE,
        }
    }

    pub fn load(logger: &mut StringLogger) -> Self {
        let mut settings = AppSettings::new();
        settings.read(logger);
        settings
    }

    fn read_file<P: AsRef<Path>>(&mut self, filename: P) -> Result<()> {
        let config = std::fs::read_to_string(&filename)?;
        let mut reader = AppSettingsReader::new(&config);
        reader.read(self)
    }

    pub fn read(&mut self, logger: &mut StringLogger) {
        let settings_dir = match get_settings_dir() {
            Some(dir) => { dir }
            None => {
                logger.log("WARNING: can't find settings directory, settings won't be loaded");
                return;
            }
        };
        let filename = settings_dir.join(Self::FILENAME);
        logger.log(format!("Loading settings from '{}'", filename.display()));
        match self.read_file(filename) {
            Ok(_) => {}
            Err(e) => {
                logger.log(format!("ERROR reading settings: {}", e));
            }
        }
    }

    fn save_color(c: egui::Color32) -> String {
        format!("[{},{},{}]", c.r(), c.g(), c.b())
    }

    pub fn save(&self, logger: &mut StringLogger) {
        let mut config = String::new();
        config.push_str(&format!("zoom = {};\n", self.zoom));
        config.push_str(&format!("theme = \"{}\";\n", self.theme));
        config.push_str(&format!("start_maximized = {};\n", if self.start_maximized { 1 } else { 0 }));
        config.push_str(&format!("image_bg_color = {};\n", Self::save_color(self.image_bg_color)));
        config.push_str(&format!("map_bg_color = {};\n", Self::save_color(self.map_bg_color)));
        config.push_str(&format!("color_picker_bg_color = {};\n", Self::save_color(self.color_picker_bg_color)));
        config.push_str(&format!("image_grid_color = {};\n", Self::save_color(self.image_grid_color)));
        config.push_str(&format!("map_grid_color = {};\n", Self::save_color(self.map_grid_color)));
        config.push_str(&format!("marching_ants_delay = {};\n", self.marching_ants_delay));
        config.push_str(&format!("marching_ants_thickness = {};\n", self.marching_ants_thickness));
        config.push_str(&format!("marching_ants_dash_size = {};\n", self.marching_ants_dash_size));
        config.push_str(&format!("marching_ants_colors = [ {}, {} ];\n",
                                 Self::save_color(self.marching_ants_color1),
                                 Self::save_color(self.marching_ants_color2)));
        if let Err(e) = write_settings_file(Self::FILENAME, &config) {
            logger.log(format!("ERROR writing settings: '{}'", e));
        }
    }
}

struct AppSettingsReader<'a> {
    tok: Tokenizer<'a>,
}

impl<'a> AppSettingsReader<'a> {
    fn new(config: &'a str) -> Self {
        AppSettingsReader {
            tok: crate::data_asset::Tokenizer::new(config),
        }
    }

    fn expect_punct(&mut self, ch: char) -> Result<Token> {
        let t = self.tok.read()?;
        if ! t.is_punct(ch) {
            return Err(Error::other(format!("expected '{}', found '{}' at line {}", ch, t, t.pos.line)));
        }
        Ok(t)
    }

    fn skip_config_value(&mut self) -> Result<()> {
        loop {
            let t = self.tok.read()?;
            if t.is_eof() || t.is_punct(';') { break; }
        }
        Ok(())
    }

    fn read_number(&mut self) -> Result<u64> {
        let t = self.tok.read()?;
        if let Some(n) = t.get_number() {
            return Ok(n)
        }
        Err(Error::other(format!("expected number, found '{}' at line {}", t, t.pos.line)))
    }

    fn read_color(&mut self) -> Result<egui::Color32> {
        self.expect_punct('[')?;
        let r = self.read_number()?;
        self.expect_punct(',')?;
        let g = self.read_number()?;
        self.expect_punct(',')?;
        let b = self.read_number()?;
        self.expect_punct(']')?;

        Ok(egui::Color32::from_rgb(r as u8, g as u8, b as u8))
    }

    fn read_number_config(&mut self) -> Result<u32> {
        let n = self.read_number()?;
        self.expect_punct(';')?;

        Ok(n as u32)
    }

    fn read_string_config(&mut self) -> Result<String> {
        let t = self.tok.read()?;
        self.expect_punct(';')?;

        if let Some(s) = t.get_string() {
            return Ok(s.to_owned());
        }
        Err(Error::other(format!("expected string, found '{}' at line {}", t, t.pos.line)))
    }

    fn read_color_config(&mut self) -> Result<egui::Color32> {
        let color = self.read_color()?;
        self.expect_punct(';')?;

        Ok(color)
    }

    fn read_color_array_config(&mut self, colors: &mut [ egui::Color32 ]) -> Result<()> {
        for (i, color) in colors.iter_mut().enumerate() {
            self.expect_punct(if i == 0 { '[' } else { ',' })?;
            *color = self.read_color()?;
        }
        self.expect_punct(']')?;
        self.expect_punct(';')?;
        Ok(())
    }

    fn read(&mut self, settings: &mut AppSettings) -> Result<()> {
        loop {
            let t = self.tok.read()?;
            if t.is_eof() { break; }

            if let TokenData::Ident(ident) = t.data {
                self.expect_punct('=')?;
                match ident.as_str() {
                    "theme" => { settings.theme = self.read_string_config()?; }
                    "zoom" => { settings.zoom = self.read_number_config()?; }
                    "start_maximized" => { settings.start_maximized = self.read_number_config()? != 0; }
                    "image_bg_color" => { settings.image_bg_color = self.read_color_config()?; }
                    "map_bg_color" => { settings.map_bg_color = self.read_color_config()?; }
                    "color_picker_bg_color" => { settings.color_picker_bg_color = self.read_color_config()?; }
                    "image_grid_color" => { settings.image_grid_color = self.read_color_config()?; }
                    "map_grid_color" => { settings.map_grid_color = self.read_color_config()?; }
                    "marching_ants_delay" => { settings.marching_ants_delay = self.read_number_config()?; }
                    "marching_ants_thickness" => { settings.marching_ants_thickness = self.read_number_config()?; }
                    "marching_ants_dash_size" => { settings.marching_ants_dash_size = self.read_number_config()?; }
                    "marching_ants_colors" => {
                        let mut colors = [egui::Color32::BLACK, egui::Color32::WHITE];
                        self.read_color_array_config(&mut colors)?;
                        settings.marching_ants_color1 = colors[0];
                        settings.marching_ants_color2 = colors[1];
                    }
                    _ => {
                        self.skip_config_value()?;
                    }
                }
                continue;
            }
        }
        Ok(())
    }
}

pub struct AppPathLibrary {
    paths: HashMap<String, PathBuf>,
    last: Option<PathBuf>,
}

impl AppPathLibrary {
    const FILENAME: &str = "saved_paths.txt";

    pub fn new() -> Self {
        AppPathLibrary {
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
        let mut tok = crate::data_asset::Tokenizer::new(&config);
        loop {
            let t = tok.read()?;
            if t.is_eof() { break; }
            if let TokenData::Ident(name) = t.data {
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
