use crate::data_asset::{DataAssetId, Tileset};

pub enum MapLayer {
    Foreground,
    Background,
}

impl MapLayer {
    pub fn name(&self) -> &'static str {
        match self {
            MapLayer::Foreground => "foreground",
            MapLayer::Background => "background",
        }
    }
}

pub enum AssetProblem {
    TilesetTooBig { num_tiles: u32 },
    MapTilesetInvalid { tileset_id: DataAssetId },
    MapTooSmall { width: u32, height: u32 },
    MapBackgroundTooSmall { bg_width: u32, bg_height: u32 },
    MapBackgroundTooBig { width: u32, height: u32, bg_width: u32, bg_height: u32 },
    MapInvalidTile { tile_x: u32, tile_y: u32, tile: u8, layer: MapLayer },
    MapTransparentTile { first_tile_x: u32, first_tile_y: u32, num_tiles: u32 },
    SpriteTooBig { num_frames: u32 },
    RoomWithNoMaps,
    RoomInvalidMapId { map_id: DataAssetId },
    RoomMapInvalidXLocation { x: u32, map_id: DataAssetId },
    RoomMapInvalidYLocation { y: u32, map_id: DataAssetId },
    ModPatternTooSmall { expected: usize, got: usize },
    ModNoteOutOfTune { song_position: u32, row: u32, chan: u8, sharp_by: u16 },
}

impl AssetProblem {
    pub fn log(&self, ui: &mut egui::Ui) {
        match self {
            AssetProblem::TilesetTooBig { num_tiles } => {
                ui.label(format!("  -> tileset has too many tiles: {} (max is 255)", num_tiles));
            }

            AssetProblem::MapTilesetInvalid { tileset_id } => {
                ui.label(format!("  -> map references an invalid tileset: {}", tileset_id));
            }

            AssetProblem::MapTooSmall { width, height } => {
                ui.label(format!(
                    "  -> map is too small: {}x{} (min is {}x{})",
                    width,
                    height,
                    super::map_data::SCREEN_WIDTH.div_ceil(Tileset::TILE_SIZE),
                    super::map_data::SCREEN_HEIGHT.div_ceil(Tileset::TILE_SIZE)
                ));
            }

            AssetProblem::MapBackgroundTooSmall { bg_width, bg_height } => {
                ui.label(format!(
                    "  -> map background is too small: {}x{} (min is {}x{})",
                    bg_width,
                    bg_height,
                    super::map_data::SCREEN_WIDTH.div_ceil(Tileset::TILE_SIZE),
                    super::map_data::SCREEN_HEIGHT.div_ceil(Tileset::TILE_SIZE)
                ));
            }

            AssetProblem::MapBackgroundTooBig { width, height, bg_width, bg_height } => {
                ui.label(format!(
                    "  -> map background is too big: {}x{} (max is map size, which is {}x{})",
                    bg_width,
                    bg_height,
                    width,
                    height
                ));
            }

            AssetProblem::MapInvalidTile { tile_x, tile_y, tile, layer } => {
                ui.label(format!(
                    "  -> map has invalid tile {} at ({}, {}) layer {} ",
                    tile,
                    tile_x,
                    tile_y,
                    layer.name()
                ));
            }

            AssetProblem::MapTransparentTile { first_tile_x, first_tile_y, num_tiles } => {
                ui.label(format!(
                    "  -> map has transparent fg tiles over missing background starting at ({}, {}), total of {} tile(s)",
                    first_tile_x,
                    first_tile_y,
                    num_tiles
                ));
            }

            AssetProblem::SpriteTooBig { num_frames } => {
                ui.label(format!("  -> sprite has too many frames: {} (max is 255)", num_frames));
            }

            AssetProblem::RoomWithNoMaps => {
                ui.label("  -> room has no maps");
            }

            AssetProblem::RoomInvalidMapId { map_id } => {
                ui.label(format!("  -> room with invalid map id: {}", map_id));
            }

            AssetProblem::RoomMapInvalidXLocation { map_id, x } => {
                ui.label(format!(
                    "  -> room map {} extends too far horizontally: {} (max is {})",
                    map_id,
                    x,
                    (i16::MAX as u32 + 1) / Tileset::TILE_SIZE
                ));
            }

            AssetProblem::RoomMapInvalidYLocation { map_id, y } => {
                ui.label(format!(
                    "  -> room map {} extends too far horizontally: {} (max is {})",
                    map_id,
                    y,
                    (i16::MAX as u32 + 1) / Tileset::TILE_SIZE
                ));
            }

            AssetProblem::ModPatternTooSmall { expected, got } => {
                ui.label(format!("  -> MOD pattern has too few cells: expected {}, got {}", expected, got));
            }

            AssetProblem::ModNoteOutOfTune { song_position, row, chan, sharp_by } => {
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
