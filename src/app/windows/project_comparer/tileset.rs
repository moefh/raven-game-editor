use core::fmt::NumBuffer;

use egui_extras::{
    TableBuilder,
    Column,
};

use crate::data_store::{
    DataStore,
    Tileset,
};
use crate::image::{
    TextureNameId,
    DynamicImageId,
    ImageCollection,
    ImageCollectionAltId,
};

use super::{
    WindowContext,
    ProjectComparer,
    ProjectComparerAssetDiffRef,
    DiffState,
};
use super::super::super::project_comparer::{
    ImageDiff,
    TilesetDiff,
};

pub fn show_tilesets(
    ui: &mut egui::Ui,
    _wc: &mut WindowContext,
    comparer: &ProjectComparer,
    asset_ref: Option<ProjectComparerAssetDiffRef>,
    cur_store: &DataStore,
    other_store: &DataStore
) -> DiffState {
    if comparer.tilesets.cur_only.is_empty() && comparer.tilesets.other_only.is_empty() && comparer.tilesets.diffs.is_empty() {
        return DiffState::EMPTY;
    }

    let mut diff_state = DiffState::CHANGED;

    if ! comparer.tilesets.diffs.is_empty() {
        ui.label("changed:");
        for (index, diff) in comparer.tilesets.diffs.iter().enumerate() {
            if let Some(cur) = cur_store.assets.tilesets.get(&diff.cur_id) {
                let selected = asset_ref.is_some_and(|r| r.is_changed(index));
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new(&cur.asset.name).selected(selected).frame_when_inactive(true)).clicked() {
                        diff_state.merge_display_asset(ProjectComparerAssetDiffRef::Changed(index));
                    }
                });
            }
        }
        ui.add_space(8.0);
    }

    if ! comparer.tilesets.cur_only.is_empty() {
        ui.label("only in current:");
        for (index, id) in comparer.tilesets.cur_only.iter().enumerate() {
            if let Some(cur) = cur_store.assets.tilesets.get(id) {
                let selected = asset_ref.is_some_and(|r| r.is_cur_only(index));
                if ui.add(egui::Button::new(&cur.asset.name).selected(selected).frame_when_inactive(true)).clicked() {
                    diff_state.merge_display_asset(ProjectComparerAssetDiffRef::CurOnly(index));
                }
            }
        }
        ui.add_space(8.0);
    }

    if ! comparer.tilesets.other_only.is_empty() {
        ui.label("only in other:");
        for (index, id) in comparer.tilesets.other_only.iter().enumerate() {
            if let Some(other) = other_store.assets.tilesets.get(id) {
                let selected = asset_ref.is_some_and(|r| r.is_other_only(index));
                if ui.add(egui::Button::new(&other.asset.name).selected(selected).frame_when_inactive(true)).clicked() {
                    diff_state.merge_display_asset(ProjectComparerAssetDiffRef::OtherOnly(index));
                }
            }
        }
        ui.add_space(4.0);
    }

    diff_state
}

pub struct TilesetDiffWidget {
    other_texture_name_id: TextureNameId,
    scroll_to_top: bool,
    image_changed: bool,
    zoom: f32,
}

impl TilesetDiffWidget {
    pub fn new() -> Self {
        TilesetDiffWidget {
            other_texture_name_id: TextureNameId::Dynamic(DynamicImageId::gen_id()),
            scroll_to_top: false,
            image_changed: false,
            zoom: 3.0,
        }
    }

    pub fn reset(&mut self) {
        self.scroll_to_top = true;
        self.image_changed = true;
    }

