use std::io::{Result, Error};
use std::path::PathBuf;

use crate::image::{ColorSet, ColorSetCollection};
use crate::data_asset::StringLogger;
use crate::data_asset::{Tokenizer, Token, TokenData};

const APP_ID: &str = "raven-game-editor";

pub fn get_settings_dir() -> Option<PathBuf> {
    use egui::os::OperatingSystem as OS;
    match OS::from_target_os() {
        OS::Nix => std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| std::env::home_dir().map(|p| p.join(".config")))
            .map(|p| { p.join(APP_ID) }),
        OS::Mac => std::env::home_dir().map(|p| {
            p.join("Library").join("Preferences").join(APP_ID)
        }),
        OS::Windows => std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .map(|p| p.join(APP_ID)),
        _ => None,
    }
}

pub fn write_settings_file(filename: impl AsRef<str>, content: &str) -> Result<()> {
    let dir = get_settings_dir().ok_or(Error::other("can't figure out config directory"))?;
    std::fs::create_dir_all(&dir)?;
    let filename = dir.join(filename.as_ref());
    std::fs::write(&filename, content)
}

pub fn read_settings_file(filename: impl AsRef<str>) -> Result<String> {
    let dir = get_settings_dir().ok_or(Error::other("can't figure out config directory"))?;
    std::fs::create_dir_all(&dir)?;
    let filename = dir.join(filename.as_ref());
    std::fs::read_to_string(&filename)
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
    pub colorsets: ColorSetCollection,
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
            colorsets: ColorSetCollection::new(),
        }
    }

    fn load_settings_file(&mut self) -> Result<()> {
        let config = read_settings_file(Self::FILENAME)?;
        let mut reader = AppSettingsReader::new(&config);
        reader.read(self)
    }

    pub fn load(logger: &mut StringLogger) -> Self {
        let mut settings = AppSettings::new();
        if let Err(e) = settings.load_settings_file() {
            logger.log(format!("ERROR loading settings:\n{}", e));
        }
        settings
    }

    fn save_color(c: egui::Color32) -> String {
        format!("[{},{},{}]", c.r(), c.g(), c.b())
    }

    pub fn cleanup_ident(name: &str) -> String {
        let mut clean = String::new();
        for ch in name.chars() {
            if matches!(ch, 'A'..='Z' | 'a'..='z' | '0'..='9' | '_') {
                clean.push(ch);
            } else {
                clean.push('_');
            }
        }
        clean
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

        // colorsets
        config.push_str("colorsets = [\n");
        for colorset in self.colorsets.get_custom_colorsets() {
            config.push_str(&format!("  {} = [ ", Self::cleanup_ident(&colorset.name)));
            config.push_str(&colorset.colors.iter().map(|c| c.to_string()).collect::<Vec<String>>().join(","));
            config.push_str(" ],\n");
        }
        config.push_str("];\n");

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

    fn read_colorsets_config(&mut self) -> Result<Vec<ColorSet>> {
        let mut colorsets = Vec::new();

        self.expect_punct('[')?;
        loop {
            let next_name = loop {
                let t = self.tok.read()?;
                if t.is_punct(']') { break None; }
                if t.is_punct(',') { continue; }
                if let Some(name) = t.get_ident() {
                    break Some(name.to_owned())
                }
                return Err(Error::other(format!("expected colorset name identifier or ']', found '{}' at line {}", t, t.pos.line)));
            };

            let name = match next_name {
                Some(name) => { name }
                None => { break; }
            };
            self.expect_punct('=')?;
            self.expect_punct('[')?;
            let mut colors = Vec::new();
            loop {
                let t = self.tok.read()?;
                if t.is_punct(']') { break; }
                if t.is_punct(',') { continue; }
                if let Some(number) = t.get_number() {
                    colors.push((number & 0xff) as u8);
                } else {
                    return Err(Error::other(format!("expected number for color byte or ']', found '{}' at line {}", t, t.pos.line)));
                }
            }

            colorsets.push(ColorSet::new(name, colors));
        }

        self.expect_punct(';')?;

        Ok(colorsets)
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
                    "colorsets" => {
                        let custom_colorsets = self.read_colorsets_config()?;
                        settings.colorsets.clear_custom_colorsets();
                        for set in custom_colorsets.into_iter() {
                            settings.colorsets.add_custom_colorset(set);
                        }
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
