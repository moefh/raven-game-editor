pub mod tokenizer;
mod value;
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
use value::{*};

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


// simple numeric arrays
static ARRAY_DEFS: LazyLock<HashMap<String,ValueDef>> = LazyLock::new(|| {
    HashMap::from([
        (String::from("uint8_t"),  ValueDef::U8Array),
        (String::from("uint16_t"), ValueDef::U16Array),
        (String::from("uint32_t"), ValueDef::U32Array),
        (String::from("int8_t"),   ValueDef::I8Array),
        (String::from("int16_t"),  ValueDef::I16Array),
    ])
});

// asset structs
static ASSET_STRUCT_DEFS: LazyLock<HashMap<String,ValueDefStruct>> = LazyLock::new(|| {
    HashMap::from([
        (String::from("SFX"), sfx::get_asset_def()),
        (String::from("MAP"), map_data::get_asset_def()),
        (String::from("ROOM"), room::get_asset_def()),
        (String::from("FONT"), font::get_asset_def()),
        (String::from("WORLD"), world::get_asset_def()),
        (String::from("IMAGE"), tileset::get_asset_def()),
        (String::from("MOD_DATA"), mod_data::get_asset_def()),
        (String::from("PROP_FONT"), prop_font::get_asset_def()),
        (String::from("PAL_SPRITE"), pal_sprite::get_asset_def()),
        (String::from("SPRITE_ANIMATION"), sprite_animation::get_asset_def()),
    ])
});

// structs in global arrays that store data used by the asset structs
static GLOBAL_STRUCT_DEFS: LazyLock<HashMap<String,ValueDefStruct>> = LazyLock::new(|| {
    let global_struct_defs_gen_table = &[
        mod_data::get_global_struct_defs,
        room::get_global_struct_defs,
        world::get_global_struct_defs,
    ];
    let mut struct_defs = HashMap::new();
    for global_struct_defs_gen in global_struct_defs_gen_table {
        for (name, struct_def) in global_struct_defs_gen().into_iter() {
            struct_defs.insert(name, struct_def);
        }
    }
    struct_defs
});

// other custom global structs (e.g. ROOM_SCRIPT)
const CUSTOM_GLOBAL_STRUCT_READERS: &[fn(&mut ProjectDataReader, &str) -> Result<bool>] = &[
    room::read_custom_global_struct,
];

pub struct ProjectData {
    prefix: String,
    got_prefix: bool,
    prefix_lower: String,
    prefix_upper: String,
    vga_bits_per_pixel: u8,
    vga_sync_bits: u8,
    tiles_per_world_block: u32,

