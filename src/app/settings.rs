use std::io::{Result, Error};
use std::collections::HashMap;

use crate::platform::{
    read_settings_file,
    write_settings_file,
};
use crate::platform::gamepad::{
    self,
    GamepadMapping,
    GamepadAxisMapping,
};
use crate::image::{
    ColorSet,
    ColorSetCollection,
};
use crate::data_asset::{
    StringLogger,
    Tokenizer,
    Token,
    TokenData
};

fn get_default_gamepad_mappings() -> HashMap<String, GamepadMapping> {
    HashMap::from([
        (String::from("Zikway HID gamepad (Vendor: 3537 Product: 1041)"), GamepadMapping {
            buttons: [
                gamepad::GAMEPAD_A,
                gamepad::GAMEPAD_B,
                0,
                gamepad::GAMEPAD_X,
                gamepad::GAMEPAD_Y,
                0,
                gamepad::GAMEPAD_LB,
                gamepad::GAMEPAD_RB,
                gamepad::GAMEPAD_LT,
                gamepad::GAMEPAD_RT,
                gamepad::GAMEPAD_SELECT,
                gamepad::GAMEPAD_START,
                gamepad::GAMEPAD_HOME,
                gamepad::GAMEPAD_L3,
                gamepad::GAMEPAD_R3,
                0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
            axes: [
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::new(gamepad::GAMEPAD_LEFT, gamepad::GAMEPAD_RIGHT),
                GamepadAxisMapping::new(gamepad::GAMEPAD_UP, gamepad::GAMEPAD_DOWN),
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
            ],
        }),

    ])
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
    pub tile_picker_zoom: u32,
    pub tile_picker_popup_zoom: u32,
    pub game_runner_ms_per_frame: u32,
    pub animation_ms_per_frame: u32,
    pub animation_delay: u32,
    pub marching_ants_delay: u32,
    pub marching_ants_dash_size: u32,
    pub marching_ants_thickness: u32,
    pub marching_ants_color1: egui::Color32,
    pub marching_ants_color2: egui::Color32,
    pub colorsets: ColorSetCollection,
    pub gamepad_mappings: HashMap<String, GamepadMapping>,
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
            image_grid_color: egui::Color32::from_rgb(0x80, 0x80, 0x80),
            map_grid_color: egui::Color32::from_rgb(0x80, 0x80, 0x80),
            tile_picker_zoom: 400,
            tile_picker_popup_zoom: 300,
            game_runner_ms_per_frame: 17,
            animation_ms_per_frame: 200,
            animation_delay: 50,
            marching_ants_delay: 100,
            marching_ants_dash_size: 5,
            marching_ants_thickness: 3,
            marching_ants_color1: egui::Color32::BLACK,
            marching_ants_color2: egui::Color32::WHITE,
            colorsets: ColorSetCollection::new(),
            gamepad_mappings: get_default_gamepad_mappings(),
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

    fn save_gamepad_mapping(id: &str, mapping: &GamepadMapping) -> String {
        fn save(id: &str, mapping: &GamepadMapping) -> std::result::Result<String,std::fmt::Error> {
            use std::fmt::Write;

            let mut out = String::with_capacity(1024);
            write!(&mut out, "  \"{}\" = [\n", id)?;
            write!(&mut out, "    [")?;
            for (i, button) in mapping.buttons.iter().enumerate() {
                if i.is_multiple_of(8) {
                    if i != 0 { write!(&mut out, ",")?; }
                    write!(&mut out, "\n      ")?;
                } else {
                    write!(&mut out, ", ")?;
                }
                write!(&mut out, "0x{:x}", button)?;
            }
            write!(&mut out, "\n")?;
            write!(&mut out, "    ],\n")?;
            write!(&mut out, "    [")?;
            for (i, axis) in mapping.axes.iter().enumerate() {
                if i.is_multiple_of(8) {
                    if i != 0 { write!(&mut out, ",")?; }
                    write!(&mut out, "\n      ")?;
                } else {
                    write!(&mut out, ", ")?;
                }
                write!(&mut out, "0x{:x},0x{:x}", axis.min, axis.max)?;
            }
            write!(&mut out, "\n")?;
            write!(&mut out, "    ]\n")?;
            write!(&mut out, "  ],\n")?;
            Ok(out)
        }
        save(id, mapping).unwrap_or(String::new())
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

        config.push_str(&format!("tile_picker_zoom = {};\n", self.tile_picker_zoom));
        config.push_str(&format!("tile_picker_popup_zoom = {};\n", self.tile_picker_popup_zoom));

        config.push_str(&format!("animation_delay = {};\n", self.animation_delay));
        config.push_str(&format!("animation_ms_per_frame = {};\n", self.animation_ms_per_frame));
        config.push_str(&format!("game_runner_ms_per_frame = {};\n", self.game_runner_ms_per_frame));

        config.push_str(&format!("marching_ants_delay = {};\n", self.marching_ants_delay));
        config.push_str(&format!("marching_ants_thickness = {};\n", self.marching_ants_thickness));
        config.push_str(&format!("marching_ants_dash_size = {};\n", self.marching_ants_dash_size));
        config.push_str(&format!(
            "marching_ants_colors = [ {}, {} ];\n",
            Self::save_color(self.marching_ants_color1),
            Self::save_color(self.marching_ants_color2)
        ));

        // colorsets
        config.push_str("colorsets = [\n");
        for colorset in self.colorsets.get_custom_colorsets() {
            config.push_str(&format!("  {} = [ ", Self::cleanup_ident(&colorset.name)));
            config.push_str(&colorset.colors.iter().map(|c| c.to_string()).collect::<Vec<String>>().join(","));
            config.push_str(" ],\n");
        }
        config.push_str("];\n");

        // gamepad mappings
        config.push_str("gamepad_mappings = {\n");
        for (id, mapping) in self.gamepad_mappings.iter() {
            config.push_str(&Self::save_gamepad_mapping(id, mapping));
        }
        config.push_str("};\n");

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

    fn read_number_array(&mut self, numbers: &mut [u32]) -> Result<()> {
        self.expect_punct('[')?;
        for (i, number) in numbers.iter_mut().enumerate() {
            if i != 0 { self.expect_punct(',')?; }
            *number = self.read_number()? as u32;
        }
        self.expect_punct(']')?;
        Ok(())
    }

    fn read_gamepad_axis_mapping_array(&mut self, axes: &mut [GamepadAxisMapping]) -> Result<()> {
        self.expect_punct('[')?;
        for (i, axis) in axes.iter_mut().enumerate() {
            if i != 0 { self.expect_punct(',')?; }
            axis.min = self.read_number()? as u32;
            self.expect_punct(',')?;
            axis.max = self.read_number()? as u32;
        }
        self.expect_punct(']')?;
        Ok(())
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
                let mut t = self.tok.read()?;
                if t.is_punct(']') { break None; }
                if t.is_punct(',') { continue; }
                if let Some(name) = t.drain_ident() {
                    break Some(name)
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

    fn read_gamepad_mappings_config(&mut self) -> Result<HashMap<String, GamepadMapping>> {
        let mut mappings = HashMap::new();

        self.expect_punct('{')?;
        loop {
            let mut t = self.tok.read()?;
            if t.is_punct('}') { break; }
            if t.is_punct(',') { continue; }
            if let Some(id) = t.drain_string() {
                let mut mapping = GamepadMapping::new();
                self.expect_punct('=')?;
                self.expect_punct('[')?;
                self.read_number_array(&mut mapping.buttons)?;
                self.expect_punct(',')?;
                self.read_gamepad_axis_mapping_array(&mut mapping.axes)?;
                self.expect_punct(']')?;
                mappings.insert(id, mapping);
            } else {
                return Err(Error::other(format!("expected string or ']', found '{}' at line {}", t, t.pos.line)));
            }
        }
        self.expect_punct(';')?;

        Ok(mappings)
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
                    "tile_picker_zoom" => { settings.tile_picker_zoom = self.read_number_config()?; }
                    "tile_picker_popup_zoom" => { settings.tile_picker_popup_zoom = self.read_number_config()?; }
                    "start_maximized" => { settings.start_maximized = self.read_number_config()? != 0; }
                    "image_bg_color" => { settings.image_bg_color = self.read_color_config()?; }
                    "map_bg_color" => { settings.map_bg_color = self.read_color_config()?; }
                    "color_picker_bg_color" => { settings.color_picker_bg_color = self.read_color_config()?; }
                    "image_grid_color" => { settings.image_grid_color = self.read_color_config()?; }
                    "map_grid_color" => { settings.map_grid_color = self.read_color_config()?; }
                    "animation_delay" => { settings.animation_delay = self.read_number_config()?; }
                    "animation_ms_per_frame" => { settings.animation_ms_per_frame = self.read_number_config()?; }
                    "game_runner_ms_per_frame" => { settings.game_runner_ms_per_frame = self.read_number_config()?; }
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
                    "gamepad_mappings" => {
                        settings.gamepad_mappings = self.read_gamepad_mappings_config()?;
                        for (id, map) in get_default_gamepad_mappings() {
                            if ! settings.gamepad_mappings.contains_key(&id) {
                                settings.gamepad_mappings.insert(id, map);
                            }
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
