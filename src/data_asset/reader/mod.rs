pub mod tokenizer;
mod image_converter;
mod pre_processor;
mod tileset;
mod map_data;
mod room;
mod world;
mod sprite;
mod pal_sprite;
mod sprite_animation;
mod sfx;
mod mod_data;
mod font;
mod prop_font;

use std::io::{Result, Error};
use std::collections::HashMap;
use std::sync::LazyLock;

use image_converter::ImageConverter;

pub use tokenizer::{
    Tokenizer,
    Token,
    TokenData,
    TokenPosition,
};

use super::{
    StringLogger,
    DataAssetId,
    DataAssetStore,
    DataAssetIdGenerator,
    AssetCollection,
    AssetIdCollection,
};

pub fn err<S: AsRef<str>>(msg: S, pos: TokenPosition) -> Error {
    Error::other(format!("line {}: {}", pos.line, msg.as_ref()))
}

pub fn error<T, S: AsRef<str>>(msg: S, pos: TokenPosition) -> Result<T> {
    Result::Err(err(msg, pos))
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum ValueType {
    U8,
    U16,
    U32,
    I8,
    I16,
    I32,
    U8Array,
    U16Array,
    U32Array,
    I8Array,
    I16Array,
    ArrayRef,
    AssetRef,
    Identifier,
    Loop(Box<ValueType>),
    Struct(Vec<ValueType>),
    Custom(fn (&mut ProjectDataReader) -> Result<Value>),
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum Value {
    U8(u8),
    U16(u16),
    U32(u32),
    I8(i8),
    I16(i16),
    I32(i32),
    U8Array(Box<ValueArray<u8>>),
    I8Array(Box<ValueArray<i8>>),
    U16Array(Box<ValueArray<u16>>),
    I16Array(Box<ValueArray<i16>>),
    U32Array(Box<ValueArray<u32>>),
    Identifier(Box<ValueName>),
    ArrayRef(Box<ValueName>),
    AssetRef(Box<ValueAssetRef>),
    Loop(Vec<Value>),
    Struct(Vec<Value>),
}

#[derive(Debug)]
pub struct ValueArray<T> {
    vec: Vec<T>,
    pos: TokenPosition,
}

impl<T> ValueArray<T> {
    pub fn new(vec: Vec<T>, pos: TokenPosition) -> Self {
        ValueArray {
            vec,
            pos,
        }
    }
}

enum ValueArrayDataI8orI16<'a> {
    I8Converted(Vec<i16>),
    I16Original(&'a Vec<i16>)
}

impl<'a> ValueArrayDataI8orI16<'a> {
    pub fn take(self) -> Vec<i16> {
        match self {
            ValueArrayDataI8orI16::I8Converted(a) => { a }
            ValueArrayDataI8orI16::I16Original(a) => { a.clone() }
        }
    }
}

#[derive(Debug)]
pub struct ValueName {
    name: String,
    pos: TokenPosition,
}

#[allow(dead_code)]
impl ValueName {
    pub fn new(name: String, pos: TokenPosition) -> Self {
        ValueName {
            name,
            pos,
        }
    }

    pub fn is_null(&self) -> bool {
        self.name == "NULL"
    }

    fn array_exists(&self, data: &ProjectData) -> bool {
        data.arrays.contains_key(&self.name) ||
            data.structs.contains_key(&self.name) ||
            data.assets.contains_key(&self.name)
    }

    fn error_not_found<T>(&self, data: &ProjectData) -> Result<T> {
        if self.array_exists(data) {
            error(format!("array has unexpected data type: '{}'", self.name), self.pos)
        } else {
            error(format!("array not found: '{}'", self.name), self.pos)
        }
    }

    pub fn get_struct_array<'a>(&self, data: &'a ProjectData) -> Result<&'a Vec<Value>> {
        match data.structs.get(&self.name) {
            Some(v)  => { Ok(v) }
            _ => { self.error_not_found(data) }
        }
    }

    pub fn get_u8_array<'a>(&self, data: &'a ProjectData) -> Result<&'a Vec<u8>> {
        match data.arrays.get(&self.name) {
            Some(Value::U8Array(a))  => { Ok(&a.vec) }
            _ => { self.error_not_found(data) }
        }
    }

    pub fn get_u16_array<'a>(&self, data: &'a ProjectData) -> Result<&'a Vec<u16>> {
        match data.arrays.get(&self.name) {
            Some(Value::U16Array(a))  => { Ok(&a.vec) }
            _ => { self.error_not_found(data) }
        }
    }

    pub fn get_u32_array<'a>(&self, data: &'a ProjectData) -> Result<&'a Vec<u32>> {
        match data.arrays.get(&self.name) {
            Some(Value::U32Array(a))  => { Ok(&a.vec) }
            _ => { self.error_not_found(data) }
        }
    }

    pub fn get_i8_array<'a>(&self, data: &'a ProjectData) -> Result<&'a Vec<i8>> {
        match data.arrays.get(&self.name) {
            Some(Value::I8Array(a))  => { Ok(&a.vec) }
            _ => { self.error_not_found(data) }
        }
    }

    pub fn get_i16_array<'a>(&self, data: &'a ProjectData) -> Result<&'a Vec<i16>> {
        match data.arrays.get(&self.name) {
            Some(Value::I16Array(a))  => { Ok(&a.vec) }
            _ => { self.error_not_found(data) }
        }
    }

    fn get_i8_or_i16_array<'a>(&self, data: &'a ProjectData) -> Result<ValueArrayDataI8orI16<'a>> {
        match data.arrays.get(&self.name) {
            Some(Value::I8Array(v)) => {
                Ok(ValueArrayDataI8orI16::I8Converted(v.vec.iter().map(|s| { (*s as i16) << 8 }).collect()))
            }
            Some(Value::I16Array(v)) => {
                Ok(ValueArrayDataI8orI16::I16Original(&v.vec))
            }
            _ => {
                self.error_not_found(data)
            }
        }
    }
}

