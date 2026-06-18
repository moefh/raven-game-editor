use crate::app::WindowContext;
use crate::image::ImageCollection;
use crate::data_asset::PalSprite;
use super::super::AssetEditorBase;

pub struct RemoveFramesDialog {
    pub image_changed: bool,
    pub open: bool,
    pub num_frames: u32,
    pub max_frames: u32,
    pub sel_frame: u32,
}

impl RemoveFramesDialog {
    pub fn new() -> Self {
        RemoveFramesDialog {
            image_changed: false,
            open: false,
            num_frames: 0,
            max_frames: 0,
            sel_frame: 0,
        }
    }

    pub fn id() -> egui::Id {
        egui::Id::new("dlg_pal_sprite_remove_frames")
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, pal_sprite: &PalSprite, sel_frame: u32) {
        if pal_sprite.num_frames <= 1 || pal_sprite.num_frames <= sel_frame { return; }
        self.max_frames = (pal_sprite.num_frames - sel_frame).min(pal_sprite.num_frames - 1);
        self.num_frames = 1;
        self.sel_frame = sel_frame;
        self.open = true;
        wc.set_window_open(Self::id(), self.open);
    }

    fn confirm(&mut self, pal_sprite: &mut PalSprite) {
        if self.sel_frame + self.num_frames < pal_sprite.num_frames {
            let src_top = (self.sel_frame + self.num_frames) * pal_sprite.height;
            let dst_top = self.sel_frame * pal_sprite.height;
            let row_len = pal_sprite.width as usize;
            let mut src_row = vec![0; row_len];
            let mut dst_row = vec![0; row_len];
            let num_copy_rows = (pal_sprite.num_frames - (self.sel_frame + self.num_frames)) * pal_sprite.height;
            for y in 0..num_copy_rows {
                let src = ((src_top + y) * pal_sprite.width) as usize;
                let dst = ((dst_top + y) * pal_sprite.width) as usize;
                src_row.copy_from_slice(&pal_sprite.data[src..src+row_len]);
                dst_row.copy_from_slice(&pal_sprite.data[dst..dst+row_len]);
                pal_sprite.data[src..src+row_len].copy_from_slice(&dst_row);
                pal_sprite.data[dst..dst+row_len].copy_from_slice(&src_row);
            }
        }
        pal_sprite.resize(pal_sprite.width, pal_sprite.height, pal_sprite.num_frames - self.num_frames, 0);
        self.image_changed = true;
    }

    pub fn show(&mut self, wc: &mut WindowContext, pal_sprite: &mut PalSprite) -> bool {
        if AssetEditorBase::show_dialog_window(wc, Self::id(), 350.0, "Remove Frames", |ui, _wc| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                egui::Grid::new(format!("editor_panel_{}_add_frames_grid", pal_sprite.asset.id))
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Num frames:");
                        ui.add(egui::Slider::new(&mut self.num_frames, 1..=16.min(self.max_frames)));
                        ui.end_row();
                    });
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if ui.button("Cancel").clicked() {
                    ui.close();
                }
                if ui.button("Ok").clicked() {
                    self.confirm(pal_sprite);
                    ui.close();
                }
            });
        }).should_close() {
            self.open = false;
            wc.set_window_open(Self::id(), self.open);
        }
        if self.image_changed {
            self.image_changed = false;
            true
        } else {
            false
        }
    }
}
