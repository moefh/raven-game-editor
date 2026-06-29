use crate::data_asset::{
    DataAssetId,
    DataAssetStore,
};
use crate::editors::{
    fix_editors_after_tiles_added,
    fix_editors_after_tiles_removed,
};
use super::{
    WindowContext,
    AssetEditors,
};

pub enum EditorAction {
    FixMapsAfterTilesAdded { tileset_id: DataAssetId, tile_index: u8, num_tiles: u8 },
    FixMapsAfterTilesRemoved { tileset_id: DataAssetId, tile_index: u8, num_tiles: u8 },
}

impl EditorAction {
    pub fn run(self, wc: &mut WindowContext, store: &mut DataAssetStore, editors: &mut AssetEditors) {
        match self {
            EditorAction::FixMapsAfterTilesAdded { tileset_id, tile_index, num_tiles } => {
                fix_editors_after_tiles_added(wc, store, editors, tileset_id, tile_index, num_tiles);
            }

            EditorAction::FixMapsAfterTilesRemoved { tileset_id, tile_index, num_tiles } => {
                fix_editors_after_tiles_removed(wc, store, editors, tileset_id, tile_index, num_tiles);
            }
        }
    }
}
