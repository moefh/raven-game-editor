pub mod tokenizer;

use std::fs;
use std::collections::HashMap;
use std::path::Path;
use std::io::{Result, Error};
use regex::Regex;
use std::sync::LazyLock;
use tokenizer::{Tokenizer, Token, TokenData, TokenPosition};

use super::{
    StringLogger, DataAssetStore, DataAssetId, DataAsset,
    PropFont, ModData, ModSample, ModCell,
    RoomMap, RoomEntity, RoomTrigger,
};

const C_KEYWORDS : &[&str] = &[
    "static",
    "const",
    "struct",
    "enum",
];

const C_STRUCT_NAMES : &[&str] = &[
    "FONT",
    "PROP_FONT",
    "PAL_SPRITE",
    "MOD_CELL",
    "MOD_DATA",
    "SFX",
    "IMAGE",
    "MAP",
    "SPRITE_ANIMATION",
    "ROOM_MAP_INFO",
    "ROOM_ENTITY_INFO",
    "ROOM_TRIGGER_INFO",
    "ROOM",
];

static RE_PRE_PROCESSOR_DEFINE: LazyLock<Regex> = LazyLock::new(
    || Regex::new(r"^#\s*define\s+([A-Za-z0-9_]+)\s+(.*)$").unwrap());
static RE_VGA_BITS_PER_PIXEL: LazyLock<Regex> = LazyLock::new(
    || Regex::new(r"^([A-Za-z0-9_]+)_DATA_VGA_BITS_PER_PIXEL$").unwrap());
static RE_VGA_SYNC_BITS: LazyLock<Regex> = LazyLock::new(
    || Regex::new(r"^([A-Za-z0-9_]+)_DATA_VGA_SYNC_BITS$").unwrap());
static RE_PRE_PROCESSOR_IF: LazyLock<Regex> = LazyLock::new(
    || Regex::new(r"^#if").unwrap());
static RE_PRE_PROCESSOR_ENDIF: LazyLock<Regex> = LazyLock::new(
    || Regex::new(r"^#endif").unwrap());

fn image_6bit_u32_to_pixels(data: &[u32], width: u32, height: u32, num_items: u32, pixel_mapping: &[u8]) -> Vec<u8> {
    const COLOR_BITS: u32 = 0b0011_1111;

    let stride = width.div_ceil(4) as usize;
    let mut pixels = Vec::<u8>::with_capacity((width * height * num_items) as usize);
    for y in 0 .. (height * num_items) as usize {
        for x in 0..stride {
            let quad = data[y*stride + x];
            if x < stride-1 || width.is_multiple_of(4) {
                pixels.push(pixel_mapping[(quad         & COLOR_BITS) as usize]);
                pixels.push(pixel_mapping[((quad >>  8) & COLOR_BITS) as usize]);
                pixels.push(pixel_mapping[((quad >> 16) & COLOR_BITS) as usize]);
                pixels.push(pixel_mapping[((quad >> 24) & COLOR_BITS) as usize]);
            } else {
                match width % 4 {
                    1 => {
                        pixels.push(pixel_mapping[(quad         & COLOR_BITS) as usize]);
                    },
                    2 => {
                        pixels.push(pixel_mapping[(quad         & COLOR_BITS) as usize]);
                        pixels.push(pixel_mapping[((quad >>  8) & COLOR_BITS) as usize]);
                    },
                    3 => {
                        pixels.push(pixel_mapping[(quad         & COLOR_BITS) as usize]);
                        pixels.push(pixel_mapping[((quad >>  8) & COLOR_BITS) as usize]);
                        pixels.push(pixel_mapping[((quad >> 16) & COLOR_BITS) as usize]);
                    },
                    _ => {},
                }
            }
        }
    }
    pixels
}

fn image_8bit_u32_to_pixels(data: &[u32], width: u32, height: u32, num_items: u32) -> Vec<u8> {
    let stride = width.div_ceil(4) as usize;
    let mut pixels = Vec::<u8>::with_capacity((width * height * num_items) as usize);
    for y in 0 .. (height * num_items) as usize {
        for x in 0..stride {
            let quad = data[y*stride + x];
            if x < stride-1 || width.is_multiple_of(4) {
                pixels.push((quad      ) as u8);
                pixels.push((quad >>  8) as u8);
                pixels.push((quad >> 16) as u8);
                pixels.push((quad >> 24) as u8);
            } else {
                match width % 4 {
                    1 => {
                        pixels.push((quad      ) as u8);
                    },
                    2 => {
                        pixels.push((quad      ) as u8);
                        pixels.push((quad >>  8) as u8);
                    },
                    3 => {
                        pixels.push((quad      ) as u8);
                        pixels.push((quad >>  8) as u8);
                        pixels.push((quad >> 16) as u8);
                    },
                    _ => {},
                }
            }
        }
    }
    pixels
}

fn pal_image_to_pixels(data: &[u8], palette: &[u8], width: u32, height: u32, num_items: u32, bits_per_pixel: u32) -> Vec<u8> {
    let width = width as usize;
    let height = height as usize;
    let num_items = num_items as usize;
    let bits_per_pixel = bits_per_pixel as usize;
    let pixels_per_byte = 8 / bits_per_pixel;
    let stride = (width * bits_per_pixel).div_ceil(8);
    let palette_index_mask = (1u8 << bits_per_pixel) - 1;

    let mut pixels = vec![0u8; width * height * num_items];
    for y in 0 .. height * num_items {
        let mut block = 0u8;
        for x in 0 .. width {
            if x % pixels_per_byte == 0 {
                block = data[y * stride + x/pixels_per_byte];
            }
            pixels[y * width + x] = palette[(block & palette_index_mask) as usize];
            block >>= bits_per_pixel;
        }
    }

    pixels
}

struct I8or16Array {
    data_size: u32,  // 8 or 16
    data: Vec<i16>,  // elements will be left shift by 8 if size==8
}

pub struct ReadData {
    // asset data
    font_data: HashMap<String,Vec<u8>>,
    prop_font_data: HashMap<String,Vec<u8>>,
    mod_sample_data: HashMap<String,I8or16Array>,
    mod_patterns: HashMap<String,Vec<ModCell>>,
    sfx_sample_data: HashMap<String,I8or16Array>,
    tileset_data: HashMap<String,Vec<u32>>,
    sprite_data: HashMap<String,Vec<u32>>,
    pal_sprite_data: HashMap<String,Vec<u8>>,
    map_tiles: HashMap<String,Vec<u8>>,
    animation_frames: HashMap<String,Vec<u8>>,
    room_maps: HashMap<String,Vec<RoomMap>>,
    room_entities: HashMap<String,Vec<RoomEntity>>,
    room_triggers: HashMap<String,Vec<RoomTrigger>>,

    // assets by index
    fonts: Vec<DataAssetId>,
    prop_fonts: Vec<DataAssetId>,
    mods: Vec<DataAssetId>,
    sfxs: Vec<DataAssetId>,
    tilesets: Vec<DataAssetId>,
    sprites: Vec<DataAssetId>,
    pal_sprites: Vec<DataAssetId>,
    maps: Vec<DataAssetId>,
    animations: Vec<DataAssetId>,
    rooms: Vec<DataAssetId>,

    // assets by name
    fonts_by_name_id: HashMap<String, DataAssetId>,
    prop_fonts_by_name_id: HashMap<String, DataAssetId>,
    mods_by_name_id: HashMap<String, DataAssetId>,
    sfxs_by_name_id: HashMap<String, DataAssetId>,
    tilesets_by_name_id: HashMap<String, DataAssetId>,
    sprites_by_name_id: HashMap<String, DataAssetId>,
    pal_sprites_by_name_id: HashMap<String, DataAssetId>,
    maps_by_name_id: HashMap<String, DataAssetId>,
    animations_by_name_id: HashMap<String, DataAssetId>,
    rooms_by_name_id: HashMap<String, DataAssetId>,
}

pub struct ProjectDataReader<'a> {
    logger: &'a mut StringLogger,
    store: &'a mut DataAssetStore,
    tok: Tokenizer<'a>,
    unread_token: Option<Token>,
    last_pos: TokenPosition,
    read_data: ReadData,
    prefix_lower: String,
    prefix_upper: String,
    last_type_size: u32,
    pixel_6bit_to_8bit: Vec<u8>,
}

fn error<T, S: AsRef<str>>(msg: S, pos: TokenPosition) -> Result<T> {
    Result::Err(Error::other(format!("line {}: {}", pos.line, msg.as_ref())))
}

fn error_expected<T, S: AsRef<str>>(expected: S, found: &Token) -> Result<T> {
    Result::Err(Error::other(format!("line {}: expected {}, found {}",
                                     found.pos.line, expected.as_ref(), found)))
}

#[allow(dead_code)]
impl<'a> ProjectDataReader<'a> {

    fn new(source: &'a str, store: &'a mut DataAssetStore, logger: &'a mut StringLogger) -> Self {

        ProjectDataReader {
            logger,
            store,
            tok: Tokenizer::new(source),
            unread_token: None,
            last_pos: TokenPosition { line: 0 },
            last_type_size: 0,
            prefix_lower: String::new(),
            prefix_upper: String::new(),
            pixel_6bit_to_8bit: Self::gen_6bit_to_8bit_map(),

            read_data: ReadData {
                font_data: HashMap::new(),
                prop_font_data: HashMap::new(),
                mod_sample_data: HashMap::new(),
                mod_patterns: HashMap::new(),
                sfx_sample_data: HashMap::new(),
                tileset_data: HashMap::new(),
                sprite_data: HashMap::new(),
                pal_sprite_data: HashMap::new(),
                map_tiles: HashMap::new(),
                animation_frames: HashMap::new(),
                room_maps: HashMap::new(),
                room_entities: HashMap::new(),
                room_triggers: HashMap::new(),

                fonts: Vec::new(),
                prop_fonts: Vec::new(),
                mods: Vec::new(),
                sfxs: Vec::new(),
                tilesets: Vec::new(),
                sprites: Vec::new(),
                pal_sprites: Vec::new(),
                maps: Vec::new(),
                animations: Vec::new(),
                rooms: Vec::new(),

                fonts_by_name_id: HashMap::new(),
                prop_fonts_by_name_id: HashMap::new(),
                mods_by_name_id: HashMap::new(),
                sfxs_by_name_id: HashMap::new(),
                tilesets_by_name_id: HashMap::new(),
                sprites_by_name_id: HashMap::new(),
                pal_sprites_by_name_id: HashMap::new(),
                maps_by_name_id: HashMap::new(),
                animations_by_name_id: HashMap::new(),
                rooms_by_name_id: HashMap::new(),
            },
        }
    }

