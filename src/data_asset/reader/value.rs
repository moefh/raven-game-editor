use std::io::Result;
use std::collections::HashMap;

use super::{
    err,
    error,
    TokenPosition,
    ProjectData,
    ProjectDataReader,
};
use super::super::DataAssetId;

#[derive(Debug)]
#[allow(dead_code)]
pub enum ValueDef {
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
    Struct(ValueDefStruct),
    StructArray(ValueDefStruct),
    Custom(fn (&mut ProjectDataReader) -> Result<Value>),
}

#[derive(Debug)]
pub struct ValueDefStruct {
    pub value_defs: Vec<(String, ValueDef)>,
}

impl ValueDefStruct {
    pub fn new(value_defs: Vec<(String, ValueDef)>) -> Self {
        ValueDefStruct {
            value_defs,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Value {
    U8(u8, TokenPosition),
    U16(u16, TokenPosition),
    U32(u32, TokenPosition),
    I8(i8, TokenPosition),
    I16(i16, TokenPosition),
    I32(i32, TokenPosition),
    U8Array(ValueArray<u8>),
    U16Array(ValueArray<u16>),
    U32Array(ValueArray<u32>),
    I8Array(ValueArray<i8>),
    I16Array(ValueArray<i16>),
    StructArray(ValueArray<ValueStruct>),
    Identifier(ValueName),
    ArrayRef(ValueName),
    AssetRef(ValueAssetRef),
    Struct(ValueStruct),
}

#[allow(dead_code)]
impl Value {
    fn get_type_name(&self) -> &'static str {
        match self {
            Value::U8(..) => { "u8" }
            Value::U16(..) => { "u16" }
            Value::U32(..) => { "u32" }
            Value::I8(..) => { "i8" }
            Value::I16(..) => { "i16" }
            Value::I32(..) => { "i32" }
            Value::U8Array(..) => { "u8[]" }
            Value::U16Array(..) => { "u16[]" }
            Value::U32Array(..) => { "u32[]" }
            Value::I8Array(..) => { "i8[]" }
            Value::I16Array(..) => { "i16[]" }
            Value::StructArray(..) => { "struct[]" }
            Value::Identifier(..) => { "identifier" }
            Value::ArrayRef(..) => { "array" }
            Value::AssetRef(..) => { "asset" }
            Value::Struct(..) => { "struct" }
        }
    }

    fn get_u8(&self) -> Option<u8> { if let Value::U8(v, _) = self { Some(*v) } else { None } }
    fn get_u16(&self) -> Option<u16> { if let Value::U16(v, _) = self { Some(*v) } else { None } }
    fn get_u32(&self) -> Option<u32> { if let Value::U32(v, _) = self { Some(*v) } else { None } }
    fn get_i8(&self) -> Option<i8> { if let Value::I8(v, _) = self { Some(*v) } else { None } }
    fn get_i16(&self) -> Option<i16> { if let Value::I16(v, _) = self { Some(*v) } else { None } }
    fn get_u8_array(&self) -> Option<&ValueArray<u8>> { if let Value::U8Array(v) = self { Some(v) } else { None } }
    fn get_u16_array(&self) -> Option<&ValueArray<u16>> { if let Value::U16Array(v) = self { Some(v) } else { None } }
    fn get_u32_array(&self) -> Option<&ValueArray<u32>> { if let Value::U32Array(v) = self { Some(v) } else { None } }
    fn get_i8_array(&self) -> Option<&ValueArray<i8>> { if let Value::I8Array(v) = self { Some(v) } else { None } }
    fn get_i16_array(&self) -> Option<&ValueArray<i16>> { if let Value::I16Array(v) = self { Some(v) } else { None } }
    fn get_struct_array(&self) -> Option<&ValueArray<ValueStruct>> { if let Value::StructArray(v) = self { Some(v) } else { None } }
    fn get_identifier(&self) -> Option<&ValueName> { if let Value::Identifier(v) = self { Some(v) } else { None } }
    fn get_array_ref(&self) -> Option<&ValueName> { if let Value::ArrayRef(v) = self { Some(v) } else { None } }
    fn get_asset_ref(&self) -> Option<&ValueAssetRef> { if let Value::AssetRef(v) = self { Some(v) } else { None } }
    fn get_struct(&self) -> Option<&ValueStruct> { if let Value::Struct(v) = self { Some(v) } else { None } }
}

#[derive(Debug)]
pub struct ValueStruct {
    pub values: HashMap<String, Value>,
    pub pos: TokenPosition,
}

impl ValueStruct {
    pub fn new(values: HashMap<String, Value>, pos: TokenPosition) -> Self {
        ValueStruct {
            values,
            pos,
        }
    }

    pub fn get_value(&self, name: &str) -> Result<&Value> {
        self.values.get(name).ok_or_else(|| {
            err(format!("struct member '{}' not found", name), self.pos)
        })
    }

    pub fn get_u8(&self, name: &str) -> Result<u8> {
        self.get_value(name).and_then(|value| { value.get_u8().ok_or_else(|| {
            err(format!("invalid value in struct member '{}': expected 'u8', found '{:?}'", name, value.get_type_name()), self.pos)
        })})
    }

    pub fn get_u16(&self, name: &str) -> Result<u16> {
        self.get_value(name).and_then(|value| { value.get_u16().ok_or_else(|| {
            err(format!("invalid value in struct member '{}': expected 'u16', found '{:?}'", name, value.get_type_name()), self.pos)
        })})
    }

    pub fn get_u32(&self, name: &str) -> Result<u32> {
        self.get_value(name).and_then(|value| { value.get_u32().ok_or_else(|| {
            err(format!("invalid value in struct member '{}': expected 'u32', found '{:?}'", name, value.get_type_name()), self.pos)
        })})
    }

    pub fn get_i8(&self, name: &str) -> Result<i8> {
        self.get_value(name).and_then(|value| { value.get_i8().ok_or_else(|| {
            err(format!("invalid value in struct member '{}': expected 'i8', found '{:?}'", name, value.get_type_name()), self.pos)
        })})
    }

    pub fn get_i16(&self, name: &str) -> Result<i16> {
        self.get_value(name).and_then(|value| { value.get_i16().ok_or_else(|| {
            err(format!("invalid value in struct member '{}': expected 'i16', found '{:?}'", name, value.get_type_name()), self.pos)
        })})
    }

    pub fn get_u8_array(&self, name: &str) -> Result<&ValueArray<u8>> {
        self.get_value(name).and_then(|value| { value.get_u8_array().ok_or_else(|| {
            err(format!("invalid value in struct member '{}': expected 'u8[]', found '{:?}'", name, value.get_type_name()), self.pos)
        })})
    }

    pub fn get_u16_array(&self, name: &str) -> Result<&ValueArray<u16>> {
        self.get_value(name).and_then(|value| { value.get_u16_array().ok_or_else(|| {
            err(format!("invalid value in struct member '{}': expected 'u16[]', found '{:?}'", name, value.get_type_name()), self.pos)
        })})
    }

    pub fn get_struct_array(&self, name: &str) -> Result<&ValueArray<ValueStruct>> {
        self.get_value(name).and_then(|value| { value.get_struct_array().ok_or_else(|| {
            err(format!("invalid value in struct member '{}': expected 'struct[]', found '{:?}'", name, value.get_type_name()), self.pos)
        })})
    }

    pub fn get_identifier(&self, name: &str) -> Result<&ValueName> {
        self.get_value(name).and_then(|value| { value.get_identifier().ok_or_else(|| {
            err(format!("invalid value in struct member '{}': expected 'identifier', found '{:?}'", name, value.get_type_name()), self.pos)
        })})
    }

    pub fn get_array_ref(&self, name: &str) -> Result<&ValueName> {
        self.get_value(name).and_then(|value| { value.get_array_ref().ok_or_else(|| {
            err(format!("invalid value in struct member '{}': expected 'array', found '{:?}'", name, value.get_type_name()), self.pos)
        })})
    }

    pub fn get_asset_ref(&self, name: &str) -> Result<&ValueAssetRef> {
        self.get_value(name).and_then(|value| { value.get_asset_ref().ok_or_else(|| {
            err(format!("invalid value in struct member '{}': expected 'asset', found '{:?}'", name, value.get_type_name()), self.pos)
        })})
    }

    pub fn get_struct(&self, name: &str) -> Result<&ValueStruct> {
        self.get_value(name).and_then(|value| { value.get_struct().ok_or_else(|| {
            err(format!("invalid value in struct member '{}': expected 'struct', found '{:?}'", name, value.get_type_name()), self.pos)
        })})
    }
}

#[derive(Debug)]
pub struct ValueArray<T> {
    pub values: Vec<T>,
    pub pos: TokenPosition,
}

impl<T> ValueArray<T> {
    pub fn new(values: Vec<T>, pos: TokenPosition) -> Self {
        ValueArray {
            values,
            pos,
        }
    }
}

pub enum ValueArrayDataI8orI16<'a> {
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
    pub name: String,
    pub pos: TokenPosition,
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

    pub fn get_struct_array<'a>(&self, data: &'a ProjectData) -> Result<&'a ValueArray<ValueStruct>> {
        match data.structs.get(&self.name) {
            Some(v) => { Ok(v) }
            _ => { self.error_not_found(data) }
        }
    }

    pub fn get_u8_array<'a>(&self, data: &'a ProjectData) -> Result<&'a Vec<u8>> {
        match data.arrays.get(&self.name) {
            Some(Value::U8Array(a))  => { Ok(&a.values) }
            _ => { self.error_not_found(data) }
        }
    }

    pub fn get_u16_array<'a>(&self, data: &'a ProjectData) -> Result<&'a Vec<u16>> {
        match data.arrays.get(&self.name) {
            Some(Value::U16Array(a))  => { Ok(&a.values) }
            _ => { self.error_not_found(data) }
        }
    }

    pub fn get_u32_array<'a>(&self, data: &'a ProjectData) -> Result<&'a Vec<u32>> {
        match data.arrays.get(&self.name) {
            Some(Value::U32Array(a))  => { Ok(&a.values) }
            _ => { self.error_not_found(data) }
        }
    }

    pub fn get_i8_array<'a>(&self, data: &'a ProjectData) -> Result<&'a Vec<i8>> {
        match data.arrays.get(&self.name) {
            Some(Value::I8Array(a))  => { Ok(&a.values) }
            _ => { self.error_not_found(data) }
        }
    }

    pub fn get_i16_array<'a>(&self, data: &'a ProjectData) -> Result<&'a Vec<i16>> {
        match data.arrays.get(&self.name) {
            Some(Value::I16Array(a))  => { Ok(&a.values) }
            _ => { self.error_not_found(data) }
        }
    }

    pub fn get_i8_or_i16_array<'a>(&self, data: &'a ProjectData) -> Result<ValueArrayDataI8orI16<'a>> {
        match data.arrays.get(&self.name) {
            Some(Value::I8Array(v)) => {
                Ok(ValueArrayDataI8orI16::I8Converted(v.values.iter().map(|s| { (*s as i16) << 8 }).collect()))
            }
            Some(Value::I16Array(v)) => {
                Ok(ValueArrayDataI8orI16::I16Original(&v.values))
            }
            _ => {
                self.error_not_found(data)
            }
        }
    }
}

#[derive(Debug)]
pub struct ValueAssetRef {
    pub name: String,
    pub index: usize,
    pub pos: TokenPosition,
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
