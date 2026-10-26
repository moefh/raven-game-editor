mod sprite;

use std::io::Result;

use super::ProjectDataWriter;
use super::super::metadata::{*};

fn write_array<I: std::fmt::Display>(writer: &ProjectDataWriter, array: &[I]) {
    for (index, item) in array.iter().enumerate() {
        if index != 0 { writer.write(","); }
        writer.write(format!("{}", item));
    }
}

pub fn write(writer: &ProjectDataWriter, metadata: &DataStoreMetadata) -> Result<()> {
    sprite::write(writer, &writer.store.asset_ids.sprites, &metadata.sprites)?;

    Ok(())
}