    fn draw_tile(&self, ui: &mut egui::Ui, wc: &mut WindowContext, image: &impl ImageCollection, index: u32) {
        let (_, rect) = ui.allocate_space(egui::Vec2::splat(self.zoom * Tileset::TILE_SIZE as f32 + 2.0));
        let tile_rect = egui::Rect::from_min_max(rect.min + egui::Vec2::splat(1.0), rect.max - egui::Vec2::splat(1.0));
        let uv = image.get_item_uv(index);
        let slot = image.texture_slot(false, false);
        if self.image_changed {
            image.load_texture(wc.tex_man, wc.egui.ctx, slot, true);
        }
        let texture = image.texture(wc.tex_man, wc.egui.ctx, slot);
        egui::Image::from_texture((texture.id(), image.get_item_size())).uv(uv).paint_at(ui, tile_rect);
        let stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(255,255,255));
        ui.painter().rect_stroke(rect, egui::CornerRadius::ZERO, stroke, egui::StrokeKind::Inside);
    }

    fn show_diff(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        diff: &TilesetDiff,
        cur_store: &DataStore,
        other_store: &DataStore
    ) {
        let available_height = ui.available_height();
        let table_cell_size = egui::Vec2::splat(self.zoom * Tileset::TILE_SIZE as f32 + 2.0);
        if let Some(cur_tileset) = cur_store.assets.tilesets.get(&diff.cur_id) &&
            let Some(other_tileset) = other_store.assets.tilesets.get(&diff.other_id) {
                let other_image = ImageCollectionAltId::new(self.other_texture_name_id, other_tileset);
                let old_item_spacing = ui.style().spacing.item_spacing;
                ui.style_mut().spacing.item_spacing = egui::Vec2::ZERO;
                let mut table = TableBuilder::new(ui)
                    .cell_layout(egui::Layout::bottom_up(egui::Align::Center))
                    .column(Column::exact(table_cell_size.x).clip(false))
                    .column(Column::exact(40.0).clip(true))
                    .column(Column::exact(table_cell_size.x).clip(true))
                    .column(Column::exact(40.0).clip(true))
                    .column(Column::exact(table_cell_size.x).clip(false))
                    .column(Column::exact(20.0).clip(true));

                if self.scroll_to_top {
                    self.scroll_to_top = false;
                    //table = table.scroll_to_row(0, Some(egui::Align::TOP));
                    table = table.vertical_scroll_offset(0.0);
                }

                table.auto_shrink(true)
                    .striped(true)
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                    .min_scrolled_height(0.0)
                    .max_scroll_height(available_height)
                    .header(20.0, |mut header| {
                        header.col(|ui| { ui.add(egui::Label::new("current").wrap_mode(egui::TextWrapMode::Extend)); });
                        header.col(|_ui| { });
                        header.col(|ui| { ui.add(egui::Label::new("both").wrap_mode(egui::TextWrapMode::Extend)); });
                        header.col(|_ui| { });
                        header.col(|ui| { ui.add(egui::Label::new("other").wrap_mode(egui::TextWrapMode::Extend)); });
                        header.col(|_ui| { });
                    })
                    .body(|body| {
                        body.rows(table_cell_size.y, diff.tiles_diff.len(), |mut row| {
                            row.set_overline(row.index() == 0);
                            if let Some(tile_diff) = diff.tiles_diff.get(row.index()) {
                                match tile_diff {
                                    ImageDiff::Equal { a, b } => {
                                        let mut buf = NumBuffer::new();
                                        row.col(|_ui| { });
                                        row.col(|ui| { ui.add_space(15.0); ui.label(a.format_into(&mut buf)); });
                                        row.col(|ui| { self.draw_tile(ui, wc, cur_tileset, *a) });
                                        row.col(|ui| { ui.add_space(15.0); ui.label(b.format_into(&mut buf)); });
                                        row.col(|_ui| { });
                                    }
                                    ImageDiff::A { a } => {
                                        let mut buf = NumBuffer::new();
                                        row.col(|ui| { self.draw_tile(ui, wc, cur_tileset, *a) });
                                        row.col(|ui| { ui.add_space(15.0); ui.label(a.format_into(&mut buf)); });
                                        row.col(|_ui| { });
                                        row.col(|ui| { ui.add_space(15.0); ui.label("--"); });
                                        row.col(|_ui| { });
                                    }
                                    ImageDiff::B { b } => {
                                        let mut buf = NumBuffer::new();
                                        row.col(|_ui| { });
                                        row.col(|ui| { ui.add_space(15.0); ui.label("--"); });
                                        row.col(|_ui| { });
                                        row.col(|ui| { ui.add_space(15.0); ui.label(b.format_into(&mut buf)); });
                                        row.col(|ui| { self.draw_tile(ui, wc, &other_image, *b) });
                                    }
                                }
                            } else {
                                row.col(|ui| { ui.label("?"); });
                                row.col(|ui| { ui.label("?"); });
                                row.col(|ui| { ui.label("?"); });
                                row.col(|ui| { ui.label("?"); });
                                row.col(|ui| { ui.label("?"); });
                            }
                            row.col(|_ui| {});
                        });
                    });
                ui.style_mut().spacing.item_spacing = old_item_spacing;
                ui.add_space(10.0);
            }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        comparer: &ProjectComparer,
        asset_ref: ProjectComparerAssetDiffRef,
        cur_store: &DataStore,
        other_store: &DataStore
    ) -> DiffState {
        ui.add_space(4.0);
        match asset_ref {
            ProjectComparerAssetDiffRef::Changed(index) => {
                if let Some(diff) = comparer.tilesets.diffs.get(index) &&
                    let Some(cur_tileset) = cur_store.assets.tilesets.get(&diff.cur_id) &&
                    let Some(other_tileset) = other_store.assets.tilesets.get(&diff.other_id) {
                        ui.label(format!("current: {} tiles", cur_tileset.num_tiles));
                        ui.label(format!("other:   {} tiles", other_tileset.num_tiles));
                        ui.add_space(8.0);
                        self.show_diff(ui, wc, diff, cur_store, other_store);
                    }
            }
            ProjectComparerAssetDiffRef::CurOnly(index) => {
                if let Some(cur_id) = comparer.tilesets.cur_only.get(index) &&
                    let Some(tileset) = cur_store.assets.tilesets.get(cur_id) {
                        ui.label(format!("{} tiles", tileset.num_tiles));
                    }
            }
            ProjectComparerAssetDiffRef::OtherOnly(index) => {
                if let Some(other_id) = comparer.tilesets.other_only.get(index) &&
                    let Some(tileset) = other_store.assets.tilesets.get(other_id) {
                        ui.label(format!("{} tiles", tileset.num_tiles));
                    }
            }
        }
        self.image_changed = false;
        DiffState::EMPTY
    }
}
