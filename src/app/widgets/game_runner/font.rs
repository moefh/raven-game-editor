use crate::data_store::{
    DataStore,
    FontDataAssetId,
    Font,
    PropFont,
};

use super::super::super::{
    WindowContext,
};
use super::super::super::editors::{
    FontPainter,
};

pub enum GameAssetFont<'a> {
    Fixed(&'a Font),
    Prop(&'a PropFont),
}

impl<'a> GameAssetFont<'a> {
    pub fn from_font_asset_id(font_asset_id: &FontDataAssetId, store: &'a DataStore) -> Option<Self> {
        match font_asset_id {
            FontDataAssetId::Fixed(font_id) => { Some(GameAssetFont::Fixed(store.assets.fonts.get(font_id)?)) }
            FontDataAssetId::Prop(font_id) => { Some(GameAssetFont::Prop(store.assets.prop_fonts.get(font_id)?)) }
        }
    }
}

impl<'a> FontPainter for GameAssetFont<'a> {
    fn font_height(&self) -> f32 {
        match self {
            GameAssetFont::Fixed(font) => { font.font_height() }
            GameAssetFont::Prop(font) => { font.font_height() }
        }
    }
    fn measure(&self, height: f32, text: &str) -> f32 {
        match self {
            GameAssetFont::Fixed(font) => { font.measure(height, text) }
            GameAssetFont::Prop(font) => { font.measure(height, text) }
        }
    }
    fn paint_char(&self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        ch: char,
        pos: egui::Pos2,
        height: f32,
        color: egui::Color32
    ) -> f32 {
        match self {
            GameAssetFont::Fixed(font) => { font.paint_char(ui, wc, ch, pos, height, color) }
            GameAssetFont::Prop(font) => { font.paint_char(ui, wc, ch, pos, height, color) }
        }
    }
}
