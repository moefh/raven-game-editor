use crate::data_asset::{
    DataAssetId,
    DataAssetStore,
    Tileset,
};

pub enum MapLayer {
    Foreground,
    Background,
    Parallax,
}

impl MapLayer {
    pub fn name(&self) -> &'static str {
        match self {
            MapLayer::Foreground => "foreground",
            MapLayer::Background => "background",
            MapLayer::Parallax   => "parallax",
        }
    }
}

pub enum AssetWarning {
    MapOpaqueTile { first_tile_x: u32, first_tile_y: u32, num_tiles: u32 },
    WorldWithNoRegions,
    WorldRegionWithNoMaps { region_index: usize },
}

impl AssetWarning {
    pub fn log(&self, ui: &mut egui::Ui, asset_id: DataAssetId, store: &DataAssetStore) {
        match self {
            AssetWarning::MapOpaqueTile { first_tile_x, first_tile_y, num_tiles } => {
                ui.label(format!(
                    "  -> map has opaque fg tiles over opaque background starting at ({}, {}), total of {} tile(s)",
                    first_tile_x,
                    first_tile_y,
                    num_tiles
                ));
            }
            AssetWarning::WorldWithNoRegions => {
                ui.label("  -> world has no regions");
            }

            AssetWarning::WorldRegionWithNoMaps { region_index } => {
                if let Some(world) = store.assets.worlds.get(&asset_id) && let Some(region) = world.regions.get(*region_index) {
                    ui.label(format!("  -> region '{}' has no rooms", region.name));
                }
            }
        }
    }
}

pub enum AssetError {
    TilesetTooBig { num_tiles: u32 },
    MapTilesetInvalid { tileset_id: DataAssetId },
    MapParallaxTooSmall { para_width: u32, para_height: u32 },
    MapParallaxTooBig { width: u32, height: u32, para_width: u32, para_height: u32 },
    MapInvalidTile { tile_x: u32, tile_y: u32, tile: u8, layer: MapLayer },
    MapTransparentTile { first_tile_x: u32, first_tile_y: u32, num_tiles: u32 },
    SpriteTooBig { num_frames: u32 },
    PalSpriteTooBig { num_frames: u32 },
    PalSpriteColorOutOfPalette { frame_num: u32, num_pixels: u64 },
    RoomWithNoMaps,
    RoomTooSmall { width: u32, height: u32 },
    RoomInvalidMapId { map_id: DataAssetId },
    RoomMapInvalidXLocation { x: u32, map_id: DataAssetId },
    RoomMapInvalidYLocation { y: u32, map_id: DataAssetId },
    RoomTriggersWithSameId { trigger1_index: usize, trigger2_index: usize, trigger_id: u16 },
    RoomDoorWithInvalidDestination { trigger_index: usize },
    WorldRegionsUsingSameRoom { room_id: DataAssetId, region1_index: usize, region2_index: usize },
    ModPatternTooSmall { expected: usize, got: usize },
    ModNoteOutOfTune { song_position: u32, row: u32, chan: u8, sharp_by: u16 },
}

