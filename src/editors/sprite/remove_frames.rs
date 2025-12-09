use crate::app::WindowContext;
use crate::image::ImageCollection;
use crate::data_asset::Sprite;

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
        egui::Id::new("dlg_sprite_remove_frames")
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, sprite: &Sprite, sel_frame: u32) {
        if sprite.num_frames <= 1 || sprite.num_frames <= sel_frame { return; }
        self.max_frames = (sprite.num_frames - sel_frame).min(sprite.num_frames - 1);
        self.num_frames = 1;
        self.sel_frame = sel_frame;
        self.open = true;
        wc.set_window_open(Self::id(), self.open);
    }

    fn confirm(&mut self, sprite: &mut Sprite) {
        if self.sel_frame + self.num_frames < sprite.num_frames {
            let src_top = (self.sel_frame + self.num_frames) * sprite.height;
            let dst_top = self.sel_frame * sprite.height;
            let row_len = sprite.width as usize;
            let mut src_row = vec![0; row_len];
            let mut dst_row = vec![0; row_len];
            let num_copy_rows = (sprite.num_frames - (self.sel_frame + self.num_frames)) * sprite.height;
            for y in 0..num_copy_rows {
                let src = ((src_top + y) * sprite.width) as usize;
                let dst = ((dst_top + y) * sprite.width) as usize;
                src_row.copy_from_slice(&sprite.data[src..src+row_len]);
                dst_row.copy_from_slice(&sprite.data[dst..dst+row_len]);
                sprite.data[src..src+row_len].copy_from_slice(&dst_row);
                sprite.data[dst..dst+row_len].copy_from_slice(&src_row);
            }
        }
        sprite.resize(sprite.width, sprite.height, sprite.num_frames - self.num_frames, 0);
        sprite.num_frames -= self.num_frames;
        self.image_changed = true;
    }

    pub fn show(&mut self, wc: &mut WindowContext, sprite: &mut Sprite) -> bool {
        if egui::Modal::new(Self::id()).show(wc.egui.ctx, |ui| {
            wc.sys_dialogs.block_ui(ui);
            ui.set_width(300.0);
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                ui.heading("Remove Frames");
                ui.separator();

                egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                    egui::Grid::new(format!("editor_panel_{}_add_frames_grid", sprite.asset.id))
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
                        self.confirm(sprite);
                        ui.close();
                    }
                });
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
