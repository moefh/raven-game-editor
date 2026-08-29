mod tileset;

use crate::data_asset::{
    DataAssetStore,
    Tileset,
};

pub use tileset::{*};

pub fn get_tileset_by_name<'a>(store: &'a DataAssetStore, name: &str) -> Option<&'a Tileset> {
    store.assets.tilesets.iter().find(|tileset| tileset.asset.name == name)
}

pub struct ProjectComparer {
    pub tilesets: TilesetListDiff,
}

impl ProjectComparer {
    pub fn new() -> Self {
        ProjectComparer {
            tilesets: TilesetListDiff::new(),
        }
    }

    pub fn run(&mut self, cur_store: &DataAssetStore, other_store: &DataAssetStore) {
        self.tilesets.compare(cur_store, other_store);
    }
}