    arrays: HashMap<String, Value>,
    structs: HashMap<String, ValueArray<ValueStruct>>,
    assets: HashMap<String, (Vec<ValueStruct>, TokenPosition)>,
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
            tiles_per_world_block: 22,

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
    pub fn get_asset_data_name(
        &self,
        index: usize,
        asset_type: &str,
        asset_name: &str,
        type_name: &str
    ) -> Option<String> {
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

    fn get_struct_def(&self, struct_name: &str, table: &'a HashMap<String,ValueDefStruct>) -> Option<&'a ValueDefStruct> {
        if let Some(name) = self.strip_upper_prefix(struct_name) {
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
            Ok(Value::ArrayRef(ValueName::new(name, name_token.pos)))
        } else {
            error(format!("unexpected {}", name_token), name_token.pos)
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

    fn read_struct(&mut self, struct_def: &ValueDefStruct) -> Result<ValueStruct> {
        let start = self.expect_punct('{')?;
        let mut values = HashMap::new();
        for (name, value_def) in struct_def.value_defs.iter() {
            values.insert(name.clone(), self.read_value(value_def)?);
            let comma = self.read()?;
            if ! comma.is_punct(',') {
                self.unread(comma)?;
            }
        }
        self.expect_punct('}')?;
        Ok(ValueStruct::new(values, start.pos))
    }

    fn read_value(&mut self, value_type: &ValueDef) -> Result<Value> {
        match value_type {
            ValueDef::U8  => { Ok(Value::U8 (self.read_number(u8::MAX  as u64)? as u8, self.last_pos)) }
            ValueDef::U16 => { Ok(Value::U16(self.read_number(u16::MAX as u64)? as u16, self.last_pos)) }
            ValueDef::U32 => { Ok(Value::U32(self.read_number(u32::MAX as u64)? as u32, self.last_pos)) }
            ValueDef::I8  => { Ok(Value::I8 (self.read_signed_number(i8::MIN  as i64, i8::MAX  as i64)? as i8, self.last_pos)) }
            ValueDef::I16 => { Ok(Value::I16(self.read_signed_number(i16::MIN as i64, i16::MAX as i64)? as i16, self.last_pos)) }
            ValueDef::I32 => { Ok(Value::I32(self.read_signed_number(i32::MIN as i64, i32::MAX as i64)? as i32, self.last_pos)) }

            ValueDef::U8Array => {
                Ok(Value::U8Array(
                    self.read_array(|r| { Ok(r.read_number(u8::MAX as u64)? as u8) })?
                ))
            }
            ValueDef::U16Array => {
                Ok(Value::U16Array(
                    self.read_array(|r| { Ok(r.read_number(u16::MAX as u64)? as u16) })?
                ))
            }
            ValueDef::U32Array => {
                Ok(Value::U32Array(
                    self.read_array(|r| { Ok(r.read_number(u32::MAX as u64)? as u32) })?
                ))
            }
            ValueDef::I8Array => {
                Ok(Value::I8Array(
                    self.read_array(|r| { Ok(r.read_signed_number(i8::MIN as i64, i8::MAX as i64)? as i8) })?
                ))
            }
            ValueDef::I16Array => {
                Ok(Value::I16Array(
                    self.read_array(|r| { Ok(r.read_signed_number(i16::MIN as i64, i16::MAX as i64)? as i16) })?
                ))
            }

            ValueDef::StructArray(struct_def) => {
                let start = self.expect_punct('{')?;
                let mut values = Vec::new();
                while let Some(t) = self.read_loop()? {
                    self.unread(t)?;
                    values.push(self.read_struct(struct_def)?);
                }
                Ok(Value::StructArray(ValueArray::new(values, start.pos)))
            }

            ValueDef::Identifier => {
                let mut ident_token = self.expect_any_ident("identifier")?;
                if let Some(ident) = ident_token.drain_ident() {
                    Ok(Value::Identifier(ValueName::new(ident, ident_token.pos)))
                } else {
                    error(format!("unexpected '{}'", ident_token), ident_token.pos)
                }
            }

            ValueDef::ArrayRef => {
                let mut ident_token = self.expect_any_ident("array name")?;
                if let Some(name) = ident_token.drain_ident() {
                    Ok(Value::ArrayRef(ValueName::new(name, ident_token.pos)))
                } else {
                    error(format!("unexpected '{}'", ident_token), ident_token.pos)
                }
            }

            ValueDef::AssetRef => {
                self.expect_punct('&')?;
                let mut ident_token = self.expect_any_ident("array name")?;
                self.expect_punct('[')?;
                let index = self.read_number(u32::MAX as u64)? as usize;
                self.expect_punct(']')?;
                if let Some(name) = ident_token.drain_ident() {
                    Ok(Value::AssetRef(ValueAssetRef::new(name, index, ident_token.pos)))
                } else {
                    error(format!("unexpected '{}'", ident_token), ident_token.pos)
                }
            }

            ValueDef::Struct(value_defs) => {
                Ok(Value::Struct(self.read_struct(value_defs)?))
            }

            ValueDef::Custom(reader) => {
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
            if let Some(value_type) = ARRAY_DEFS.get(type_name) {
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
            if let Some(struct_def) = self.get_struct_def(&struct_tag, &GLOBAL_STRUCT_DEFS) {
                self.expect_punct('[')?;
                self.expect_punct(']')?;
                self.expect_punct('=')?;
                self.expect_punct('{')?;
                let mut values = Vec::new();
                while let Some(t) = self.read_loop()? {
                    self.unread(t)?;
                    values.push(self.read_struct(struct_def)?);
                }
                self.expect_punct(';')?;
                self.data.structs.insert(name, ValueArray::new(values, name_token.pos));
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

    fn read_asset_array(&mut self, struct_def: &ValueDefStruct) -> Result<()> {
        let mut name_token = self.expect_any_ident("array name for")?;
        if let Some(name) = name_token.drain_ident() {
            self.expect_punct('[')?;
            self.expect_punct(']')?;
            self.expect_punct('=')?;
            self.expect_punct('{')?;
            let mut values = Vec::new();
            while let Some(t) = self.read_loop()? {
                self.unread(t)?;
                values.push(self.read_struct(struct_def)?);
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
            if let Some(struct_def) = self.get_struct_def(&struct_tag, &ASSET_STRUCT_DEFS) {
                // read struct arrays of known assets
                self.read_asset_array(struct_def)?;
                return Ok(());
            } else if let Some(prefixless_tag) = self.strip_upper_prefix(&struct_tag) {
                // read custom structs
                for custom_reader in CUSTOM_GLOBAL_STRUCT_READERS {
                    if custom_reader(self, prefixless_tag)? {
                        return Ok(())
                    }
                }
            }
            error(format!("unknown struct '{}'", struct_tag), struct_tag_token.pos)
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
                return error(format!("must have #define with prefix before this line: {}", t), t.pos);
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
        for (name, (asset_structs, pos)) in self.data.assets.iter() {
            let ids: Vec<DataAssetId> = (0..asset_structs.len()).map(|_| id_generator.gen_id()).collect();
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
        for (name, (asset_structs, data_pos)) in self.data.assets.iter() {
            if let Some(name) = self.strip_lower_prefix(name) && let Some(asset_ids_of_type) = self.data.asset_ids.get(name) {
                for (index, asset_struct) in asset_structs.iter().enumerate() {
                    match name {
                        "tilesets" => {
                            if let Some(&id) = asset_ids_of_type.get(index) {
                                asset_ids.tilesets.push(id);
                                assets.tilesets.insert(id, tileset::create(id, asset_struct, &self.data, &image_converter)?);
                            }
                        }
                        "maps" => {
                            if let Some(&id) = asset_ids_of_type.get(index) {
                                asset_ids.maps.push(id);
                                assets.maps.insert(id, map_data::create(id, asset_struct, &self.data)?);
                            }
                        }
                        "rooms" => {
                            if let Some(&id) = asset_ids_of_type.get(index) {
                                asset_ids.rooms.push(id);
                                assets.rooms.insert(id, room::create(id, asset_struct, &self.data)?);
                            }
                        }
                        "worlds" => {
                            if let Some(&id) = asset_ids_of_type.get(index) {
                                asset_ids.worlds.push(id);
                                assets.worlds.insert(id, world::create(id, asset_struct, &self.data)?);
                            }
                        }
                        "sprites" => {
                            if let Some(&id) = asset_ids_of_type.get(index) {
                                asset_ids.sprites.push(id);
                                assets.sprites.insert(id, sprite::create(id, asset_struct, &self.data, &image_converter)?);
                            }
                        }
                        "pal_sprites" => {
                            if let Some(&id) = asset_ids_of_type.get(index) {
                                asset_ids.pal_sprites.push(id);
                                assets.pal_sprites.insert(id, pal_sprite::create(id, asset_struct, &self.data)?);
                            }
                        }
                        "sprite_animations" => {
                            if let Some(&id) = asset_ids_of_type.get(index) {
                                asset_ids.animations.push(id);
                                assets.animations.insert(id, sprite_animation::create(id, asset_struct, &self.data)?);
                            }
                        }
                        "sfxs" => {
                            if let Some(&id) = asset_ids_of_type.get(index) {
                                asset_ids.sfxs.push(id);
                                assets.sfxs.insert(id, sfx::create(id, asset_struct, &self.data)?);
                            }
                        }
                        "mods" => {
                            if let Some(&id) = asset_ids_of_type.get(index) {
                                asset_ids.mods.push(id);
                                assets.mods.insert(id, mod_data::create(id, asset_struct, &self.data)?);
                            }
                        }
                        "fonts" => {
                            if let Some(&id) = asset_ids_of_type.get(index) {
                                asset_ids.fonts.push(id);
                                assets.fonts.insert(id, font::create(id, asset_struct, &self.data)?);
                            }
                        }
                        "prop_fonts" => {
                            if let Some(&id) = asset_ids_of_type.get(index) {
                                asset_ids.prop_fonts.push(id);
                                assets.prop_fonts.insert(id, prop_font::create(id, asset_struct, &self.data)?);
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
            tiles_per_world_block: self.data.tiles_per_world_block,
        })
    }

    pub fn read_from_string(input: &str, logger: &mut StringLogger) -> Result<DataAssetStore> {
        let mut reader = ProjectDataReader::new(input, logger);
        reader.read_data()?;
        reader.create_store()
    }
}
