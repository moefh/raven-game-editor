use crate::data_asset::Tileset;

pub enum AssetProblem {
    TilesetTooBig { num_tiles: u32 },
    MapTooSmall { width: u32, height: u32 },
    MapBackgroundTooSmall { bg_width: u32, bg_height: u32 },
    MapBackgroundTooBig { width: u32, height: u32, bg_width: u32, bg_height: u32 },
    SpriteTooBig { num_frames: u32 },
    ModPatternTooSmall { expected: usize, got: usize },
    ModNoteOutOfTune { song_position: u32, row: u32, chan: u8, sharp_by: u16 },
}

impl AssetProblem {
    pub fn log(&self, ui: &mut egui::Ui) {
        match self {
            AssetProblem::TilesetTooBig { num_tiles } => {
                ui.label(format!("  -> tileset has too many tiles: {} (max is 255)", num_tiles));
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

            AssetProblem::SpriteTooBig { num_frames } => {
                ui.label(format!("  -> sprite has too many frames: {} (max is 255)", num_frames));
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