#[derive(Debug)]
pub struct ValueAssetRef {
    name: String,
    index: usize,
    pos: TokenPosition,
}

impl ValueAssetRef {
    pub fn new(name: String, index: usize, pos: TokenPosition) -> Self {
        ValueAssetRef {
            name,
            index,
            pos,
        }
    }

    pub fn get_asset_id(&self, data: &ProjectData) -> Result<DataAssetId> {
        match data.asset_ids_by_prefixed_name.get(&self.name).and_then(|v| v.get(self.index)) {
            Some(id) => { Ok(*id) }
            None => { error(format!("invalid asset reference: '{}[{}]'", self.name, self.index), self.pos) }
        }
    }
}

// simple numeric arrays
static ARRAY_VALUE_TYPES: LazyLock<HashMap<String,ValueType>> = LazyLock::new(|| {
    HashMap::from([
        (String::from("uint8_t"),  ValueType::U8Array),
        (String::from("uint16_t"), ValueType::U16Array),
        (String::from("uint32_t"), ValueType::U32Array),
        (String::from("int8_t"),   ValueType::I8Array),
        (String::from("int16_t"),  ValueType::I16Array),
    ])
});

// structs used inside asset structs
static AUX_STRUCT_VALUE_TYPES: LazyLock<HashMap<String,ValueType>> = LazyLock::new(|| {
    HashMap::from([
        (String::from("MOD_CELL"), ValueType::Struct(vec![
            ValueType::U8,          // sample
            ValueType::U8,          // note index
            ValueType::U16,         // effect
        ])),

        (String::from("ROOM_MAP_INFO"), ValueType::Struct(vec![
            ValueType::U16,         // x
            ValueType::U16,         // y
            ValueType::AssetRef,    // room
        ])),

        (String::from("ROOM_TRIGGER_INFO"), ValueType::Struct(vec![
            ValueType::Identifier,  // type
            ValueType::I16,         // x
            ValueType::I16,         // y
            ValueType::Custom(custom_read_room_trigger_info),
        ])),

        (String::from("WORLD_REGION"), ValueType::Struct(vec![
            ValueType::U8,          // x
            ValueType::U8,          // y
            ValueType::U8,          // width
            ValueType::U8,          // height
            ValueType::ArrayRef,    // block_bitmap
            ValueType::ArrayRef,    // blocks
            ValueType::ArrayRef,    // room_indices
        ])),
    ])
});