    fn gen_6bit_to_8bit_map() -> Vec<u8> {
        let mut pixel_6bit_to_8bit = vec![0u8; 64];
        for (pix_6bit, pix_8bit) in pixel_6bit_to_8bit.iter_mut().enumerate() {
            let r6 = pix_6bit & 0x03;
            let g6 = (pix_6bit & 0x0c) >> 2;
            let b6 = (pix_6bit & 0x30) >> 4;
            let r8 = (r6 << 1) | (r6 >> 1);
            let g8 = (g6 << 1) | (g6 >> 1);
            let b8 = b6;
            *pix_8bit = (r8 | (g8 << 3) | (b8 << 6)) as u8;
        }
        pixel_6bit_to_8bit
    }

    fn read(&mut self) -> Result<Token> {
        if let Some(t) = self.unread_token.take() {
            self.last_pos.line = t.pos.line;
            return Ok(t);
        }
        let t = self.tok.read()?;
        self.last_pos.line = t.pos.line;
        Ok(t)
    }

    fn unread(&mut self, t: Token) -> Result<()> {
        if self.unread_token.is_none() {
            self.unread_token = Some(t);
            return Ok(());
        }
        error("trying to unread a token while a token is already unread", t.pos)?
    }

    fn expect_token(&mut self) -> Result<Token> {
        match self.read() {
            Ok(tok) if ! tok.is_eof() => Ok(tok),
            v => v,
        }
    }

    fn expect_ident(&mut self, id: &str) -> Result<Token> {
        let t = self.tok.read()?;
        if t.is_ident(id) {
            return Ok(t)
        }
        error(format!("expected '{}', found '{}'", id, t), t.pos)?
    }

    fn expect_any_ident(&mut self, expected: &str) -> Result<Token> {
        let t = self.tok.read()?;
        if t.is_any_ident() {
            return Ok(t);
        }
        error(format!("expected {}, found '{}'", expected, t), t.pos)?
    }

    fn expect_punct(&mut self, ch: char) -> Result<Token> {
        let t = self.tok.read()?;
        if t.is_punct(ch) {
            return Ok(t)
        }
        error(format!("expected '{}', found '{}'", ch, t), t.pos)?
    }

    fn expect_any_punct(&mut self, expected: &str) -> Result<Token> {
        let t = self.tok.read()?;
        if t.is_any_punct() {
            return Ok(t)
        }
        error(format!("expected {}, found '{}'", expected, t), t.pos)
    }

    fn expect_number(&mut self, expected: &str) -> Result<Token> {
        let t = self.tok.read()?;
        if t.is_any_number() {
            return Ok(t)
        }
        error(format!("expected {}, found '{}'", expected, t), t.pos)
    }

    fn read_number(&mut self) -> Result<u64> {
        let t = self.tok.read()?;
        if let Some(n) = t.get_number() {
            return Ok(n)
        }
        error(format!("expected number, found '{}'", t), t.pos)
    }

    fn read_signed_number(&mut self) -> Result<i64> {
        let t = self.tok.read()?;
        if let Some(n) = t.get_number() {
            return Ok(n as i64)
        }
        if t.is_punct('-') {
            let t = self.tok.read()?;
            if let Some(n) = t.get_number() {
                return Ok(-(n as i64))
            }
            error(format!("expected number, found '{}'", t), t.pos)?
        }
        error(format!("expected '-' or number, found '{}'", t), t.pos)?
    }

    fn parse_number(source: &str) -> Option<u64> {
        let mut num_tok = Tokenizer::new(source);
        match num_tok.read() {
            Ok(t) => t.get_number(),
            Err(_) => None,
        }
    }

    fn get_image_pixels(&self, data: &[u32], width: u32, height: u32, num_items: u32) -> Vec<u8> {
        if self.store.vga_bits_per_pixel == 8 {
            image_8bit_u32_to_pixels(data, width, height, num_items)
        } else {
            image_6bit_u32_to_pixels(data, width, height, num_items, &self.pixel_6bit_to_8bit)
        }
    }

    fn handle_pre_processor_define(&mut self, name_id: &str, value: &str) {
        if name_id.starts_with(&format!("{}SPRITE_WIDTH_", self.prefix_upper)) ||
            name_id.starts_with(&format!("{}SPRITE_HEIGHT_", self.prefix_upper)) ||
            name_id.starts_with(&format!("{}SPRITE_STRIDE_", self.prefix_upper)) ||
            name_id.starts_with(&format!("{}SPRITE_FRAMES_", self.prefix_upper)) {
                return;
            }
        if name_id.starts_with(&format!("{}PAL_SPRITE_WIDTH_", self.prefix_upper)) ||
            name_id.starts_with(&format!("{}PAL_SPRITE_HEIGHT_", self.prefix_upper)) ||
            name_id.starts_with(&format!("{}PAL_SPRITE_FRAMES_", self.prefix_upper)) ||
            name_id.starts_with(&format!("{}PAL_SPRITE_DEPTH_", self.prefix_upper)) {
                return;
            }

        if name_id == format!("{}DATA_SAVE_TIMESTAMP", self.prefix_upper) { return; }

        self.logger.log(format!("-> ignoring define '{}' = '{}'", name_id, value));
    }

    fn handle_pre_processor_if(&mut self, line: &str) {

        if line == format!("#if {}DATA_BYTES", self.prefix_upper) { return; }
        if line == format!("#endif /* {}DATA_BYTES */", self.prefix_upper) { return; }

        self.logger.log(format!("-> ignoring pre-processor if line: {}", line));
    }

    fn handle_pre_processor_other(&mut self, line: &str) {
        self.logger.log(format!("-> ignoring unknown pre-processor line: {}", line));
    }

    fn handle_pre_processor_non_define(&mut self, line: &str) {
        if RE_PRE_PROCESSOR_IF.is_match(line) || RE_PRE_PROCESSOR_ENDIF.is_match(line) {
            self.handle_pre_processor_if(line);
        } else {
            self.handle_pre_processor_other(line);
        }
    }

    fn handle_pre_processor_line(&mut self, line: &str) {
        if let Some(define) = RE_PRE_PROCESSOR_DEFINE.captures(line) &&
            let Some(name) = define.get(1) &&
            let Some(value) = define.get(2) {
                self.handle_pre_processor_define(name.as_str(), value.as_str());
                return;
            }
        self.handle_pre_processor_non_define(line);
    }