impl AssetError {
    pub fn log(&self, ui: &mut egui::Ui, asset_id: DataAssetId, store: &DataAssetStore) {
        match self {
            AssetError::TilesetTooBig { num_tiles } => {
                ui.label(format!("  -> tileset has too many tiles: {} (max is 255)", num_tiles));
            }

            AssetError::MapTilesetInvalid { tileset_id } => {
                ui.label(format!("  -> map references an invalid tileset: {}", tileset_id));
            }

            AssetError::MapParallaxTooSmall { para_width, para_height } => {
                ui.label(format!(
                    "  -> map parallax is too small: {}x{} (min is {}x{})",
                    para_width,
                    para_height,
                    super::SCREEN_WIDTH.div_ceil(Tileset::TILE_SIZE),
                    super::SCREEN_HEIGHT.div_ceil(Tileset::TILE_SIZE)
                ));
            }

            AssetError::MapParallaxTooBig { width, height, para_width, para_height } => {
                ui.label(format!(
                    "  -> map parallax is too big: {}x{} (max is map size, which is {}x{})",
                    para_width,
                    para_height,
                    width,
                    height
                ));
            }

            AssetError::MapInvalidTile { tile_x, tile_y, tile, layer } => {
                ui.label(format!(
                    "  -> map has invalid tile {} at ({}, {}) layer {} ",
                    tile,
                    tile_x,
                    tile_y,
                    layer.name()
                ));
            }

            AssetError::MapTransparentTile { first_tile_x, first_tile_y, num_tiles } => {
                ui.label(format!(
                    "  -> map has transparent fg tiles over missing background starting at ({}, {}), total of {} tile(s)",
                    first_tile_x,
                    first_tile_y,
                    num_tiles
                ));
            }

            AssetError::SpriteTooBig { num_frames } => {
                ui.label(format!("  -> sprite has too many frames: {} (max is 255)", num_frames));
            }

            AssetError::PalSpriteTooBig { num_frames } => {
                ui.label(format!("  -> palette sprite has too many frames: {} (max is 255)", num_frames));
            }

            AssetError::PalSpriteColorOutOfPalette { frame_num, num_pixels } => {
                ui.label(format!("  -> palette sprite has colors not in palette (first bad frame: {}, total {} pixels)",
                                 frame_num, num_pixels));
            }

            AssetError::RoomWithNoMaps => {
                ui.label("  -> room has no maps");
            }

            AssetError::RoomTooSmall { width, height } => {
                ui.label(format!(
                    "  -> room is too small: {}x{} (min is {}x{})",
                    width,
                    height,
                    super::SCREEN_WIDTH.div_ceil(Tileset::TILE_SIZE),
                    super::SCREEN_HEIGHT.div_ceil(Tileset::TILE_SIZE)
                ));
            }

            AssetError::RoomInvalidMapId { map_id } => {
                ui.label(format!("  -> room with invalid map id: {}", map_id));
            }

            AssetError::RoomMapInvalidXLocation { map_id, x } => {
                ui.label(format!(
                    "  -> room map {} extends too far horizontally: {} (max is {})",
                    map_id,
                    x,
                    (i16::MAX as u32 + 1) / Tileset::TILE_SIZE
                ));
            }

            AssetError::RoomMapInvalidYLocation { map_id, y } => {
                ui.label(format!(
                    "  -> room map {} extends too far horizontally: {} (max is {})",
                    map_id,
                    y,
                    (i16::MAX as u32 + 1) / Tileset::TILE_SIZE
                ));
            }

            AssetError::RoomTriggersWithSameId { trigger1_index, trigger2_index, trigger_id } => {
                if let Some(room) = store.assets.rooms.get(&asset_id) &&
                    let Some(tr1) = room.triggers.get(*trigger1_index) &&
                    let Some(tr2) = room.triggers.get(*trigger2_index) {
                        ui.label(format!(
                            "  -> triggers '{}' and '{}' have the same id: {}",
                            tr1.name_id,
                            tr2.name_id,
                            trigger_id
                        ));
                    } else {
                        ui.label("*** ERROR: INVALID ROOM TRIGGER REFERENCE");
                        ui.label(format!("*** trigger1_index={}, trigger2_index={}", trigger1_index, trigger2_index));
                    }
            }

            AssetError::RoomDoorWithInvalidDestination { trigger_index } => {
                if let Some(room) = store.assets.rooms.get(&asset_id) &&
                    let Some(door) = room.triggers.get(*trigger_index) {
                        ui.label(format!("  -> door '{}' has invalid destination", door.name_id));
                    }
            }

            AssetError::WorldRegionsUsingSameRoom { room_id, region1_index, region2_index } => {
                if let Some(world) = store.assets.worlds.get(&asset_id) &&
                    let Some(room) = store.assets.rooms.get(room_id) &&
                    let Some(region1) = world.regions.get(*region1_index) &&
                    let Some(region2) = world.regions.get(*region2_index) {
                        ui.label(format!(
                            "  -> room '{}' is used in multiple regions: '{}', '{}'",
                            room.asset.name,
                            region1.name,
                            region2.name,
                        ));
                    } else {
                        ui.label("*** ERROR: INVALID WORLD REGION REFERENCE");
                        ui.label(format!("*** room_id={}, region1_index={}, region2_index={}", room_id, region1_index, region2_index));
                    }
            }

            AssetError::ModPatternTooSmall { expected, got } => {
                ui.label(format!("  -> MOD pattern has too few cells: expected {}, got {}", expected, got));
            }

            AssetError::ModNoteOutOfTune { song_position, row, chan, sharp_by } => {
                if *sharp_by == 0 {
                    ui.label(format!(
                        "  -> MOD note has invalid period: song_position {}, row {}, channel {}",
                        song_position,
                        row,
                        chan
                    ));
                } else {
                    ui.label(format!(
                        "  -> MOD note out of tune by {} periods: song_position {}, row {}, channel {}",
                        sharp_by,
                        song_position,
                        row,
                        chan
                    ));
                }
            }
        }
    }
}