// asset structs
static ASSET_STRUCT_VALUE_TYPES: LazyLock<HashMap<String,ValueType>> = LazyLock::new(|| {
    HashMap::from([
        (String::from("FONT"), ValueType::Struct(vec![
            ValueType::U8,          // width
            ValueType::U8,          // height
            ValueType::ArrayRef,    // data
        ])),

        (String::from("PROP_FONT"), ValueType::Struct(vec![
            ValueType::U8,          // height
            ValueType::ArrayRef,    // data
            ValueType::U8Array,     // char widths
            ValueType::U16Array,    // char offsets
        ])),

        (String::from("PAL_SPRITE"), ValueType::Struct(vec![
            ValueType::I16,         // width
            ValueType::I16,         // height
            ValueType::I16,         // num frames
            ValueType::U16,         // bits per pixel
            ValueType::U8Array,     // palette
            ValueType::ArrayRef,    // data
        ])),

        (String::from("MOD_DATA"), ValueType::Struct(vec![
            ValueType::Loop(Box::new(  // 31 samples
                ValueType::Struct(vec![
                    ValueType::U32,    // len
                    ValueType::U32,    // loop start
                    ValueType::U32,    // loop len
                    ValueType::U8,     // finetune
                    ValueType::U8,     // volume
                    ValueType::U16,    // bits per sample
                    ValueType::Custom(custom_read_mod_sample_data),  // data: ArrayRef(i8/i16)
                ])
            )),
            ValueType::U8,         // num channels
            ValueType::U8,         // num song positions
            ValueType::U8Array,    // song positions
            ValueType::U8,         // num patterns
            ValueType::ArrayRef,   // patterns
        ])),

        (String::from("SFX"), ValueType::Struct(vec![
            ValueType::U32,       // len
            ValueType::U32,       // loop start
            ValueType::U32,       // loop end
            ValueType::U16,       // bits per sample
            ValueType::Custom(custom_read_sfx_sample_data),  // data: ArrayRef(i8/i16)
        ])),

        (String::from("IMAGE"), ValueType::Struct(vec![
            ValueType::U32,       // width
            ValueType::U32,       // height
            ValueType::U32,       // stride
            ValueType::U32,       // num frames
            ValueType::ArrayRef,  // data
        ])),

        (String::from("MAP"), ValueType::Struct(vec![
            ValueType::U16,       // w
            ValueType::U16,       // h
            ValueType::U16,       // parallax w
            ValueType::U16,       // parallax h
            ValueType::AssetRef,  // tileset
            ValueType::ArrayRef,  // tiles (u8)
        ])),

        (String::from("SPRITE_ANIMATION"), ValueType::Struct(vec![
            ValueType::ArrayRef,          // frame indices (u8)
            ValueType::AssetRef,          // sprite
            ValueType::Struct(vec![       // collision
                ValueType::U16,              // x
                ValueType::U16,              // y
                ValueType::U16,              // width
                ValueType::U16,              // height
            ]),
            ValueType::I8,                // use foot frames
            ValueType::I8,                // foot overlap
            ValueType::Loop(Box::new(     // 20 loops
                ValueType::Struct(vec![
                    ValueType::U16,       // offset
                    ValueType::U16,       // length
                ])
            )),
        ])),

        (String::from("ROOM"), ValueType::Struct(vec![
            ValueType::U16,       // num maps
            ValueType::U16,       // num triggers
            ValueType::ArrayRef,  // maps (ROOM_MAP_INFO)
            ValueType::ArrayRef,  // triggers (ROOM_TRIGGER_INFO)
        ])),

        (String::from("WORLD"), ValueType::Struct(vec![
            ValueType::U16,       // num regions
            ValueType::ArrayRef,  // regions (WORLD_REGION)
        ])),
    ])
});

// structs inside the room trigger data union
static ROOM_TRIGGER_VALUE_TYPES: LazyLock<HashMap<String,ValueType>> = LazyLock::new(|| {
    HashMap::from([
        (String::from("any"), ValueType::Struct(vec![
            ValueType::U32,       // data0
            ValueType::U32,       // data1
            ValueType::U32,       // data2
            ValueType::U32,       // data3
        ])),

        (String::from("door"), ValueType::Struct(vec![
            ValueType::AssetRef,  // room
            ValueType::U16,       // door id
        ])),

        (String::from("player_spawn"), ValueType::Struct(vec![
            ValueType::U8,        // direction
        ])),

        (String::from("enemy_spawn"), ValueType::Struct(vec![
            ValueType::AssetRef,  // animation
        ])),

        (String::from("trap"), ValueType::Struct(vec![
            ValueType::U16,       // width
            ValueType::U16,       // height
            ValueType::U16,       // trap
        ])),
    ])
});

fn custom_read_mod_sample_data(reader: &mut ProjectDataReader) -> Result<Value> {
    reader.read_sample_data_ref()
}

fn custom_read_sfx_sample_data(reader: &mut ProjectDataReader) -> Result<Value> {
    reader.read_sample_data_ref()
}

fn custom_read_room_trigger_info(reader: &mut ProjectDataReader) -> Result<Value> {
    reader.read_trigger_info_union()
}

pub struct AssetDef {
    pub value: Value,
    pub pos: TokenPosition,
}

pub struct ProjectData {
    prefix: String,
    got_prefix: bool,
    prefix_lower: String,
    prefix_upper: String,
    vga_bits_per_pixel: u8,
    vga_sync_bits: u8,

    arrays: HashMap<String, Value>,
    structs: HashMap<String, Vec<Value>>,
    assets: HashMap<String, (Vec<AssetDef>, TokenPosition)>,
    enums: HashMap<String,Vec<String>>,
    asset_ids: HashMap<String, Vec<DataAssetId>>,
    asset_ids_by_prefixed_name: HashMap<String, Vec<DataAssetId>>,
}

