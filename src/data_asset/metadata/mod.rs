mod sprite;

use std::collections::HashMap;

pub use sprite::{*};

use super::{
    DataAssetId,
};

pub struct DataStoreMetadata {
    sprites: HashMap<DataAssetId,SpriteMetadata>,
}

impl DataStoreMetadata {
    pub fn new() -> Self {
        DataStoreMetadata {
            sprites: HashMap::new(),
        }
    }

    pub fn get_sprite<'a>(&'a self, sprite_id: &DataAssetId) -> Option<&'a SpriteMetadata> {
        self.sprites.get(sprite_id)
    }

    pub fn get_sprite_mut<'a>(&'a mut self, sprite_id: &DataAssetId) -> &'a mut SpriteMetadata {
        self.sprites.entry(*sprite_id).or_insert_with(SpriteMetadata::new)
    }
}