    // return "<x>" for "<prefix>_<type_name>_<x>"
    fn get_global_lower_of_type<'x>(&self, ident: &'x str, type_name_id: &str) -> Option<&'x str> {
        if ! ident.starts_with(&self.prefix_lower) { return None; }
        let ident_no_prefix = &ident[self.prefix_lower.len()..];
        if ! ident_no_prefix.starts_with(type_name_id) { return None; }
        let ident_no_type = &ident_no_prefix[type_name_id.len()..];
        if ! ident_no_type.starts_with("_") { return None; }
        Some(&ident_no_type[1..])
    }

    // return "<x>" for "<prefix>_<x>"
    fn get_global_lower<'x>(&self, ident: &'x str) -> Option<&'x str> {
        if ! ident.starts_with(&self.prefix_lower) { return None; }
        Some(&ident[self.prefix_lower.len()..])
    }

    // compare to "<prefix>_<name>"
    fn is_global_lower(&self, ident: &str, name_id: &str) -> bool {
        if let Some(ident_name_id) = self.get_global_lower(ident) {
            return ident_name_id == name_id;
        }
        false
    }

    // return "<X>" for "<PREFIX>_<TYPE_NAME>_<X>_<SUFFIX>"
    fn get_global_upper_of_type_with_suffix<'x>(&self, ident: &'x str, type_name_id: &str, suffix: &str) -> Option<&'x str> {
        let name_id_suffix = match self.get_global_upper_of_type(ident, type_name_id) {
            Some(name_id) => name_id,
            None => { return None; },
        };

        if ! name_id_suffix.ends_with(suffix) { return None; }
        let name_id_no_suffix = &name_id_suffix[..name_id_suffix.len()-suffix.len()];
        if ! name_id_no_suffix.ends_with("_") { return None; }
        Some(&name_id_no_suffix[..name_id_no_suffix.len()-1])
    }

    // return "<X>" for "<PREFIX>_<TYPE_NAME>_<X>"
    fn get_global_upper_of_type<'x>(&self, ident: &'x str, type_name_id: &str) -> Option<&'x str> {
        if ! ident.starts_with(&self.prefix_upper) { return None; }
        let ident_no_prefix = &ident[self.prefix_upper.len()..];
        if ! ident_no_prefix.starts_with(type_name_id) { return None; }
        let ident_no_type = &ident_no_prefix[type_name_id.len()..];
        if ! ident_no_type.starts_with("_") { return None; }
        Some(&ident_no_type[1..])
    }

    // starts with "<PREFIX>_<TYPE_NAME>_"
    fn is_global_upper_of_type(&self, ident: &str, type_name_id: &str) -> bool {
        if ! ident.starts_with(&self.prefix_upper) { return false; }
        let ident_no_prefix = &ident[self.prefix_upper.len()..];
        if ! ident_no_prefix.starts_with(type_name_id) { return false; }
        let ident_no_type = &ident_no_prefix[type_name_id.len()..];
        if ! ident_no_type.starts_with("_") { return false; }
        true
    }

    // return "<X>" for "<PREFIX>_<X>_<SUFFIX>"
    fn get_global_upper_with_suffix<'x>(&self, ident: &'x str, suffix: &str) -> Option<&'x str> {
        let name_suffix = match self.get_global_upper(ident) {
            Some(name) => name,
            None => { return None; },
        };

        if ! name_suffix.ends_with(suffix) { return None; }
        let name_no_suffix = &name_suffix[..name_suffix.len()-suffix.len()];
        if ! name_no_suffix.ends_with("_") { return None; }
        Some(&name_no_suffix[..name_no_suffix.len()-1])
    }

    // return "<X>" for "<PREFIX>_<X>"
    fn get_global_upper<'x>(&self, ident: &'x str) -> Option<&'x str> {
        if ! ident.starts_with(&self.prefix_upper) { return None; }
        Some(&ident[self.prefix_upper.len()..])
    }

    fn set_project_prefix(&mut self, prefix: &str) {
        self.logger.log(format!("-> got project prefix '{}'", prefix));
        self.prefix_upper.push_str(prefix);
        self.prefix_upper.push('_');
        self.prefix_upper.make_ascii_uppercase();
        self.prefix_lower.push_str(prefix);
        self.prefix_lower.push('_');
        self.prefix_lower.make_ascii_lowercase();
        self.store.project_prefix = prefix.to_owned();
    }

    fn read_project_defines(&mut self) -> Result<()> {
        let mut got_prefix = false;
        loop {
            let t = self.expect_token()?;

            if let Some(line) = t.get_pre_processor() {
                // pre-processor line
                if let Some(define) = RE_PRE_PROCESSOR_DEFINE.captures(line) &&
                    let Some(name) = define.get(1) &&
                    let Some(value) = define.get(2) {
                        // #define
                        if let Some(name_prefix) = RE_VGA_SYNC_BITS.captures(name.as_str()) &&
                            let Some(prefix) = name_prefix.get(1) {
                                // #define <PREFIX>_DATA_VGA_SYNC_BITS
                                match Self::parse_number(value.as_str()) {
                                    Some(vga_sync_bits) => {
                                        if vga_sync_bits > 0xff {
                                            return error(format!("bad vga_sync_bits value: {:#x}", vga_sync_bits), t.pos);
                                        }
                                        if ! got_prefix {
                                            self.set_project_prefix(prefix.as_str());
                                            got_prefix = true;
                                        } else if self.store.project_prefix != prefix.as_str() {
                                            return error(format!("define for vga_sync_bits has invalid prefix: {}", name.as_str()), t.pos);
                                        }
                                        self.logger.log(format!("-> got vga_sync_bits {:#04x}", vga_sync_bits));
                                        self.store.vga_sync_bits = vga_sync_bits as u8;
                                        continue;
                                    },
                                    None => {
                                        return error(format!("bad vga_sync_bits value: {}", value.as_str()), t.pos);
                                    }
                                }
                            }
                        if let Some(name_prefix) = RE_VGA_BITS_PER_PIXEL.captures(name.as_str()) &&
                            let Some(prefix) = name_prefix.get(1) {
                                // #define <PREFIX>_DATA_VGA_BITS_PER_PIXEL
                                match Self::parse_number(value.as_str()) {
                                    Some(vga_bits_per_pixel) => {
                                        if vga_bits_per_pixel != 6 && vga_bits_per_pixel != 8 {
                                            return error(format!("bad vga_bits_per_pixel value: {} (only 8 and 6 are supported)",
                                                                 vga_bits_per_pixel), t.pos);
                                        }
                                        if ! got_prefix {
                                            self.set_project_prefix(prefix.as_str());
                                            got_prefix = true;
                                        } else if self.store.project_prefix != prefix.as_str() {
                                            return error(format!("define for vga_bits_per_pixel has invalid prefix: {}", name.as_str()), t.pos);
                                        }
                                        self.logger.log(format!("-> got vga_bits_per_pixel {:#04x}", vga_bits_per_pixel));
                                        self.store.vga_bits_per_pixel = vga_bits_per_pixel as u8;
                                        continue;
                                    },
                                    None => {
                                        return error(format!("bad vga_bits_per_pixel value: {}", value.as_str()), t.pos);
                                    }
                                }
                            }
                        self.handle_pre_processor_define(name.as_str(), value.as_str());
                        continue;
                    }

                self.handle_pre_processor_non_define(line);
                continue;
            }

            // anything other than pre-processor line:
            if ! got_prefix {
                return error(format!("must have define for vga_sync_bits before this: {}", &t), t.pos);
            }
            return self.unread(t);
        }
    }

    // =======================================================================================
    // === ARRAYS
    // =======================================================================================

    fn read_u8_array(&mut self) -> Result<Vec<u8>> {
        self.expect_punct('{')?;

        let mut data = Vec::<u8>::new();

        loop {
            let t = self.read()?;
            if t.is_punct('}') { break; }
            if let Some(n) = t.get_number() {
                if n > 0xff {
                    error(format!("array element is too large (expected 0 <= {} <= 255)", n), self.last_pos)?;
                }
                data.push(n as u8);
            } else {
                return error_expected("number", &t)?;
            }

            let next = self.expect_any_punct("',' or '}'")?;
            if next.is_punct('}') { break; }
            if ! next.is_punct(',') {
                error_expected("',' or '}'", &next)?;
            }
        }

        Ok(data)
    }

    fn read_u16_array(&mut self) -> Result<Vec<u16>> {
        self.expect_punct('{')?;

        let mut data = Vec::<u16>::new();

        loop {
            let t = self.read()?;
            if t.is_punct('}') { break; }
            if let Some(n) = t.get_number() {
                if n > 0xffff {
                    error(format!("array element is too large (expected 0 <= {} <= 65535)", n), self.last_pos)?;
                }
                data.push(n as u16);
            } else {
                return error_expected("number", &t)?;
            }

            let next = self.expect_any_punct("',' or '}'")?;
            if next.is_punct('}') { break; }
            if ! next.is_punct(',') {
                error_expected("',' or '}'", &next)?;
            }
        }

        Ok(data)
    }

    fn read_u32_array(&mut self) -> Result<Vec<u32>> {
        self.expect_punct('{')?;

        let mut data = Vec::<u32>::new();

        loop {
            let t = self.read()?;
            if t.is_punct('}') { break; }
            if let Some(n) = t.get_number() {
                if n > 0xffffffff {
                    error(format!("array element is too large (expected 0 <= {} <= 0xffffffff)", n), self.last_pos)?;
                }
                data.push(n as u32);
            } else {
                return error_expected("number", &t)?;
            }

            let next = self.expect_any_punct("',' or '}'")?;
            if next.is_punct('}') { break; }
            if ! next.is_punct(',') {
                error_expected("',' or '}'", &next)?;
            }
        }

        Ok(data)
    }

    fn read_i8or16_array(&mut self, data_size: u32) -> Result<I8or16Array> {
        let t = self.expect_punct('{')?;

        let (min_el, max_el) = match data_size {
            8 => (-128, 127),
            16 => (-32768, 32767),
            _ => return error(format!("invalid array element size: {} (must be 8 or 16)", data_size), t.pos)?,
        };
        let mut data = Vec::<i16>::new();

        loop {
            let t = self.read()?;
            if t.is_punct('}') { break; }
            let n = match t {
                Token{ data: TokenData::Number(n), .. } => n as i64,
                Token{ data: TokenData::Punct('-'), .. } => {
                    let t = self.expect_token()?;
                    if let Some(n) = t.get_number() {
                        -(n as i64)
                    } else {
                        return error_expected("number", &t)?;
                    }
                },
                _ => {
                    return error_expected("'-' or number", &t)?;
                },
            };
            if (n < min_el) || (n > max_el) {
                error(format!("invalid array element value (expected {} <= {} <= {})", min_el, n, max_el), self.last_pos)?;
            }
            if data_size == 8 {
                data.push((n<<8) as i16);
            } else {
                data.push(n as i16);
            }

            let next = self.expect_any_punct("',' or '}'")?;
            if next.is_punct('}') { break; }
            if ! next.is_punct(',') {
                error_expected("',' or '}'", &next)?;
            }
        }

        Ok(I8or16Array {
            data_size,
            data,
        })
    }

    // =======================================================================================
    // === REFERENCES ("&<prefix>_<type>[<index>]")
    // =======================================================================================

    fn read_asset_index_reference(&mut self, type_name_id: &str) -> Result<DataAssetId> {
        self.expect_punct('&')?;
        let name_id_tok = self.expect_token()?;
        self.expect_punct('[')?;
        let index = self.read_number()? as usize;
        self.expect_punct(']')?;

        if let Some(ident) = name_id_tok.get_ident() && self.is_global_lower(ident, type_name_id) {} else {
            error(format!("invalid global name for {}: '{}'", type_name_id, name_id_tok), name_id_tok.pos)?
        }

        if let Some(id) = match type_name_id {
            "tilesets" => self.read_data.tilesets.get(index),
            "maps" => self.read_data.maps.get(index),
            "sprites" => self.read_data.sprites.get(index),
            "sprite_animations" => self.read_data.animations.get(index),
            _ => error(format!("internal error: unexpected asset type '{}' as data_asset/reader/mod.rs:{}", name_id_tok, line!()),
                       name_id_tok.pos)?,
        } {
            Ok(*id)
        } else {
            error(format!("index {} not found in {}", index, name_id_tok), name_id_tok.pos)
        }
    }

    // =======================================================================================
    // === FONT
    // =======================================================================================

    fn read_font_data(&mut self, name_id: &str) -> Result<()> {
        self.expect_punct('[')?;
        self.expect_punct(']')?;
        self.expect_punct('=')?;

        let data = self.read_u8_array()?;

        self.expect_punct(';')?;

        self.logger.log(format!("-> got font data '{}'", name_id));
        self.read_data.font_data.insert(name_id.to_string(), data);

        Ok(())
    }

    fn read_fonts(&mut self) -> Result<()> {
        self.expect_punct('[')?;
        self.expect_punct(']')?;
        self.expect_punct('=')?;
        self.expect_punct('{')?;

        loop {
            let t = self.expect_any_punct("'{' or '}'")?;
            if t.is_punct('}') { break; }
            if ! t.is_punct('{') { return error(format!("expected '{{' or '}}', got {}", t), t.pos)?; }

            let width = self.read_number()? as u32;
            self.expect_punct(',')?;
            let height = self.read_number()? as u32;
            self.expect_punct(',')?;
            let ident = self.expect_any_ident("font data identifier")?;
            self.expect_punct('}')?;
            self.expect_punct(',')?;

            let full_name_id = match ident.get_ident() {
                Some(ident) => ident,
                None => { return error_expected("font_data_...", &t)?; },
            };
            let (name_id, data) = if let Some(name_id) = self.get_global_lower_of_type(full_name_id, "font_data") &&
                let Some(data) = self.read_data.font_data.get(name_id) {
                    (name_id, data)
                } else {
                    return error(format!("font data not found: '{}'", full_name_id), ident.pos)?;
                };

            let data = super::font::CreationData {
                width,
                height,
                data,
            };
            if let Some(id) = self.store.add_font_from(DataAsset::identifier_to_name(name_id), data) {
                self.read_data.fonts.push(id);
                self.read_data.fonts_by_name_id.insert(name_id.to_string(), id);
                self.logger.log(format!("  -> added FONT '{}' id={}", name_id, id));
            } else {
                return error(format!("error adding font '{}'", name_id), ident.pos)?;
            }
        }

        self.expect_punct(';')?;
        Ok(())
    }

    // =======================================================================================
    // === PROP FONT
    // =======================================================================================

    fn read_prop_font_data(&mut self, name_id: &str) -> Result<()> {
        self.expect_punct('[')?;
        self.expect_punct(']')?;
        self.expect_punct('=')?;

        let data = self.read_u8_array()?;

        self.expect_punct(';')?;

        self.logger.log(format!("-> got font data '{}'", name_id));
        self.read_data.prop_font_data.insert(name_id.to_string(), data);

        Ok(())
    }

    fn read_prop_fonts(&mut self) -> Result<()> {
        self.expect_punct('[')?;
        self.expect_punct(']')?;
        self.expect_punct('=')?;
        self.expect_punct('{')?;

        loop {
            let t = self.expect_any_punct("'{' or '}'")?;
            if t.is_punct('}') { break; }
            if ! t.is_punct('{') { return error(format!("expected '{{' or '}}', got {}", t), t.pos)?; }

            let height = self.read_number()? as u32;
            self.expect_punct(',')?;
            let data_ident = self.expect_any_ident("prop font data identifier")?;
            self.expect_punct(',')?;
            let char_widths = self.read_u8_array()?;
            self.expect_punct(',')?;
            let char_offsets = self.read_u16_array()?;
            self.expect_punct('}')?;
            self.expect_punct(',')?;

            if char_widths.len() != PropFont::NUM_CHARS as usize {
                error(format!("invalid prop font char widths length: expected {}, fount {}", PropFont::NUM_CHARS, char_widths.len()), t.pos)?
            }
            if char_offsets.len() != PropFont::NUM_CHARS as usize {
                error(format!("invalid prop font char offsets length: expected {}, fount {}", PropFont::NUM_CHARS, char_offsets.len()), t.pos)?
            }

            let full_name_id = match data_ident.get_ident() {
                Some(ident) => ident,
                None => { return error_expected("prop_font_data_...", &t)?; },
            };
            let (name_id, data) = if let Some(name_id) = self.get_global_lower_of_type(full_name_id, "prop_font_data") &&
                let Some(data) = self.read_data.prop_font_data.get(name_id) {
                    (name_id, data)
                } else {
                    error(format!("prop font data not found: '{}'", full_name_id), data_ident.pos)?
                };

            let data = super::prop_font::CreationData {
                height,
                data,
                char_widths,
                char_offsets,
            };
            if let Some(id) = self.store.add_prop_font_from(DataAsset::identifier_to_name(name_id), data) {
                self.read_data.prop_fonts.push(id);
                self.read_data.prop_fonts_by_name_id.insert(name_id.to_string(), id);
                self.logger.log(format!("  -> added PROP_FONT '{}' id={}", name_id, id));
            } else {
                return error(format!("error adding prop font '{}'", name_id), data_ident.pos)?;
            }
        }

        self.expect_punct(';')?;
        Ok(())
    }

    // =======================================================================================
    // === MOD
    // =======================================================================================

    fn read_mod_sample_data(&mut self, name_id: &str) -> Result<()> {
        self.expect_punct('[')?;
        self.expect_punct(']')?;
        self.expect_punct('=')?;

        let data = self.read_i8or16_array(self.last_type_size)?;

        self.expect_punct(';')?;

        self.logger.log(format!("-> got mod sample data '{}'", name_id));
        self.read_data.mod_sample_data.insert(name_id.to_string(), data);
        Ok(())
    }

    fn read_mod_pattern(&mut self, name_id: &str) -> Result<()> {
        self.expect_punct('[')?;
        self.expect_punct(']')?;
        self.expect_punct('=')?;
        self.expect_punct('{')?;

        let mut pattern = Vec::<ModCell>::new();

        loop {
            let t = self.expect_token()?;
            if t.is_punct('}') { break; }
            if ! t.is_punct('{') {
                error_expected("'{{' or '}}'", &t)?;
            }

            let sample = self.read_number()?;
            self.expect_punct(',')?;
            let note_index = self.read_number()?;
            self.expect_punct(',')?;
            let effect = self.read_number()?;
            self.expect_punct(',')?;

            self.expect_punct('}')?;
            self.expect_punct(',')?;

            pattern.push(ModCell {
                sample: sample as u8,
                period: if note_index == 0xff { 0 } else { ModData::get_note_period((note_index % 12) as i32, (note_index / 12) as i32) },
                effect: effect as u16,
            });
        }

        self.expect_punct(';')?;

        self.logger.log(format!("-> got mod pattern '{}'", name_id));
        self.read_data.mod_patterns.insert(name_id.to_string(), pattern);
        Ok(())
    }

    fn read_mod_sample_defs(&mut self) -> Result<Vec<ModSample>> {
        self.expect_punct('{')?;

        let mut sample_defs = Vec::<ModSample>::new();

        loop {
            let t = self.expect_token()?;
            if t.is_punct('}') { break; }
            if ! t.is_punct('{') {
                error_expected("'{{' or '}}'", &t)?;
            }

            let len = self.read_number()?;
            self.expect_punct(',')?;
            let loop_start = self.read_number()?;
            self.expect_punct(',')?;
            let loop_len = self.read_number()?;
            self.expect_punct(',')?;
            let finetune = self.read_number()?;
            self.expect_punct(',')?;
            let volume = self.read_number()?;
            self.expect_punct(',')?;
            let bits_per_sample = self.read_number()?;
            self.expect_punct(',')?;
            self.expect_punct('{')?;
            self.expect_punct('.')?;
            self.expect_any_ident("'data', 'data8' or 'data16'")?;
            self.expect_punct('=')?;
            let data_ident = self.expect_any_ident("NULL or sample data")?;
            self.expect_punct('}')?;
            self.expect_punct(',')?;
            self.expect_punct('}')?;
            self.expect_punct(',')?;

            let sample_full_name_id = match data_ident.get_ident() {
                Some(ident) => ident,
                None => { error(format!("invalid sample data: '{}'", data_ident), data_ident.pos)? },
            };
            let samples = if sample_full_name_id == "NULL" {
                None
            } else if let Some(sample_name) = self.get_global_lower_of_type(sample_full_name_id, "mod_samples") &&
                let Some(sample_data) = self.read_data.mod_sample_data.get(sample_name) {
                    if sample_data.data_size == bits_per_sample as u32 {
                        Some(sample_data.data.clone())   // samples may be shared between mods
                    } else {
                        error(format!("invalid sample: data has {} bits per sample, but sample definition wants {}",
                                           sample_data.data_size, bits_per_sample), data_ident.pos)?
                    }
                } else {
                    error(format!("sample data not found: '{}'", sample_full_name_id), data_ident.pos)?
                };

            sample_defs.push(ModSample {
                len: len as u32,
                loop_start: loop_start as u32,
                loop_len: loop_len as u32,
                finetune: if finetune > 7 { finetune as i8 - 16 } else { finetune as i8 },
                volume: volume as u8,
                bits_per_sample: bits_per_sample as u16,
                data: samples,
            });
        }

        Ok(sample_defs)
    }

    fn read_mod(&mut self) -> Result<()> {
        let sample_defs = self.read_mod_sample_defs()?;
        self.expect_punct(',')?;

        let num_channels = self.read_number()? as u8;
        self.expect_punct(',')?;

        let num_song_positions = self.read_number()? as usize;
        self.expect_punct(',')?;
        let song_positions = self.read_u8_array()?;
        self.expect_punct(',')?;

        let num_patterns = self.read_number()? as usize;
        self.expect_punct(',')?;
        let pattern_ident = self.expect_any_ident("pattern data")?;

        self.expect_punct(',')?;
        self.expect_punct('}')?;

        let (name_id, pattern) = if let Some(full_pattern_name) = pattern_ident.get_ident() &&
            let Some(name) = self.get_global_lower_of_type(full_pattern_name, "mod_pattern") &&
            let Some(pattern) = self.read_data.mod_patterns.get(name) {
                (name, pattern)
            } else {
                error(format!("mod pattern not found: {}", pattern_ident), pattern_ident.pos)?
            };

        if num_song_positions != song_positions.len() {
            error(format!("mod with invalid num song positions: expected {}, got {}",
                          song_positions.len(), num_song_positions), pattern_ident.pos)?
        }

        let expected_num_patterns = pattern.len() / (num_channels as usize * 64);
        if num_patterns != expected_num_patterns {
            error(format!("mod with invalid num patterns: expected {}, got {}",
                          expected_num_patterns, num_patterns), pattern_ident.pos)?
        }

        let data = super::mod_data::CreationData {
            num_channels,
            samples: sample_defs,
            pattern,
            song_positions,
        };
        if let Some(id) = self.store.add_mod_from(DataAsset::identifier_to_name(name_id), data) {
            self.read_data.mods.push(id);
            self.read_data.mods_by_name_id.insert(name_id.to_string(), id);
            self.logger.log(format!("  -> added MOD '{}' id={}", name_id, id));
        } else {
            return error(format!("error adding mod '{}'", name_id), pattern_ident.pos)?;
        }
        Ok(())
    }

    fn read_mods(&mut self) -> Result<()> {
        self.expect_punct('[')?;
        self.expect_punct(']')?;
        self.expect_punct('=')?;
        self.expect_punct('{')?;

        loop {
            let t = self.expect_token()?;
            if t.is_punct('}') { break; }
            if ! t.is_punct('{') {
                error_expected("'{{' or '}}'", &t)?;
            }

            self.read_mod()?;
            self.expect_punct(',')?;
        }

        self.expect_punct(';')?;
        Ok(())
    }

    // =======================================================================================
    // === SFX
    // =======================================================================================

    fn read_sfx_sample_data(&mut self, name_id: &str) -> Result<()> {
        self.expect_punct('[')?;
        self.expect_punct(']')?;
        self.expect_punct('=')?;

        let data = self.read_i8or16_array(self.last_type_size)?;

        self.expect_punct(';')?;

        self.logger.log(format!("-> got sfx sample data '{}'", name_id));
        self.read_data.sfx_sample_data.insert(name_id.to_string(), data);

        Ok(())
    }

    fn read_sfxs(&mut self) -> Result<()> {
        self.expect_punct('[')?;
        self.expect_punct(']')?;
        self.expect_punct('=')?;
        self.expect_punct('{')?;

        loop {
            let t = self.expect_any_punct("'{' or '}'")?;
            if t.is_punct('}') { break; }
            if ! t.is_punct('{') { return error(format!("expected '{{' or '}}', got {}", t), t.pos)?; }

            let len = self.read_number()? as u32;
            self.expect_punct(',')?;
            let loop_start = self.read_number()? as u32;
            self.expect_punct(',')?;
            let loop_len = self.read_number()? as u32;
            self.expect_punct(',')?;
            let bits_per_sample = self.read_number()? as u16;
            self.expect_punct(',')?;
            self.expect_punct('{')?;
            self.expect_punct('.')?;
            self.expect_any_ident("'data', 'data8' or 'data16'")?;
            self.expect_punct('=')?;
            let data_ident = self.expect_any_ident("NULL or sample data")?;
            self.expect_punct('}')?;
            self.expect_punct('}')?;
            self.expect_punct(',')?;

            let full_name_id = match data_ident.get_ident() {
                Some(ident) => ident,
                None => { error(format!("invalid sfx data: '{}'", &data_ident), data_ident.pos)? }
            };
            let (name_id, sample_data) = if let Some(name_id) = self.get_global_lower_of_type(full_name_id, "sfx_samples") &&
                let Some(data) = self.read_data.sfx_sample_data.get(name_id) {
                    (name_id, data)
                } else {
                    return error(format!("unknown sfx samples '{}'", full_name_id), data_ident.pos)?;
                };
            let sample_data = if sample_data.data_size == bits_per_sample as u32 {
                &sample_data.data
            } else {
                error(format!("invalid sample: data has {} bits per sample, but sfx wants {}",
                                   sample_data.data_size, bits_per_sample), data_ident.pos)?
            };

            let data = super::sfx::CreationData {
                len,
                loop_start,
                loop_len,
                bits_per_sample,
                samples: sample_data,
            };
            if let Some(id) = self.store.add_sfx_from(DataAsset::identifier_to_name(name_id), data) {
                self.read_data.sfxs.push(id);
                self.read_data.sfxs_by_name_id.insert(name_id.to_string(), id);
                self.logger.log(format!("  -> added SFX '{}' id={}", name_id, id));
            } else {
                error(format!("error adding sfx '{}'", full_name_id), data_ident.pos)?
            }
        }

        self.expect_punct(';')?;
        Ok(())
    }

    // =======================================================================================
    // === TILESET
    // =======================================================================================

    fn read_tileset_data(&mut self, name_id: &str) -> Result<()> {
        self.expect_punct('[')?;
        self.expect_punct(']')?;
        self.expect_punct('=')?;

        let data = self.read_u32_array()?;

        self.expect_punct(';')?;

        self.logger.log(format!("-> got tileset data '{}'", name_id));
        self.read_data.tileset_data.insert(name_id.to_string(), data);

        Ok(())
    }

    fn read_tilesets(&mut self) -> Result<()> {
        self.expect_punct('[')?;
        self.expect_punct(']')?;
        self.expect_punct('=')?;
        self.expect_punct('{')?;

        loop {
            let t = self.expect_any_punct("'{' or '}'")?;
            if t.is_punct('}') { break; }
            if ! t.is_punct('{') { return error(format!("expected '{{' or '}}', got {}", t), t.pos)?; }

            let width = self.read_number()? as u32;
            self.expect_punct(',')?;
            let height = self.read_number()? as u32;
            self.expect_punct(',')?;
            let stride = self.read_number()? as u32;
            self.expect_punct(',')?;
            let num_tiles = self.read_number()? as u32;
            self.expect_punct(',')?;
            let ident = self.expect_any_ident("tileset data identifier")?;
            self.expect_punct('}')?;
            self.expect_punct(',')?;

            let full_name_id = match ident.get_ident() {
                Some(ident) => ident,
                None => { return error_expected("tileset_data_...", &t)?; },
            };
            let (name_id, data) = if let Some(name_id) = self.get_global_lower_of_type(full_name_id, "tileset_data") &&
                let Some(data) = self.read_data.tileset_data.get(name_id) {
                    (name_id, data)
                } else {
                    return error(format!("tileset data not found: '{}'", full_name_id), ident.pos)?;
                };

            if width != super::Tileset::TILE_SIZE || height != super::Tileset::TILE_SIZE {
                error(format!("invalid tileset size: got {}x{}, expected {}x{}",
                              width, height, super::Tileset::TILE_SIZE, super::Tileset::TILE_SIZE), t.pos)?;
            }
            let want_stride = width.div_ceil(4);
            if stride != want_stride {
                error(format!("tileset stride doesn't match width: got {}, expected {}", stride, want_stride), t.pos)?;
            }
            let want_len = stride * height * num_tiles;
            if data.len() as u32 != want_len {
                error(format!("unexpected tileset data length: got {}, expected {} = {}*{}*{}",
                              data.len(), want_len, stride, height, num_tiles), t.pos)?;
            }

            let data = super::tileset::CreationData {
                width,
                height,
                num_tiles,
                pixels: self.get_image_pixels(data, width, height, num_tiles),
            };
            if let Some(id) = self.store.add_tileset_from(DataAsset::identifier_to_name(name_id), data) {
                self.read_data.tilesets.push(id);
                self.read_data.tilesets_by_name_id.insert(name_id.to_string(), id);
                self.logger.log(format!("  -> added TILESET '{}' id={}", name_id, id));
            } else {
                return error(format!("error adding tileset '{}'", name_id), ident.pos)?;
            }
        }

        self.expect_punct(';')?;
        Ok(())
    }

    // =======================================================================================
    // === SPRITE
    // =======================================================================================

    fn read_sprite_data(&mut self, name_id: &str) -> Result<()> {
        self.expect_punct('[')?;
        self.expect_punct(']')?;
        self.expect_punct('=')?;

        let data = self.read_u32_array()?;

        self.expect_punct(';')?;

        self.logger.log(format!("-> got sprite data '{}'", name_id));
        self.read_data.sprite_data.insert(name_id.to_string(), data);

        Ok(())
    }

    fn read_sprites(&mut self) -> Result<()> {
        self.expect_punct('[')?;
        self.expect_punct(']')?;
        self.expect_punct('=')?;
        self.expect_punct('{')?;

        loop {
            let t = self.expect_any_punct("'{' or '}'")?;
            if t.is_punct('}') { break; }
            if ! t.is_punct('{') { return error(format!("expected '{{' or '}}', got {}", t), t.pos)?; }

            let width = self.read_number()? as u32;
            self.expect_punct(',')?;
            let height = self.read_number()? as u32;
            self.expect_punct(',')?;
            let stride = self.read_number()? as u32;
            self.expect_punct(',')?;
            let num_frames = self.read_number()? as u32;
            self.expect_punct(',')?;
            let ident = self.expect_any_ident("sprite data identifier")?;
            self.expect_punct('}')?;
            self.expect_punct(',')?;

            let full_name_id = match ident.get_ident() {
                Some(ident) => ident,
                None => { return error_expected("sprite_data_...", &t)?; },
            };
            let (name_id, data) = if let Some(name_id) = self.get_global_lower_of_type(full_name_id, "sprite_data") &&
                let Some(data) = self.read_data.sprite_data.get(name_id) {
                    (name_id, data)
                } else {
                    return error(format!("sprite data not found: '{}'", full_name_id), ident.pos)?;
                };
            if super::Sprite::MIRROR_FRAMES && ! num_frames.is_multiple_of(2) {
                error(format!("sprite with an odd number of tiles, should be even: {}", num_frames), t.pos)?;
            }
            let want_stride = width.div_ceil(4);  // (width+3)/4
            if stride != want_stride {
                error(format!("sprite stride doesn't match width: got {}, expected {}", stride, want_stride), t.pos)?;
            }
            let want_len = stride * height * num_frames;
            if data.len() as u32 != want_len {
                error(format!("unexpected sprite data length: got {}, expected {} = {}*{}*{}",
                              data.len(), want_len, stride, height, num_frames), t.pos)?;
            }

            let div_ignore_mirrors = if super::Sprite::MIRROR_FRAMES { 2 } else { 1 };
            let data = super::sprite::CreationData {
                width,
                height,
                num_frames: num_frames / div_ignore_mirrors,
                pixels: self.get_image_pixels(&data[0..data.len()/div_ignore_mirrors as usize], width, height, num_frames/div_ignore_mirrors),
            };
            if let Some(id) = self.store.add_sprite_from(DataAsset::identifier_to_name(name_id), data) {
                self.read_data.sprites.push(id);
                self.read_data.sprites_by_name_id.insert(name_id.to_string(), id);
                self.logger.log(format!("  -> added SPRITE '{}' id={}", name_id, id));
            } else {
                return error(format!("error adding sprite '{}'", name_id), ident.pos)?;
            }
        }

        self.expect_punct(';')?;
        Ok(())
    }

    // =======================================================================================
    // === PALETTED SPRITE
    // =======================================================================================

    fn read_pal_sprite_data(&mut self, name_id: &str) -> Result<()> {
        self.expect_punct('[')?;
        self.expect_punct(']')?;
        self.expect_punct('=')?;

        let data = self.read_u8_array()?;

        self.expect_punct(';')?;

        self.logger.log(format!("-> got pal_sprite data '{}'", name_id));
        self.read_data.pal_sprite_data.insert(name_id.to_string(), data);

        Ok(())
    }

    fn read_pal_sprites(&mut self) -> Result<()> {
        self.expect_punct('[')?;
        self.expect_punct(']')?;
        self.expect_punct('=')?;
        self.expect_punct('{')?;

        loop {
            let t = self.expect_any_punct("'{' or '}'")?;
            if t.is_punct('}') { break; }
            if ! t.is_punct('{') { return error(format!("expected '{{' or '}}', got {}", t), t.pos)?; }

            let width = self.read_number()? as u32;
            self.expect_punct(',')?;
            let height = self.read_number()? as u32;
            self.expect_punct(',')?;
            let num_frames = self.read_number()? as u32;
            self.expect_punct(',')?;
            let bits_per_pixel = self.read_number()? as u32;
            self.expect_punct(',')?;
            let palette = self.read_u8_array()?;
            self.expect_punct(',')?;
            let ident = self.expect_any_ident("pal_sprite data identifier")?;
            self.expect_punct('}')?;
            self.expect_punct(',')?;

            let full_name_id = match ident.get_ident() {
                Some(ident) => ident,
                None => { return error_expected("pal_sprite_data_...", &t)?; },
            };
            let (name_id, data) = if let Some(name_id) = self.get_global_lower_of_type(full_name_id, "pal_sprite_data") &&
                let Some(data) = self.read_data.pal_sprite_data.get(name_id) {
                    (name_id, data)
                } else {
                    return error(format!("pal_sprite data not found: '{}'", full_name_id), ident.pos)?;
                };
            let want_len = (width * bits_per_pixel).div_ceil(8) * height * num_frames;
            if data.len() as u32 != want_len {
                error(format!("unexpected pal_sprite data length: got {}, expected {} = {}*{}*{}",
                              data.len(), want_len, (width * bits_per_pixel).div_ceil(8), height, num_frames), t.pos)?;
            }
            if palette.len() != 16 {
                error(format!("invalid palette length: {} (must be 16)", palette.len()), t.pos)?;
            }
            let pixels = pal_image_to_pixels(data, &palette, width, height, num_frames, bits_per_pixel);
            let data = super::pal_sprite::CreationData {
                width,
                height,
                num_frames,
                bits_per_pixel,
                palette,
                pixels,
            };
            if let Some(id) = self.store.add_pal_sprite_from(DataAsset::identifier_to_name(name_id), data) {
                self.read_data.pal_sprites.push(id);
                self.read_data.pal_sprites_by_name_id.insert(name_id.to_string(), id);
                self.logger.log(format!("  -> added PAL_SPRITE '{}' id={}", name_id, id));
            } else {
                return error(format!("error adding pal_sprite '{}'", name_id), ident.pos)?;
            }
        }

        self.expect_punct(';')?;
        Ok(())
    }

    // =======================================================================================
    // === MAP
    // =======================================================================================

    fn read_map_tiles(&mut self, name_id: &str) -> Result<()> {
        self.expect_punct('[')?;
        self.expect_punct(']')?;
        self.expect_punct('=')?;

        let data = self.read_u8_array()?;

        self.expect_punct(';')?;

        self.logger.log(format!("-> got map tiles '{}'", name_id));
        self.read_data.map_tiles.insert(name_id.to_string(), data);
        Ok(())
    }

    fn read_maps(&mut self) -> Result<()> {
        self.expect_punct('[')?;
        self.expect_punct(']')?;
        self.expect_punct('=')?;
        self.expect_punct('{')?;

        loop {
            let t = self.expect_any_punct("'{' or '}'")?;
            if t.is_punct('}') { break; }
            if ! t.is_punct('{') { return error(format!("expected '{{' or '}}', got {}", t), t.pos)?; }

            let width = self.read_number()? as u32;
            self.expect_punct(',')?;
            let height = self.read_number()? as u32;
            self.expect_punct(',')?;
            let para_width = self.read_number()? as u32;
            self.expect_punct(',')?;
            let para_height = self.read_number()? as u32;
            self.expect_punct(',')?;
            let tileset_id = self.read_asset_index_reference("tilesets")?;
            self.expect_punct(',')?;
            let ident = self.expect_any_ident("map tiles identifier")?;
            self.expect_punct('}')?;
            self.expect_punct(',')?;

            let full_name_id = match ident.get_ident() {
                Some(ident) => ident,
                None => { return error_expected("map_tiles_...", &t)?; },
            };
            let (name_id, tiles_data) = if let Some(name_id) = self.get_global_lower_of_type(full_name_id, "map_tiles") &&
                let Some(data) = self.read_data.map_tiles.get(name_id) {
                    (name_id, data)
                } else {
                    return error(format!("sprite data not found: '{}'", full_name_id), ident.pos)?;
                };

            let want_tiles_data_len = width * height * 3 + para_width * para_height;
            if tiles_data.len() as u32 != want_tiles_data_len {
                error(format!("unexpected map tiles data length: got {}, expected {} = {}*{}*3 + {}*{}",
                              tiles_data.len(), want_tiles_data_len, width, height, para_width, para_height), t.pos)?;
            }

            let data = super::map_data::CreationData {
                tileset_id,
                width,
                height,
                para_width,
                para_height,
                tiles: tiles_data,
            };
            if let Some(id) = self.store.add_map_from(DataAsset::identifier_to_name(name_id), data) {
                self.read_data.maps.push(id);
                self.read_data.maps_by_name_id.insert(name_id.to_string(), id);
                self.logger.log(format!("  -> added MAP '{}' id={} with tileset_id={}", name_id, id, tileset_id));
            } else {
                return error(format!("error adding map '{}'", name_id), ident.pos)?;
            }
        }

        self.expect_punct(';')?;
        Ok(())
    }

    // =======================================================================================
    // === SPRITE ANIMATIONS
    // =======================================================================================

    fn read_sprite_animation_frames(&mut self, name_id: &str) -> Result<()> {
        self.expect_punct('[')?;
        self.expect_punct(']')?;
        self.expect_punct('=')?;

        let data = self.read_u8_array()?;

        self.expect_punct(';')?;

        self.logger.log(format!("-> got sprite animations frames '{}'", name_id));
        self.read_data.animation_frames.insert(name_id.to_string(), data);
        Ok(())
    }

    fn read_sprite_animation_loops(&mut self) -> Result<Vec<super::sprite_animation::LoopCreationData>> {
        self.expect_punct('{')?;

        let mut loops = Vec::new();

        loop {
            let t = self.expect_token()?;
            if t.is_punct('}') { break; }
            if ! t.is_punct('{') {
                error_expected("'{{' or '}}'", &t)?;
            }

            let offset = self.read_number()?;
            self.expect_punct(',')?;
            let len = self.read_number()?;

            loops.push(super::sprite_animation::LoopCreationData {
                offset: offset as u16,
                len: len as u16,
            });
            self.expect_punct('}')?;
            self.expect_punct(',')?;
        }

        Ok(loops)
    }

    fn read_sprite_animations(&mut self) -> Result<()> {
        self.expect_punct('[')?;
        self.expect_punct(']')?;
        self.expect_punct('=')?;
        self.expect_punct('{')?;

        loop {
            let t = self.expect_any_punct("'{' or '}'")?;
            if t.is_punct('}') { break; }
            if ! t.is_punct('{') { return error(format!("expected '{{' or '}}', got {}", t), t.pos)?; }

            let frames_ident = self.expect_any_ident("animation frames identifier")?;
            self.expect_punct(',')?;
            let sprite_id = self.read_asset_index_reference("sprites")?;
            self.expect_punct(',')?;
            let clip_rect = self.read_u16_array()?;
            self.expect_punct(',')?;
            let use_foot_frames = self.read_number()?;
            self.expect_punct(',')?;
            let foot_overlap = self.read_signed_number()?;
            self.expect_punct(',')?;
            let loops = self.read_sprite_animation_loops()?;
            self.expect_punct('}')?;
            self.expect_punct(',')?;

            let (clip_x, clip_y, clip_w, clip_h) = if let Some(&[cx, cy, cw, ch]) = clip_rect.get(0..4) {
                (cx as i32, cy as i32, cw as i32, ch as i32)
            } else {
                error(format!("animation clip rectangle must have 4 numbers, found {}", clip_rect.len()), t.pos)?
            };

            let full_name_id = match frames_ident.get_ident() {
                Some(ident) => ident,
                None => { return error_expected("sprite_animation_frames_...", &t)?; },
            };
            let (name_id, frames_data) = if let Some(name_id) = self.get_global_lower_of_type(full_name_id, "sprite_animation_frames") &&
                let Some(data) = self.read_data.animation_frames.get(name_id) {
                    (name_id, data)
                } else {
                    return error(format!("sprite animation frames data not found: '{}'", full_name_id), t.pos)?;
                };

            let data = super::sprite_animation::CreationData {
                sprite_id,
                clip_rect: super::Rect::new(clip_x, clip_y, clip_w, clip_h),
                use_foot_frames: use_foot_frames != 0,
                foot_overlap: foot_overlap as i8,
                loops: &loops,
                frame_indices: frames_data,
            };
            if let Some(id) = self.store.add_animation_from(DataAsset::identifier_to_name(name_id), data) {
                self.read_data.animations.push(id);
                self.read_data.animations_by_name_id.insert(name_id.to_string(), id);
                self.logger.log(format!("  -> added SPRITE_ANIMATION '{}' id={} with sprite_id={}", name_id, id, sprite_id));
            } else {
                return error(format!("error adding sprite animation '{}' with sprite id '{}'", name_id, sprite_id), t.pos)?;
            }
        }

        self.expect_punct(';')?;
        Ok(())
    }

    // =======================================================================================
    // === ROOMS
    // =======================================================================================

    fn read_room_maps(&mut self, name_id: &str) -> Result<()> {
        self.expect_punct('[')?;
        self.expect_punct(']')?;
        self.expect_punct('=')?;
        self.expect_punct('{')?;

        let mut maps = Vec::<RoomMap>::new();
        loop {
            let t = self.expect_token()?;
            if t.is_punct('}') { break; }
            if ! t.is_punct('{') {
                error_expected("'{{' or '}}'", &t)?;
            }

            let x = self.read_number()? as u16;
            self.expect_punct(',')?;
            let y = self.read_number()? as u16;
            self.expect_punct(',')?;
            let map_id = self.read_asset_index_reference("maps")?;
            self.expect_punct('}')?;
            self.expect_punct(',')?;

            maps.push(RoomMap {
                x,
                y,
                map_id,
            });

        }
        self.expect_punct(';')?;

        self.logger.log(format!("-> got room maps '{}'", name_id));
        self.read_data.room_maps.insert(name_id.to_string(), maps);
        Ok(())
    }

    fn read_room_entities(&mut self, name_id: &str) -> Result<()> {
        self.expect_punct('[')?;
        self.expect_punct(']')?;
        self.expect_punct('=')?;
        self.expect_punct('{')?;

        let mut entities = Vec::<RoomEntity>::new();
        loop {
            let t = self.expect_token()?;
            if t.is_punct('}') { break; }
            if ! t.is_punct('{') {
                error_expected("'{{' or '}}'", &t)?;
            }

            let x = self.read_number()? as i16;
            self.expect_punct(',')?;
            let y = self.read_number()? as i16;
            self.expect_punct(',')?;
            let animation_id = self.read_asset_index_reference("sprite_animations")?;
            self.expect_punct(',')?;
            let data0 = self.read_number()? as u16;
            self.expect_punct(',')?;
            let data1 = self.read_number()? as u16;
            self.expect_punct(',')?;
            let data2 = self.read_number()? as u16;
            self.expect_punct(',')?;
            let data3 = self.read_number()? as u16;
            self.expect_punct('}')?;
            self.expect_punct(',')?;

            entities.push(RoomEntity {
                name_id: String::new(),
                x,
                y,
                animation_id,
                data0,
                data1,
                data2,
                data3,
            });
        }
        self.expect_punct(';')?;

        self.logger.log(format!("-> got room entities '{}'", name_id));
        self.read_data.room_entities.insert(name_id.to_string(), entities);
        Ok(())
    }

    fn read_room_triggers(&mut self, name_id: &str) -> Result<()> {
        self.expect_punct('[')?;
        self.expect_punct(']')?;
        self.expect_punct('=')?;
        self.expect_punct('{')?;

        let mut triggers = Vec::<RoomTrigger>::new();
        loop {
            let t = self.expect_token()?;
            if t.is_punct('}') { break; }
            if ! t.is_punct('{') {
                error_expected("'{{' or '}}'", &t)?;
            }

            let x = self.read_number()? as i16;
            self.expect_punct(',')?;
            let y = self.read_number()? as i16;
            self.expect_punct(',')?;
            let width = self.read_number()? as i16;
            self.expect_punct(',')?;
            let height = self.read_number()? as i16;
            self.expect_punct(',')?;
            let data0 = self.read_number()? as u16;
            self.expect_punct(',')?;
            let data1 = self.read_number()? as u16;
            self.expect_punct(',')?;
            let data2 = self.read_number()? as u16;
            self.expect_punct(',')?;
            let data3 = self.read_number()? as u16;
            self.expect_punct('}')?;
            self.expect_punct(',')?;

            triggers.push(RoomTrigger {
                name_id: String::new(),
                x,
                y,
                width,
                height,
                data0,
                data1,
                data2,
                data3,
            });
        }
        self.expect_punct(';')?;

        self.logger.log(format!("-> got room triggers '{}'", name_id));
        self.read_data.room_triggers.insert(name_id.to_string(), triggers);
        Ok(())
    }

    fn read_rooms(&mut self) -> Result<()> {
        self.expect_punct('[')?;
        self.expect_punct(']')?;
        self.expect_punct('=')?;
        self.expect_punct('{')?;

        loop {
            let t = self.expect_any_punct("'{' or '}'")?;
            if t.is_punct('}') { break; }
            if ! t.is_punct('{') { return error(format!("expected '{{' or '}}', got {}", t), t.pos)?; }

            let num_maps = self.read_number()? as usize;
            self.expect_punct(',')?;
            let num_entities = self.read_number()? as usize;
            self.expect_punct(',')?;
            let num_triggers = self.read_number()? as usize;
            self.expect_punct(',')?;
            let maps_ident = self.expect_any_ident("room maps identifier")?;
            self.expect_punct(',')?;
            let entities_ident = self.expect_any_ident("room entities identifier")?;
            self.expect_punct(',')?;
            let triggers_ident = self.expect_any_ident("room triggers identifier")?;
            self.expect_punct('}')?;
            self.expect_punct(',')?;

            let full_name_id = match maps_ident.get_ident() {
                Some(ident) => ident,
                None => { return error_expected("room_maps_...", &t)?; },
            };
            let (name_id, maps_data) = if let Some(name_id) = self.get_global_lower_of_type(full_name_id, "room_maps") &&
                let Some(data) = self.read_data.room_maps.get(name_id) {
                    (name_id, data)
                } else {
                    return error(format!("room maps data not found: '{}'", full_name_id), maps_ident.pos)?;
                };
            let entities_name_id = match entities_ident.get_ident() {
                Some(ident) => ident,
                None => { return error_expected("room_maps_...", &t)?; },
            };
            let entities_data = if let Some(entities_name_id) = self.get_global_lower_of_type(entities_name_id, "room_entities") &&
                let Some(data) = self.read_data.room_entities.get(entities_name_id) {
                    data
                } else {
                    return error(format!("room entities data not found: '{}'", entities_name_id), entities_ident.pos)?;
                };
            let triggers_name_id = match triggers_ident.get_ident() {
                Some(ident) => ident,
                None => { return error_expected("room_maps_...", &t)?; },
            };
            let triggers_data = if let Some(triggers_name) = self.get_global_lower_of_type(triggers_name_id, "room_triggers") &&
                let Some(data) = self.read_data.room_triggers.get(triggers_name) {
                    data
                } else {
                    return error(format!("room triggers data not found: '{}'", triggers_name_id), triggers_ident.pos)?;
                };
            if num_maps != maps_data.len() {
                error(format!("unexpected maps length: got {}, expected {}", maps_data.len(), num_maps), t.pos)?;
            }
            if num_entities != entities_data.len() {
                error(format!("unexpected entities length: got {}, expected {}", entities_data.len(), num_entities), t.pos)?;
            }
            if num_triggers != triggers_data.len() {
                error(format!("unexpected triggers length: got {}, expected {}", triggers_data.len(), num_triggers), t.pos)?;
            }

            let data = super::room::CreationData {
                maps: maps_data,
                entities: entities_data,
                triggers: triggers_data,
            };
            if let Some(id) = self.store.add_room_from(DataAsset::identifier_to_name(name_id), data) {
                self.read_data.rooms.push(id);
                self.read_data.rooms_by_name_id.insert(name_id.to_string(), id);
                self.logger.log(format!("  -> added ROOM '{}' id={}", name_id, id));
            } else {
                return error(format!("error adding room '{}'", name_id), maps_ident.pos)?;
            }
        }

        self.expect_punct(';')?;
        Ok(())
    }

    // =======================================================================================
    // === NAMES
    // =======================================================================================

    // read enum list of "<NAME>" for items "<PREFIX>_<TYPE_AND_NAME>_<NAME>"
    fn read_asset_names_enum(&mut self, type_and_name_id: &str) -> Result<Vec<String>> {
        self.expect_punct('{')?;

        let mut names = Vec::<String>::new();
        loop {
            let t = self.expect_token()?;
            if t.is_punct('}') { break; }

            let name = if let Some(ident) = t.get_ident() &&
                let Some(name_id) = self.get_global_upper_of_type(ident, type_and_name_id) {
                    let mut name_id = name_id.to_string();
                    name_id.make_ascii_lowercase();
                    name_id
                } else {
                    return error(format!("expected '<PREFIX>_{}_xxx' or '}}', got {}", type_and_name_id, t), t.pos)?;
                };
            names.push(name);

            let next = self.expect_any_punct("',' or '}'")?;
            if next.is_punct('}') { break; }
            if ! next.is_punct(',') {
                error_expected("',' or '}'", &next)?;
            }
        }
        self.expect_punct(';')?;
        Ok(names)
    }

    fn get_enum_asset_and_item_prefix(&self, ident: &str, asset_type: &str, name_suffix: &str, name_type: &str,
                                      ids_by_name_id: &HashMap<String,DataAssetId>, pos: TokenPosition) -> Result<(DataAssetId, String)> {
        let asset_name_upper = match self.get_global_upper_of_type_with_suffix(ident, asset_type, name_suffix) {
            Some(name) => name,
            None => { return error(format!("unknown enum for {}: '{}'", asset_type, ident), pos); }
        };
        let mut asset_name = asset_name_upper.to_string();
        asset_name.make_ascii_lowercase();
        let asset_id = match ids_by_name_id.get(&asset_name) {
            Some(id) => id,
            None => { return error(format!("{} not found: '{}'", asset_type, &asset_name), pos); }
        };
        let mut item_prefix = String::new();
        item_prefix.push_str(asset_type);
        item_prefix.push('_');
        item_prefix.push_str(asset_name_upper);
        item_prefix.push('_');
        item_prefix.push_str(name_type);
        Ok((*asset_id, item_prefix))
    }

    fn read_sprite_animation_loop_names(&mut self, ident: &str, pos: TokenPosition) -> Result<()> {
        let (id, item_prefix) = self.get_enum_asset_and_item_prefix(ident, "SPRITE_ANIMATION", "LOOP_NAMES", "LOOP",
                                                                    &self.read_data.animations_by_name_id, pos)?;
        let names = self.read_asset_names_enum(&item_prefix)?;
        let animation = match self.store.assets.animations.get_mut(&id) {
            Some(asset) => asset,
            None => { return error(format!("internal error: animation id {} not found", id), pos); }
        };
        self.logger.log(format!("-> reading SPRITE_ANIMATION LOOP names for '{}':", animation.asset.name));
        for (index, name_id) in names.iter().enumerate() {
            if let Some(anim_loop) = animation.loops.get_mut(index) {
                self.logger.log(format!("  -> {}", name_id));
                anim_loop.name_id.push_str(name_id);
            } else {
                return error(format!("animation '{}' doesn't have loop {}", animation.asset.name, index), pos);
            }
        }
        for (index, aloop) in animation.loops.iter_mut().enumerate() {
            if aloop.name_id.is_empty() {
                aloop.name_id = format!("loop {}", index);
            }
        }
        Ok(())
    }

    fn read_sprite_animation_names(&mut self, ident: &str, pos: TokenPosition) -> Result<()> {
        if ident.ends_with("LOOP_NAMES") {
            self.read_sprite_animation_loop_names(ident, pos)?;
            return Ok(());
        }
        error(format!("unknown sprite animation enum: '{}'", ident), pos)
    }

    fn read_room_entity_names(&mut self, ident: &str, pos: TokenPosition) -> Result<()> {
        let (id, item_prefix) = self.get_enum_asset_and_item_prefix(ident, "ROOM", "ENT_NAMES", "ENT",
                                                                    &self.read_data.rooms_by_name_id, pos)?;
        let name_ids = self.read_asset_names_enum(&item_prefix)?;
        let room = match self.store.assets.rooms.get_mut(&id) {
            Some(asset) => asset,
            None => { return error(format!("internal error: room id {} not found", id), pos); }
        };
        self.logger.log(format!("-> reading ROOM ENTITY names for '{}':", room.asset.name));
        for (index, name_id) in name_ids.iter().enumerate() {
            if let Some(ent) = room.entities.get_mut(index) {
                self.logger.log(format!("  -> {}", name_id));
                ent.name_id.push_str(name_id);
            } else {
                return error(format!("room '{}' doesn't have entity {}", room.asset.name, index), pos);
            }
        }
        Ok(())
    }

    fn read_room_trigger_names(&mut self, ident: &str, pos: TokenPosition) -> Result<()> {
        let (id, item_prefix) = self.get_enum_asset_and_item_prefix(ident, "ROOM", "TRG_NAMES", "TRG",
                                                                    &self.read_data.rooms_by_name_id, pos)?;
        let name_ids = self.read_asset_names_enum(&item_prefix)?;
        let room = match self.store.assets.rooms.get_mut(&id) {
            Some(asset) => asset,
            None => { return error(format!("internal error: room id {} not found", id), pos); }
        };
        self.logger.log(format!("-> reading ROOM TRIGGER names for '{}':", room.asset.name));
        for (index, name_id) in name_ids.iter().enumerate() {
            if let Some(trg) = room.triggers.get_mut(index) {
                self.logger.log(format!("  -> {}", name_id));
                trg.name_id.push_str(name_id);
            } else {
                return error(format!("room '{}' doesn't have trigger {}", room.asset.name, index), pos);
            }
        }
        Ok(())
    }

    fn read_room_names(&mut self, ident: &str, pos: TokenPosition) -> Result<()> {
        if ident.ends_with("_ENT_NAMES") {
            self.read_room_entity_names(ident, pos)?;
            return Ok(());
        }
        if ident.ends_with("_TRG_NAMES") {
            self.read_room_trigger_names(ident, pos)?;
            return Ok(());
        }
        error(format!("unknown room enum: '{}'", ident), pos)
    }

    // =======================================================================================
    // === IDS
    // =======================================================================================

    fn read_asset_ids(&mut self, ident: &str, pos: TokenPosition) -> Result<()> {
        let name = match self.get_global_upper_with_suffix(ident, "IDS") {
            Some(name) => name,
            None => { return error(format!("invalid IDS enum: {}", ident), pos); },
        };
        let mut name_with_id = name.to_string();
        name_with_id.push_str("_ID");

        self.expect_punct('{')?;

        self.logger.log(format!("-> reading {} asset ids", name));
        let mut got_count = false;
        loop {
            let t = self.expect_token()?;
            if t.is_punct('}') { break; }

            match t.get_ident() {
                Some(ident) => {
                    if let Some(name) = self.get_global_upper_of_type(ident, name) && name == "COUNT" {
                        got_count = true;
                    } else if let Some(item_name) = self.get_global_upper_of_type(ident, &name_with_id) {
                        self.logger.log(format!("  -> got asset id '{}'", item_name));
                    } else {
                        return error(format!("invalid asset ID: {}", ident), pos);
                    }
                },
                None => { return error_expected("identifier", &t); }
            };

            let next = self.expect_any_punct("',' or '}'")?;
            if next.is_punct('}') { break; }
            if ! next.is_punct(',') {
                error_expected("',' or '}'", &next)?;
            }
        }
        self.expect_punct(';')?;

        if ! got_count {
            self.logger.log(format!("-> WARNING: asset ids for {} doesn't end with COUNT", ident));
        }

        Ok(())
    }

    // =======================================================================================
    // =======================================================================================
    // =======================================================================================

    fn is_struct_name(&self, ident: &str) -> bool {
        if let Some(name) = self.get_global_upper(ident) {
            return C_STRUCT_NAMES.contains(&name);
        }
        false
    }

    pub fn read_project(&mut self) -> Result<()> {
        self.read_project_defines()?;

        loop {
            let t = self.read()?;
            if t.is_eof() { break; }

            if let Some(line) = t.get_pre_processor() {
                self.handle_pre_processor_line(line);
                continue;
            }

            // font stuff
            if let Some(ident) = t.get_ident() && let Some(name) = self.get_global_lower_of_type(ident, "font_data") {
                self.read_font_data(name)?;
                continue;
            }
            if let Some(ident) = t.get_ident() && self.is_global_lower(ident, "fonts") {
                self.read_fonts()?;
                continue;
            }

            // proportional font stuff
            if let Some(ident) = t.get_ident() && let Some(name_id) = self.get_global_lower_of_type(ident, "prop_font_data") {
                self.read_prop_font_data(name_id)?;
                continue;
            }
            if let Some(ident) = t.get_ident() && self.is_global_lower(ident, "prop_fonts") {
                self.read_prop_fonts()?;
                continue;
            }

            // MOD stuff
            if let Some(ident) = t.get_ident() && let Some(name_id) = self.get_global_lower_of_type(ident, "mod_samples") {
                self.read_mod_sample_data(name_id)?;
                continue;
            }
            if let Some(ident) = t.get_ident() && let Some(name) = self.get_global_lower_of_type(ident, "mod_pattern") {
                self.read_mod_pattern(name)?;
                continue;
            }
            if let Some(ident) = t.get_ident() && self.is_global_lower(ident, "mods") {
                self.read_mods()?;
                continue;
            }

            // sfx stuff
            if let Some(ident) = t.get_ident() && let Some(name) = self.get_global_lower_of_type(ident, "sfx_samples") {
                self.read_sfx_sample_data(name)?;
                continue;
            }
            if let Some(ident) = t.get_ident() && self.is_global_lower(ident, "sfxs") {
                self.read_sfxs()?;
                continue;
            }

            // tileset stuff
            if let Some(ident) = t.get_ident() && let Some(name) = self.get_global_lower_of_type(ident, "tileset_data") {
                self.read_tileset_data(name)?;
                continue;
            }
            if let Some(ident) = t.get_ident() && self.is_global_lower(ident, "tilesets") {
                self.read_tilesets()?;
                continue;
            }

            // sprite stuff
            if let Some(ident) = t.get_ident() && let Some(name) = self.get_global_lower_of_type(ident, "sprite_data") {
                self.read_sprite_data(name)?;
                continue;
            }
            if let Some(ident) = t.get_ident() && self.is_global_lower(ident, "sprites") {
                self.read_sprites()?;
                continue;
            }

            // paletted sprite stuff
            if let Some(ident) = t.get_ident() && let Some(name) = self.get_global_lower_of_type(ident, "pal_sprite_data") {
                self.read_pal_sprite_data(name)?;
                continue;
            }
            if let Some(ident) = t.get_ident() && self.is_global_lower(ident, "pal_sprites") {
                self.read_pal_sprites()?;
                continue;
            }

            // map stuff
            if let Some(ident) = t.get_ident() && let Some(name) = self.get_global_lower_of_type(ident, "map_tiles") {
                self.read_map_tiles(name)?;
                continue;
            }
            if let Some(ident) = t.get_ident() && self.is_global_lower(ident, "maps") {
                self.read_maps()?;
                continue;
            }

            // sprite animation stuff
            if let Some(ident) = t.get_ident() && let Some(name) = self.get_global_lower_of_type(ident, "sprite_animation_frames") {
                self.read_sprite_animation_frames(name)?;
                continue;
            }
            if let Some(ident) = t.get_ident() && self.is_global_lower(ident, "sprite_animations") {
                self.read_sprite_animations()?;
                continue;
            }

            // room stuff
            if let Some(ident) = t.get_ident() && let Some(name) = self.get_global_lower_of_type(ident, "room_maps") {
                self.read_room_maps(name)?;
                continue;
            }
            if let Some(ident) = t.get_ident() && let Some(name) = self.get_global_lower_of_type(ident, "room_entities") {
                self.read_room_entities(name)?;
                continue;
            }
            if let Some(ident) = t.get_ident() && let Some(name) = self.get_global_lower_of_type(ident, "room_triggers") {
                self.read_room_triggers(name)?;
                continue;
            }
            if let Some(ident) = t.get_ident() && self.is_global_lower(ident, "rooms") {
                self.read_rooms()?;
                continue;
            }

            if let Some(ident) = t.get_ident() {
                // C keywords, C types and struct names
                if C_KEYWORDS.contains(&ident) { continue; }
                if self.is_struct_name(ident) { continue; }
                if ident == "uint8_t" { self.last_type_size = 8; continue; }
                if ident == "int8_t" { self.last_type_size = 8; continue; }
                if ident == "uint16_t" { self.last_type_size = 16; continue; }
                if ident == "int16_t" { self.last_type_size = 16; continue; }
                if ident == "uint32_t" { self.last_type_size = 32; continue; }
                if ident == "int32_t" { self.last_type_size = 32; continue; }

                // asset ids
                if ident.ends_with("IDS") {
                    self.read_asset_ids(ident, t.pos)?;
                    continue;
                }

                // asset names
                if self.is_global_upper_of_type(ident, "SPRITE_ANIMATION") {
                    self.read_sprite_animation_names(ident, t.pos)?;
                    continue;
                }
                if self.is_global_upper_of_type(ident, "ROOM") {
                    self.read_room_names(ident, t.pos)?;
                    continue;
                }
            }

            error(format!("unexpected '{}'", t), t.pos)?;
        }

        // name unnamed animation loops
        for anim in self.store.assets.animations.iter_mut() {
            for (index, aloop) in anim.loops.iter_mut().enumerate() {
                if aloop.name_id.is_empty() {
                    aloop.name_id = format!("loop {}", index);
                }
            }
        }

        Ok(())
    }

}

pub fn read_project<P: AsRef<Path>>(filename: P, store: &mut DataAssetStore, logger: &mut StringLogger) -> Result<()> {
    logger.log(format!("-> reading file {}", filename.as_ref().display()));
    store.vga_bits_per_pixel = 6;  // default for compatibility

    let data = match fs::read_to_string(filename) {
        Ok(data) => data,
        Err(e) => {
            logger.log(format!("ERROR: {}", e));
            return Err(e);
        }
    };

    let mut reader = ProjectDataReader::new(&data, store, logger);
    match reader.read_project() {
        Ok(()) => {
            logger.log("DONE: project read");
            Ok(())
        },
        Err(e) => {
            logger.log(format!("ERROR: {}", e));
            Err(e)
        }
    }
}
