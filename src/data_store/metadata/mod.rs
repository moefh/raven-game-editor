mod sprite;

use std::collections::HashMap;

pub use sprite::{*};

use super::{
    DataAssetId,
};

pub struct DataStoreMetadata {
    pub sprites: HashMap<DataAssetId,SpriteMetadata>,
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

    pub fn get_sprite_mut<'a>(&'a mut self, sprite_id: &DataAssetId) -> Option<&'a mut SpriteMetadata> {
        self.sprites.get_mut(sprite_id)
    }

    pub fn get_or_insert_sprite_mut(&mut self, sprite_id: DataAssetId) -> &mut SpriteMetadata {
        self.sprites.entry(sprite_id).or_insert_with(SpriteMetadata::new)
    }

    pub fn duplicate_sprite(&mut self, sprite_id: DataAssetId, dup_id: DataAssetId) {
        if let Some(sprite_md) = self.get_sprite(&sprite_id) {
            self.sprites.insert(dup_id, sprite_md.duplicate());
        }
    }

    pub fn remove_sprite(&mut self, sprite_id: DataAssetId) {
        self.sprites.remove(&sprite_id);
    }
}
