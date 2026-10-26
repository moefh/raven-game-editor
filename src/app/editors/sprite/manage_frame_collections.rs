use crate::misc::IMAGES;
use crate::image::ImageCollection;
use crate::data_store::{
    metadata,
    Sprite,
};

use super::super::{
    AssetEditorBase,
    WindowContext,
};

enum ColsAction {
    None,
    Add,
    Select(usize),
    Remove(usize),
    MoveUp(usize),
    MoveDown(usize),
}

enum ColAction {
    None,
    Add(u32),
    Remove(u32),
}

pub struct ManageFrameCollectionsDialog {
    dlg_id: egui::Id,
    pub open: bool,
    pub confirmed: bool,
    pub scroll_top: bool,
    pub frame_zoom: f32,
    pub frames: Vec<metadata::SpriteFrameCollection>,
    pub sel_frame: Option<usize>,
}

impl ManageFrameCollectionsDialog {
    pub fn new() -> Self {
        ManageFrameCollectionsDialog {
            dlg_id: egui::Id::unique("dlg_sprite_manage_frame_collections"),
            open: false,
            confirmed: false,
            scroll_top: false,
            frame_zoom: 1.0,
            frames: Vec::new(),
            sel_frame: None,
        }
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, sprite: &Sprite) {
        self.frames.clear();
        if let Some(sprite_md) = wc.store_metadata.get_sprite(&sprite.asset.id) {
            self.frames.append(&mut sprite_md.frames.clone());
        }
        self.set_sel_frame(if self.frames.is_empty() { None } else { Some(0) });
        self.frame_zoom = 80.0 / sprite.width as f32;
        self.confirmed = false;
        self.open = true;
        wc.set_dialog_open(self.dlg_id, self.open);
    }

    fn set_sel_frame(&mut self, sel_frame: Option<usize>) {
        self.sel_frame = sel_frame;
        self.scroll_top = true;
    }

    fn confirm(&mut self, wc: &mut WindowContext, sprite: &Sprite) {
        let sprite_md = wc.store_metadata.get_sprite_mut(&sprite.asset.id);
        sprite_md.frames.clear();
        sprite_md.frames.append(&mut self.frames.clone());
        self.confirmed = true;
    }

    fn show_frame_collections(&mut self, ui: &mut egui::Ui) -> ColsAction {
        let mut action = ColsAction::None;
        for (index, col) in self.frames.iter().enumerate() {
            let can_move_up = index > 0;
            let can_move_down = index < self.frames.len().saturating_sub(1);
            ui.horizontal(|ui| {
                if ui.add(egui::Button::image(IMAGES.trash)).on_hover_text("Remove").clicked() {
                    action = ColsAction::Remove(index);
                }
                if ui.add_enabled(can_move_up, egui::Button::image(IMAGES.arrow_up)).on_hover_text("Move Up").clicked() {
                    action = ColsAction::MoveUp(index);
                }
                if ui.add_enabled(can_move_down, egui::Button::image(IMAGES.arrow_down)).on_hover_text("Move Down").clicked() {
                    action = ColsAction::MoveDown(index);
                }
                if ui.add(
                    egui::Button::new(&col.name)
                        .selected(self.sel_frame.is_some_and(|i| i == index))
                        .frame_when_inactive(self.sel_frame.is_some_and(|i| i == index))
                        .truncate()
                ).clicked() {
                    action = ColsAction::Select(index);
                }
            });
        }
        ui.add_space(5.0);
        if ui.add(egui::Button::new("Create").truncate()).clicked() {
            action = ColsAction::Add;
        }
        action
    }

