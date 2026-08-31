use crate::data_asset::{
    DataAssetId,
    DataAssetStore,
};

use super::{
    fix_after_tileset_tiles_added,
    fix_after_tileset_tiles_removed,
    fix_after_tileset_tiles_shuffled,
    fix_after_sprite_frames_added,
    fix_after_sprite_frames_removed,
    fix_after_pal_sprite_frames_added,
    fix_after_pal_sprite_frames_removed,
    WindowContext,
    EditorStore,
};
use super::super::AssetExporter;

pub enum EditorAction {
    ExportMap { map_id: DataAssetId },
    ExportRoom { room_id: DataAssetId },
    ExportSpriteAnimation { animation_id: DataAssetId },
    TilesetTilesAdded { tileset_id: DataAssetId, hole_start: u8, hole_size: u8, num_tiles_after_hole: u8 },
    TilesetTilesRemoved { tileset_id: DataAssetId, hole_start: u8, hole_size: u8, num_tiles_after_hole: u8 },
    TilesetTilesShuffled { tileset_id: DataAssetId, shuffle: Vec<u32> },
    SpriteFramesAdded { sprite_id: DataAssetId, hole_start: u32, hole_size: u32, num_frames_after_hole: u32 },
    SpriteFramesRemoved { sprite_id: DataAssetId, hole_start: u32, hole_size: u32, num_frames_after_hole: u32 },
    PalSpriteFramesAdded { pal_sprite_id: DataAssetId, hole_start: u32, hole_size: u32, num_frames_after_hole: u32 },
    PalSpriteFramesRemoved { pal_sprite_id: DataAssetId, hole_start: u32, hole_size: u32, num_frames_after_hole: u32 },
}

impl EditorAction {
    pub fn run(self, wc: &mut WindowContext, store: &mut DataAssetStore, editors: &mut EditorStore, exporter: &mut AssetExporter) {
        match self {
            EditorAction::ExportMap { map_id } => {
                let request_id = format!("export_map_{}", map_id);
                exporter.add_request(request_id.clone(), map_id);
                wc.sys_dialogs.save_file(
                    Some(wc.egui.window),
                    request_id,
                    "map",
                    "Export Map",
                    &[
                        ("Raven map files (*.ravmap)", &["ravmap"]),
                        ("All files (*.*)", &["*"]),
                    ]
                );
            }

            EditorAction::ExportRoom { room_id } => {
                let request_id = format!("export_room_{}", room_id);
                exporter.add_request(request_id.clone(), room_id);
                wc.sys_dialogs.save_file(
                    Some(wc.egui.window),
                    request_id,
                    "room",
                    "Export Room",
                    &[
                        ("Raven room files (*.ravroom)", &["ravroom"]),
                        ("All files (*.*)", &["*"]),
                    ]
                );
            }

            EditorAction::ExportSpriteAnimation { animation_id } => {
                let request_id = format!("export_sprite_animation_{}", animation_id);
                exporter.add_request(request_id.clone(), animation_id);
                wc.sys_dialogs.save_file(
                    Some(wc.egui.window),
                    request_id,
                    "sprite_animation",
                    "Export Sprite Animation",
                    &[
                        ("Raven sprite animation files (*.ravanim)", &["ravanim"]),
                        ("All files (*.*)", &["*"]),
                    ]
                );
            }

            EditorAction::TilesetTilesAdded { tileset_id, hole_start, hole_size, num_tiles_after_hole } => {
                fix_after_tileset_tiles_added(wc, store, editors, tileset_id, hole_start, hole_size, num_tiles_after_hole);
            }

            EditorAction::TilesetTilesRemoved { tileset_id, hole_start, hole_size, num_tiles_after_hole } => {
                fix_after_tileset_tiles_removed(wc, store, editors, tileset_id, hole_start, hole_size, num_tiles_after_hole);
            }

            EditorAction::TilesetTilesShuffled { tileset_id, shuffle } => {
                fix_after_tileset_tiles_shuffled(wc, store, editors, tileset_id, shuffle);
            }

            EditorAction::SpriteFramesAdded { sprite_id, hole_start, hole_size, num_frames_after_hole } => {
                fix_after_sprite_frames_added(wc, store, editors, sprite_id, hole_start, hole_size, num_frames_after_hole);
            }

            EditorAction::SpriteFramesRemoved { sprite_id, hole_start, hole_size, num_frames_after_hole } => {
                fix_after_sprite_frames_removed(wc, store, editors, sprite_id, hole_start, hole_size, num_frames_after_hole);
            }

            EditorAction::PalSpriteFramesAdded { pal_sprite_id, hole_start, hole_size, num_frames_after_hole } => {
                fix_after_pal_sprite_frames_added(wc, store, editors, pal_sprite_id, hole_start, hole_size, num_frames_after_hole);
            }

            EditorAction::PalSpriteFramesRemoved { pal_sprite_id, hole_start, hole_size, num_frames_after_hole } => {
                fix_after_pal_sprite_frames_removed(wc, store, editors, pal_sprite_id, hole_start, hole_size, num_frames_after_hole);
            }
        }
    }
}
