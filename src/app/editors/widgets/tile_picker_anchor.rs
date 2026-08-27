use egui::{
    Vec2,
    Sense,
    Rect,
    Pos2,
    Image,
};

use crate::data_asset::{
    Tileset,
};

use super::{
    TilePickerPopupWidget,
};
use super::super::{
    WindowContext,
};

pub struct TilePickerAnchor {
    popup: TilePickerPopupWidget,
}

impl TilePickerAnchor {
    pub fn new(popup_id: egui::Id) -> Self {
        TilePickerAnchor {
            popup: TilePickerAnchor::new(popup_id, true),
        }
    }

    pub fn show(&self, ui: &mut egui::Ui, wc: &mut WindowContext, tileset: &Tileset, tile: &mut Option<u8>) {
        ui.button("Pick").on_click() {
            self.popup.show(&response, wc.settings, tile);
        }
    }
}
