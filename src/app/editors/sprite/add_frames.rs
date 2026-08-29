use crate::image::ImageCollection;
use crate::data_asset::Sprite;

use super::super::{
    AssetEditorBase,
    WindowContext,
    EditorAction,
    AddImageLocation,
};

pub struct AddFramesDialog {
    pub open: bool,
    pub add_frame_location: AddImageLocation,
    pub num_frames: u32,
    pub sel_frame: u32,
    pub clear_color: u8,
    dlg_id_grid: String,
    dlg_id_insert_location_combo: String,
    confirmed: bool,
}

impl AddFramesDialog {
    pub fn new() -> Self {
        AddFramesDialog {
            dlg_id_grid: String::from("editor_panel_sprite_add_frame_grid"),
            dlg_id_insert_location_combo: String::from("editor_panel_sprite_add_frame_insert_at_combo"),
            confirmed: false,
            open: false,
            add_frame_location: AddImageLocation::AfterSelected,
            num_frames: 0,
            sel_frame: 0,
            clear_color: 0,
        }
    }

    pub fn id() -> egui::Id {
        egui::Id::new("dlg_sprite_add_frames")
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, sel_frame: u32, clear_color: u8) {
        self.add_frame_location = AddImageLocation::AfterSelected;
        self.num_frames = 1;
        self.sel_frame = sel_frame;
        self.clear_color = clear_color;
        self.open = true;
        wc.set_dialog_open(Self::id(), self.open);
    }

    fn confirm(&mut self, sprite: &mut Sprite, wc: &mut WindowContext) {
        let old_num_frames = sprite.num_frames;
        let insertion_point = match self.add_frame_location {
            AddImageLocation::BeforeSelected => { self.sel_frame.min(sprite.num_frames) }
            AddImageLocation::AfterSelected => { (self.sel_frame + 1).min(sprite.num_frames) }
            AddImageLocation::AtEnd => { sprite.num_frames }
        };
        sprite.resize(sprite.width, sprite.height, sprite.num_frames + self.num_frames, self.clear_color);
        let frame_size = (sprite.width * sprite.height) as usize;
        let src_start = insertion_point as usize * frame_size;
        let src_end = (sprite.num_frames - self.num_frames) as usize * frame_size;
        let dst_start = (insertion_point + self.num_frames) as usize * frame_size;
        sprite.data.copy_within(src_start..src_end, dst_start);
        sprite.data[src_start..dst_start].fill(self.clear_color);
        let num_frames_after_hole = old_num_frames - insertion_point;
        wc.add_editor_action(EditorAction::SpriteFramesAdded {
            sprite_id: sprite.asset.id,
            hole_start: insertion_point,
            hole_size: self.num_frames,
            num_frames_after_hole,
        });
        self.confirmed = true;
    }

    pub fn show(&mut self, wc: &mut WindowContext, sprite: &mut Sprite) -> bool {
        if AssetEditorBase::show_dialog_window(wc, Self::id(), 350.0, "Add frames", |ui, wc| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                egui::Grid::new(&self.dlg_id_grid)
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Num frames:");
                        ui.add(egui::Slider::new(&mut self.num_frames, 1..=16));
                        ui.end_row();

                        ui.label("Insert at:");
                        egui::ComboBox::from_id_salt(&self.dlg_id_insert_location_combo)
                            .selected_text(self.add_frame_location.text())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.add_frame_location,
                                    AddImageLocation::BeforeSelected,
                                    AddImageLocation::BeforeSelected.text()
                                );
                                ui.selectable_value(
                                    &mut self.add_frame_location,
                                    AddImageLocation::AfterSelected,
                                    AddImageLocation::AfterSelected.text()
                                );
                                ui.selectable_value(
                                    &mut self.add_frame_location,
                                    AddImageLocation::AtEnd,
                                    AddImageLocation::AtEnd.text()
                                );
                            });
                        ui.end_row();
                    });
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if ui.button("Cancel").clicked() {
                    ui.close();
                }
                if ui.button("Ok").clicked() {
                    self.confirm(sprite, wc);
                    ui.close();
                }
            });
        }).should_close() {
            self.open = false;
            wc.set_dialog_open(Self::id(), self.open);
        }
        if self.confirmed {
            self.confirmed = false;
            true
        } else {
            false
        }
    }
}
