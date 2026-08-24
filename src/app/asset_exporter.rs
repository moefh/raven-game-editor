use std::collections::HashMap;

use crate::data_asset::{
    DataAssetId,
    DataAssetType,
    DataAssetStore,
    StringLogger,
};

use super::{
    SysDialogs,
    SysDialogResponse,
    SysDialogOpenFile,
};

pub struct AssetExporter {
    pending_requests: HashMap<String, DataAssetId>,
}

impl AssetExporter {
    pub fn new() -> Self {
        AssetExporter {
            pending_requests: HashMap::new(),
        }
    }

    pub fn add_request(&mut self, sys_dialog_id: String, asset_id: DataAssetId) {
        self.pending_requests.insert(sys_dialog_id, asset_id);
    }

    pub fn check_dialog_response(
        &mut self,
        sys_dialogs: &mut SysDialogs,
        store: &DataAssetStore,
        logger: &mut StringLogger
    ) -> bool {
        let mut has_errors = false;
        self.pending_requests.retain(|request_id, asset_id| {
            if let Some(SysDialogResponse::File(file)) = sys_dialogs.get_response_for(request_id) {
                if Self::export_asset(file, *asset_id, store, logger) {
                    has_errors = true;
                }
                false
            } else {
                true
            }
        });
        has_errors
    }

    fn export_asset(file: SysDialogOpenFile, asset_id: DataAssetId, store: &DataAssetStore, logger: &mut StringLogger) -> bool {
        if let Some(asset) = store.assets.get_asset(asset_id) {
            match asset.asset_type {
                DataAssetType::MapData => { Self::export_map(file, asset_id, store, logger) }
                DataAssetType::Room => { Self::export_room(file, asset_id, store, logger) }
                DataAssetType::SpriteAnimation => { Self::export_sprite_animation(file, asset_id, store, logger) }
                _ => {
                    logger.log(format!("ERROR: exporting asset type {:?} not implemented!", asset.asset_type));
                    true
                }
            }
        } else {
            logger.log(format!("ERROR: exporting asset {}: asset not found!", asset_id));
            true
        }
    }

    fn export_map(file: SysDialogOpenFile, map_id: DataAssetId, store: &DataAssetStore, logger: &mut StringLogger) -> bool {
        logger.log("WRITING MAP");
        match store.serialize_map(map_id, logger).and_then(|content| file.write_string(content)) {
            Ok(()) => {
                logger.log(format!("DONE: map saved to {}", file.filename()));
                false
            }
            Err(e) => {
                logger.log(format!("ERROR exporting map: {}", e));
                true
            }
        }
    }

    fn export_room(file: SysDialogOpenFile, room_id: DataAssetId, store: &DataAssetStore, logger: &mut StringLogger) -> bool {
        logger.log("WRITING ROOM");
        match store.serialize_room(room_id, logger).and_then(|content| file.write_string(content)) {
            Ok(()) => {
                logger.log(format!("DONE: room saved to {}", file.filename()));
                false
            }
            Err(e) => {
                logger.log(format!("ERROR exporting room: {}", e));
                true
            }
        }
    }

    fn export_sprite_animation(file: SysDialogOpenFile, animation_id: DataAssetId, store: &DataAssetStore, logger: &mut StringLogger) -> bool {
        logger.log("WRITING SPRITE ANIMATION");
        match store.serialize_sprite_animation(animation_id, logger).and_then(|content| file.write_string(content)) {
            Ok(()) => {
                logger.log(format!("DONE: sprite animation saved to {}", file.filename()));
                false
            }
            Err(e) => {
                logger.log(format!("ERROR exporting sprite animation: {}", e));
                true
            }
        }
    }

}