impl ProjectData {
    pub fn new() -> Self {
        ProjectData {
            prefix: String::from("PROJECT"),
            got_prefix: false,
            prefix_lower: String::new(),
            prefix_upper: String::new(),
            vga_bits_per_pixel: 6,
            vga_sync_bits: 0xc0,

            arrays: HashMap::new(),
            structs: HashMap::new(),
            assets: HashMap::new(),
            enums: HashMap::new(),
            asset_ids: HashMap::new(),
            asset_ids_by_prefixed_name: HashMap::new(),
        }
    }

    pub fn extract_asset_name<'a>(&self, array_prefix: &str, value_name: &'a ValueName) -> Result<&'a str> {
        if value_name.name.starts_with(&self.prefix_lower) &&
            value_name.name[self.prefix_lower.len() ..].starts_with(array_prefix) {
                Ok(&value_name.name[self.prefix_lower.len() + array_prefix.len() ..])
            } else {
                error(format!("invalid name: '{}'", value_name.name), value_name.pos)
            }

    }

    fn check_name_match(name: &str, parts: &[&str]) -> bool {
        if name.len() != parts.iter().fold(0, |len, p| len + p.len()) {
            return false;
        }

        let mut pos = 0;
        for part in parts {
            if ! name[pos .. pos + part.len()].eq_ignore_ascii_case(part) {
                return false;
            }
            pos += part.len();
        }
        true
    }

    // look for enum with tag
    //    <prefix><asset_type>_<asset_name>_<type_name>_NAMES
    // and get the nth item's <X> part if it's in the form
    //    <prefix><asset_type>_<asset_name>_<type_name>_<X>
    pub fn get_asset_data_name(&self, index: usize, asset_type: &str,
                               asset_name: &str, type_name: &str) -> Option<String> {
        let enum_tag_parts = &[ &self.prefix_upper, asset_type, "_", asset_name, "_", type_name, "_NAMES" ];
        let enum_item_prefix = &[ &self.prefix_upper, asset_type, "_", asset_name, "_", type_name, "_" ];
        let enum_item_prefix_len = enum_item_prefix.iter().fold(0, |len, p| len + p.len());
        for (name, enum_items) in self.enums.iter() {
            if Self::check_name_match(name, enum_tag_parts) {
                return if let Some(item_name) = enum_items.get(index) &&
                    item_name.len() > enum_item_prefix_len &&
                    Self::check_name_match(&item_name[..enum_item_prefix_len], enum_item_prefix) {
                        Some(String::from(&item_name[enum_item_prefix_len..]).to_lowercase())
                    } else {
                        None
                    };
            }
        }
        None
    }
}

pub struct ProjectDataReader<'a> {
    logger: &'a mut StringLogger,
    tok: Tokenizer<'a>,
    unread_token: Option<Token>,
    last_pos: TokenPosition,
    data: ProjectData,
}

