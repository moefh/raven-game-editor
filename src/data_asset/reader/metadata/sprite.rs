use std::io::Result;

use super::super::{
    ValueStruct,
    ValueDef,
    ValueDefStruct,
};
use super::super::super::metadata::{
    SpriteMetadata,
    SpriteFrameCollection,
};

pub fn get_metadata_def() -> ValueDefStruct {
    ValueDefStruct::new(vec![
        (String::from("frames"), ValueDef::StructArray(
            ValueDefStruct::new(vec![
                (String::from("name"), ValueDef::Identifier),
                (String::from("indices"), ValueDef::U32Array),
            ])
        )),
    ])
}

pub fn create(md_struct: &ValueStruct) -> Result<SpriteMetadata> {
    let mut frames = Vec::new();

    let md_frames = md_struct.get_struct_array("frames")?;
    for md_frame in md_frames.values.iter() {
        let ident = md_frame.get_identifier("name")?;
        let indices = md_frame.get_u32_array("indices")?;
        frames.push(SpriteFrameCollection {
            name: ident.name.clone(),
            indices: indices.values.clone(),
        });
    }

    Ok(SpriteMetadata {
        frames,
    })
}