    fn draw_frame(&self, ui: &mut egui::Ui, wc: &mut WindowContext, image: &impl ImageCollection, index: u32, selected: bool) -> bool {
        let image_size = self.frame_zoom * image.get_item_size();
        let (rect, resp) = ui.allocate_at_least(image_size + egui::Vec2::new(20.0, 0.0), egui::Sense::CLICK);
        let image_rect = egui::Rect::from_min_size(rect.min, image_size);
        let uv = image.get_item_uv(index);
        let slot = image.texture_slot(false, false);
        let texture = image.texture(wc.tex_man, wc.egui.ctx, slot);
        let image = egui::Image::from_texture((texture.id(), image.get_item_size())).uv(uv);
        let image = if selected { image } else { image.tint(egui::Color32::from_rgb(128,128,128)) };
        image.paint_at(ui, image_rect);
        let stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(255,255,255));
        ui.painter().rect_stroke(image_rect, egui::CornerRadius::ZERO, stroke, egui::StrokeKind::Inside);
        resp.clicked()
    }

    pub fn show(&mut self, wc: &mut WindowContext, sprite: &Sprite) -> bool {
        if ! self.open { return false; }

        if AssetEditorBase::show_dialog_window(wc, self.dlg_id, 600.0, "Sprite Frame Collections", |ui, wc| {
            egui::Panel::bottom("dlg_sprite_manage_frame_collections_buttons").show(ui, |ui| {
                ui.add_space(5.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui.button("Cancel").clicked() {
                        ui.close();
                    }
                    if ui.button("Ok").clicked() {
                        self.confirm(wc, sprite);
                        ui.close();
                    }
                });
            });
            egui::Panel::left("dlg_sprite_manage_frame_collections_list")
                .resizable(false)
                .exact_size(260.0)
                .show(ui, |ui| {
                    ui.add_space(5.0);
                    ui.label("Frame Collections:");
                    ui.add_space(2.0);
                    egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                        match self.show_frame_collections(ui) {
                            ColsAction::Select(index) => { self.set_sel_frame(Some(index)); }
                            ColsAction::MoveUp(index) => { if index > 0 { self.frames.swap(index-1, index); } }
                            ColsAction::MoveDown(index) => { if index+1 < self.frames.len() { self.frames.swap(index, index+1); } }
                            ColsAction::Remove(index) => {
                                if index < self.frames.len() {
                                    self.frames.remove(index);
                                    if self.frames.is_empty() {
                                        self.set_sel_frame(None);
                                    } else if self.sel_frame.is_some_and(|i| i >= index) {
                                        self.set_sel_frame(Some(index.saturating_sub(1)));
                                    }
                                }
                            }
                            ColsAction::Add => {
                                let new_index = self.frames.len();
                                self.frames.push(metadata::SpriteFrameCollection::new("new_collection"));
                                self.set_sel_frame(Some(new_index));
                            }
                            ColsAction::None => {}
                        }
                    });
                });

            egui::CentralPanel::default().show(ui, |ui| {
                if let Some(col) = self.sel_frame.and_then(|i| self.frames.get_mut(i)) {
                    ui.horizontal(|ui| {
                        ui.label("Name: ");
                        ui.text_edit_singleline(&mut col.name);
                    });
                }
                ui.add_space(5.0);
                ui.label("Frames:");
                let mut action = ColAction::None;
                let mut scroll = egui::ScrollArea::both();
                if self.scroll_top {
                    self.scroll_top = false;
                    scroll = scroll.scroll_offset(egui::Vec2::ZERO);
                }
                scroll.auto_shrink([true, true]).show(ui, |ui| {
                    if let Some(col) = self.sel_frame.and_then(|i| self.frames.get(i)) {
                        for frame_num in 0..sprite.num_frames {
                            ui.horizontal(|ui| {
                                let selected = col.indices.contains(&frame_num);
                                if self.draw_frame(ui, wc, sprite, frame_num, selected) {
                                    if selected {
                                        action = ColAction::Remove(frame_num);
                                    } else {
                                        action = ColAction::Add(frame_num);
                                    }
                                }
                            });
                        }
                    }
                });
                match action {
                    ColAction::Remove(frame_num) => {
                        if let Some(col) = self.sel_frame.and_then(|i| self.frames.get_mut(i)) {
                            col.indices.retain(|&num| num != frame_num);
                        }
                    }
                    ColAction::Add(frame_num) => {
                        if let Some(col) = self.sel_frame.and_then(|i| self.frames.get_mut(i)) {
                            col.indices.push(frame_num);
                            col.indices.sort();
                        }
                    }
                    ColAction::None => {}
                }
            });
        }).should_close() {
            self.open = false;
            wc.set_dialog_open(self.dlg_id, self.open);
            return self.confirmed;
        }
        false
    }
}