impl<'a> ProjectDataReader<'a> {
    fn new(source: &'a str, logger: &'a mut StringLogger) -> Self {
        ProjectDataReader {
            logger,
            tok: Tokenizer::new(source),
            unread_token: None,
            last_pos: TokenPosition { line: 0 },
            data: ProjectData::new(),
        }
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

    fn unexpected(&self, t: &Token) -> Result<()> {
        error(format!("unexpected '{}'", t), t.pos)
    }

    fn expect_token(&mut self) -> Result<Token> {
        match self.read() {
            Ok(Token { data: TokenData::Eof(), pos }) => error("unexpected <eof>", pos),
            Ok(tok) if ! tok.is_eof() => Ok(tok),
            v => v,
        }
    }

    fn expect_ident(&mut self, ident: &str) -> Result<Token> {
        let t = self.read()?;
        if let Some(got_ident) = t.get_ident() && got_ident == ident {
            return Ok(t);
        }
        error(format!("expected {}, found '{}'", ident, t), t.pos)?
    }

    fn expect_any_ident(&mut self, expected: &str) -> Result<Token> {
        let t = self.read()?;
        if t.is_any_ident() {
            return Ok(t);
        }
        error(format!("expected {}, found '{}'", expected, t), t.pos)?
    }

    fn expect_punct(&mut self, ch: char) -> Result<Token> {
        let t = self.read()?;
        if t.is_punct(ch) {
            return Ok(t)
        }
        error(format!("expected '{}', found '{}'", ch, t), t.pos)?
    }

    // ========================================================
    // === NUMBER
    // ========================================================

    fn read_number(&mut self, max: u64) -> Result<u64> {
        let t = self.read()?;
        if let Some(n) = t.get_number() {
            if n <= max {
                return Ok(n)
            }
            error(format!("number too large: {} (must be <= {})", n, max), t.pos)
        } else {
            error(format!("expected number, found '{}'", t), t.pos)
        }
    }

    fn read_signed_number(&mut self, min: i64, max: i64) -> Result<i64> {
        let t = self.read()?;
        if let Some(n) = t.get_number() {
            if n > (i64::MAX as u64) || (n as i64) > max {
                return error(format!("number too large: {} (must be <= {})", n, max), t.pos);
            }
            return Ok(n as i64)
        }
        if t.is_punct('-') {
            let t = self.read()?;
            if let Some(n) = t.get_number() {
                if n > (i64::MAX as u64) || -(n as i64) < min {
                    return error(format!("number too small: {} (must be >= {})", n, min), t.pos);
                }
                return Ok(-(n as i64))
            }
            error(format!("expected number, found '{}'", t), t.pos)?
        }
        error(format!("expected '-' or number, found '{}'", t), t.pos)?
    }

    // =========================================================
    // === VALUE PARSER
    // =========================================================

    fn strip_upper_prefix(&self, ident: &'a str) -> Option<&'a str> {
        if ident.starts_with(&self.data.prefix_upper) {
            Some(&ident[self.data.prefix_upper.len()..])
        } else {
            None
        }
    }

    fn strip_lower_prefix(&self, ident: &'a str) -> Option<&'a str> {
        if ident.starts_with(&self.data.prefix_lower) {
            Some(&ident[self.data.prefix_lower.len()..])
        } else {
            None
        }
    }

    fn get_value_type(&self, type_name: &str, table: &'a HashMap<String,ValueType>) -> Option<&'a ValueType> {
        if let Some(name) = self.strip_upper_prefix(type_name) {
            table.get(name)
        } else {
            None
        }
    }

    fn read_loop(&mut self) -> Result<Option<Token>> {
        let t = self.expect_token()?;
        if t.is_punct('}') {
            return Ok(None);
        }
        if ! t.is_punct(',') {
            return Ok(Some(t));
        }
        let t = self.expect_token()?;
        if t.is_punct('}') {
            return Ok(None);
        }
        Ok(Some(t))
    }

    fn read_sample_data_ref(&mut self) -> Result<Value> {
        self.expect_punct('{')?;
        self.expect_punct('.')?;
        self.expect_any_ident("field name")?;
        self.expect_punct('=')?;
        let mut name_token = self.expect_any_ident("array name")?;
        self.expect_punct('}')?;
        if let Some(name) = name_token.drain_ident() {
            Ok(Value::ArrayRef(Box::new(ValueName::new(name, name_token.pos))))
        } else {
            error(format!("unexpected {}", name_token), name_token.pos)
        }
    }

    fn read_trigger_info_union(&mut self) -> Result<Value> {
        self.expect_punct('.')?;
        let mut trigger_type_token = self.expect_any_ident("field name")?;
        self.expect_punct('=')?;
        if let Some(trigger_type) = trigger_type_token.drain_ident() {
            if let Some(value_type) = ROOM_TRIGGER_VALUE_TYPES.get(&trigger_type) {
                self.read_value(value_type)
            } else {
                error(format!("unknown trigger type: {}", trigger_type), trigger_type_token.pos)
            }
        } else {
            error(format!("unexpected {}", trigger_type_token), trigger_type_token.pos)
        }
    }

    fn read_array<T>(&mut self, read_element: fn(&mut ProjectDataReader) -> Result<T>) -> Result<ValueArray<T>> {
        let mut data = Vec::new();
        let start = self.expect_punct('{')?;
        while let Some(t) = self.read_loop()? {
            self.unread(t)?;
            data.push(read_element(self)?);
        }
        Ok(ValueArray::new(data, start.pos))
    }

    fn read_value(&mut self, value_type: &ValueType) -> Result<Value> {
        match value_type {
            ValueType::U8  => { Ok(Value::U8 (self.read_number(u8::MAX  as u64)? as u8)) }
            ValueType::U16 => { Ok(Value::U16(self.read_number(u16::MAX as u64)? as u16)) }
            ValueType::U32 => { Ok(Value::U32(self.read_number(u32::MAX as u64)? as u32)) }
            ValueType::I8  => { Ok(Value::I8 (self.read_signed_number(i8::MIN  as i64, i8::MAX  as i64)? as i8)) }
            ValueType::I16 => { Ok(Value::I16(self.read_signed_number(i16::MIN as i64, i16::MAX as i64)? as i16)) }
            ValueType::I32 => { Ok(Value::I32(self.read_signed_number(i32::MIN as i64, i32::MAX as i64)? as i32)) }

            ValueType::U8Array  => {
                Ok(Value::U8Array(Box::new(
                    self.read_array(|r| { Ok(r.read_number(u8::MAX as u64)? as u8) })?
                )))
            }
            ValueType::U16Array => {
                Ok(Value::U16Array(Box::new(
                    self.read_array(|r| { Ok(r.read_number(u16::MAX as u64)? as u16) })?
                )))
            }
            ValueType::U32Array => {
                Ok(Value::U32Array(Box::new(
                    self.read_array(|r| { Ok(r.read_number(u32::MAX as u64)? as u32) })?
                )))
            }
            ValueType::I8Array  => {
                Ok(Value::I8Array(Box::new(
                    self.read_array(|r| { Ok(r.read_signed_number(i8::MIN as i64, i8::MAX as i64)? as i8) })?
                )))
            }
            ValueType::I16Array => {
                Ok(Value::I16Array(Box::new(
                    self.read_array(|r| { Ok(r.read_signed_number(i16::MIN as i64, i16::MAX as i64)? as i16) })?
                )))
            }

            ValueType::Identifier => {
                let mut ident_token = self.expect_any_ident("identifier")?;
                if let Some(ident) = ident_token.drain_ident() {
                    Ok(Value::Identifier(Box::new(ValueName::new(ident, ident_token.pos))))
                } else {
                    error(format!("unexpected '{}'", ident_token), ident_token.pos)
                }
            }

            ValueType::ArrayRef => {
                let mut ident_token = self.expect_any_ident("array name")?;
                if let Some(name) = ident_token.drain_ident() {
                    Ok(Value::ArrayRef(Box::new(ValueName::new(name, ident_token.pos))))
                } else {
                    error(format!("unexpected '{}'", ident_token), ident_token.pos)
                }
            }

            ValueType::AssetRef => {
                self.expect_punct('&')?;
                let mut ident_token = self.expect_any_ident("array name")?;
                self.expect_punct('[')?;
                let index = self.read_number(u32::MAX as u64)? as usize;
                self.expect_punct(']')?;
                if let Some(name) = ident_token.drain_ident() {
                    Ok(Value::AssetRef(Box::new(ValueAssetRef::new(name, index, ident_token.pos))))
                } else {
                    error(format!("unexpected '{}'", ident_token), ident_token.pos)
                }
            }

            ValueType::Struct(value_types) => {
                self.expect_punct('{')?;
                let mut values = Vec::new();
                for value_type in value_types.iter() {
                    values.push(self.read_value(value_type)?);
                    let comma = self.read()?;
                    if ! comma.is_punct(',') {
                        self.unread(comma)?;
                    }
                }
                self.expect_punct('}')?;
                Ok(Value::Struct(values))
            }

            ValueType::Loop(value_type) => {
                self.expect_punct('{')?;
                let mut values = Vec::new();
                while let Some(t) = self.read_loop()? {
                    self.unread(t)?;
                    values.push(self.read_value(value_type)?);
                }
                Ok(Value::Loop(values))
            }

            ValueType::Custom(reader) => {
                reader(self)
            }
        }
    }

    fn read_data_array(&mut self, type_name: &str) -> Result<()> {
        let mut name_token = self.expect_any_ident("array name")?;
        if let Some(name) = name_token.drain_ident() {
            self.expect_punct('[')?;
            self.expect_punct(']')?;
            self.expect_punct('=')?;
            if let Some(value_type) = ARRAY_VALUE_TYPES.get(type_name) {
                let value = self.read_value(value_type)?;
                self.data.arrays.insert(name, value);
            } else {
                return error(format!("unsupported array type: {}", type_name), name_token.pos);
            }
            self.expect_punct(';')?;
            Ok(())
        } else {
            self.unexpected(&name_token)
        }
    }

    fn read_data_struct_array(&mut self) -> Result<()> {
        let mut struct_tag_token = self.expect_any_ident("struct tag")?;
        let mut name_token = self.expect_any_ident("array name")?;
        if let Some(struct_tag) = struct_tag_token.drain_ident() && let Some(name) = name_token.drain_ident() {
            if let Some(value_type) = self.get_value_type(&struct_tag, &AUX_STRUCT_VALUE_TYPES) {
                self.expect_punct('[')?;
                self.expect_punct(']')?;
                self.expect_punct('=')?;
                self.expect_punct('{')?;
                let mut values = Vec::new();
                while let Some(t) = self.read_loop()? {
                    self.unread(t)?;
                    values.push(self.read_value(value_type)?);
                }
                self.expect_punct(';')?;
                self.data.structs.insert(name, values);
                Ok(())
            } else {
                error(format!("unknown struct tag: {}", struct_tag), struct_tag_token.pos)
            }
        } else {
            self.unexpected(&struct_tag_token)
        }
    }

    fn read_extern_data(&mut self) -> Result<()> {
        self.expect_ident("const")?;
        self.expect_ident("struct")?;
        self.expect_any_ident("struct tag")?;
        self.expect_any_ident("identifier")?;
        self.expect_punct(';')?;
        Ok(())
    }

    fn read_static_data(&mut self) -> Result<()> {
        self.expect_ident("const")?;
        let type_token = self.expect_any_ident("'struct' or type")?;
        if let Some(type_name) = type_token.get_ident() {
            if type_name == "struct" {
                self.read_data_struct_array()
            } else {
                self.read_data_array(type_name)
            }
        } else {
            self.unexpected(&type_token)
        }
    }

    fn read_asset_array(&mut self, value_type: &ValueType) -> Result<()> {
        let mut name_token = self.expect_any_ident("array name")?;
        if let Some(name) = name_token.drain_ident() {
            self.expect_punct('[')?;
            self.expect_punct(']')?;
            self.expect_punct('=')?;
            self.expect_punct('{')?;
            let mut values = Vec::new();
            while let Some(t) = self.read_loop()? {
                let pos = t.pos;
                self.unread(t)?;
                values.push(AssetDef {
                    value: self.read_value(value_type)?,
                    pos,
                });
            }
            self.data.assets.insert(name, (values, name_token.pos));
            self.expect_punct(';')?;
            Ok(())
        } else {
            self.unexpected(&name_token)
        }
    }

    fn read_global_data(&mut self) -> Result<()> {
        self.expect_ident("struct")?;
        let mut struct_tag_token = self.expect_any_ident("struct tag")?;
        if let Some(struct_tag) = struct_tag_token.drain_ident() {
            if let Some(value_type) = self.get_value_type(&struct_tag, &ASSET_STRUCT_VALUE_TYPES) {
                // read struct arrays of known assets
                self.read_asset_array(value_type)
            } else if let Some(prefixless_tag) = self.strip_upper_prefix(&struct_tag) && prefixless_tag == "ROOM_SCRIPT" {
                // ignore ROOM_SCRIPT struct array
                self.expect_punct('*')?;
                self.expect_any_ident("room script table identifier")?;
                self.expect_punct('[')?;
                self.expect_punct(']')?;
                self.expect_punct('=')?;
                self.expect_punct('{')?;
                while let Some(t) = self.read_loop()? {
                    if ! t.is_punct('&') {
                        return error(format!("expected '&', found '{}'", t), t.pos);
                    }
                    self.expect_any_ident("room script table")?;
                }
                self.expect_punct(';')?;
                Ok(())
            } else {
                error(format!("unknown struct '{}'", struct_tag), struct_tag_token.pos)
            }
        } else {
            self.unexpected(&struct_tag_token)
        }
    }

    fn read_enum(&mut self) -> Result<()> {
        let mut enum_tag_token = self.expect_any_ident("enum tag")?;
        if let Some(enum_tag) = enum_tag_token.drain_ident() {
            self.expect_punct('{')?;
            let mut values = Vec::new();
            while let Some(mut t) = self.read_loop()? {
                if let Some(name) = t.drain_ident() {
                    values.push(name);
                } else {
                    return error(format!("expected enum identifier, got '{}'", t), t.pos);
                }
            }
            self.data.enums.insert(enum_tag, values);
            self.expect_punct(';')?;
            Ok(())
        } else {
            self.unexpected(&enum_tag_token)
        }
    }

    // =========================================================
    // === PARSER DISPATCH
    // =========================================================

    fn read_data(&mut self) -> Result<()> {
        loop {
            let mut t = self.read()?;

            if t.is_eof() { break; }

            if let Some(line) = t.drain_pre_processor() {
                // read project prefix, vga_sync_bits, etc.
                // ignore #if/#else/#endif, log anything else
                pre_processor::handle_line(&line, &mut self.data, t.pos, self.logger)?;
                continue;
            }

            if ! self.data.got_prefix {
                return error(format!("must have #define with prefix before this line: {}", &t), t.pos);
            }

            if let Some(ident) = t.get_ident() {
                if ident == "extern" {
                    // room script table declarations: ignore
                    self.read_extern_data()?;
                    continue;
                } else if ident == "static" {
                    // asset data (image pixels, mod patterns, etc):
                    // read to (`data.arrays`, `data.structs`)
                    self.read_static_data()?;
                    continue;
                } else if ident == "const" {
                    // main asset structs: read to `data.asset`
                    self.read_global_data()?;
                    continue;
                } else if ident == "enum" {
                    // asset ids and sub-item names
                    // (room triggers, animation loops, etc):
                    // read to `data.enums`
                    self.read_enum()?;
                    continue;
                }
            }

            error(format!("unexpected '{}'", t), t.pos)?;
        }
        Ok(())
    }

    // =========================================================
    // === CONVERT DATA TO ASSETS
    // =========================================================

    fn create_store(mut self) -> Result<DataAssetStore> {
        // generate asset ids
        let mut id_generator = DataAssetIdGenerator::new();
        for (name, (data_vec, pos)) in self.data.assets.iter() {
            let ids: Vec<DataAssetId> = (0..data_vec.len()).map(|_| id_generator.gen_id()).collect();
            self.data.asset_ids_by_prefixed_name.insert(name.to_owned(), ids.clone());
            let unprefixed_name = self.strip_lower_prefix(name).ok_or_else(|| {
                err(format!("invalid asset array name: {}", name), *pos)
            })?;
            self.data.asset_ids.insert(unprefixed_name.to_owned(), ids);
        }

        // create assets
        let mut assets = AssetCollection::new();
        let mut asset_ids = AssetIdCollection::new();
        let image_converter = ImageConverter::new(self.data.vga_bits_per_pixel);
        for (name, (data_vec, data_pos)) in self.data.assets.iter() {
            if let Some(name) = self.strip_lower_prefix(name) && let Some(asset_ids_of_type) = self.data.asset_ids.get(name) {
                for (index, asset_data) in data_vec.iter().enumerate() {
                    match name {
                        "tilesets" => {
                            if let Some(&id) = asset_ids_of_type.get(index) {
                                asset_ids.tilesets.push(id);
                                assets.tilesets.insert(id, tileset::create(id, asset_data, &self.data, &image_converter)?);
                            }
                        }
                        "maps" => {
                            if let Some(&id) = asset_ids_of_type.get(index) {
                                asset_ids.maps.push(id);
                                assets.maps.insert(id, map_data::create(id, asset_data, &self.data)?);
                            }
                        }
                        "rooms" => {
                            if let Some(&id) = asset_ids_of_type.get(index) {
                                asset_ids.rooms.push(id);
                                assets.rooms.insert(id, room::create(id, asset_data, &self.data)?);
                            }
                        }
                        "worlds" => {
                            if let Some(&id) = asset_ids_of_type.get(index) {
                                asset_ids.worlds.push(id);
                                assets.worlds.insert(id, world::create(id, asset_data, &self.data)?);
                            }
                        }
                        "sprites" => {
                            if let Some(&id) = asset_ids_of_type.get(index) {
                                asset_ids.sprites.push(id);
                                assets.sprites.insert(id, sprite::create(id, asset_data, &self.data, &image_converter)?);
                            }
                        }
                        "pal_sprites" => {
                            if let Some(&id) = asset_ids_of_type.get(index) {
                                asset_ids.pal_sprites.push(id);
                                assets.pal_sprites.insert(id, pal_sprite::create(id, asset_data, &self.data)?);
                            }
                        }
                        "sprite_animations" => {
                            if let Some(&id) = asset_ids_of_type.get(index) {
                                asset_ids.animations.push(id);
                                assets.animations.insert(id, sprite_animation::create(id, asset_data, &self.data)?);
                            }
                        }
                        "sfxs" => {
                            if let Some(&id) = asset_ids_of_type.get(index) {
                                asset_ids.sfxs.push(id);
                                assets.sfxs.insert(id, sfx::create(id, asset_data, &self.data)?);
                            }
                        }
                        "mods" => {
                            if let Some(&id) = asset_ids_of_type.get(index) {
                                asset_ids.mods.push(id);
                                assets.mods.insert(id, mod_data::create(id, asset_data, &self.data)?);
                            }
                        }
                        "fonts" => {
                            if let Some(&id) = asset_ids_of_type.get(index) {
                                asset_ids.fonts.push(id);
                                assets.fonts.insert(id, font::create(id, asset_data, &self.data)?);
                            }
                        }
                        "prop_fonts" => {
                            if let Some(&id) = asset_ids_of_type.get(index) {
                                asset_ids.prop_fonts.push(id);
                                assets.prop_fonts.insert(id, prop_font::create(id, asset_data, &self.data)?);
                            }
                        }
                        _ => {
                            return error(format!("unknown asset array: {}", name), *data_pos);
                        }
                    }
                }
            }
        }
        Ok(DataAssetStore {
            id_generator,
            assets,
            asset_ids,
            project_prefix: self.data.prefix,
            vga_bits_per_pixel: self.data.vga_bits_per_pixel,
            vga_sync_bits: self.data.vga_sync_bits,
        })
    }

    pub fn read_from_string(input: &str, logger: &mut StringLogger) -> Result<DataAssetStore> {
        let mut reader = ProjectDataReader::new(input, logger);
        reader.read_data()?;
        reader.create_store()
    }
}
