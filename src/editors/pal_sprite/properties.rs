use crate::app::WindowContext;
use crate::image::ImageCollection;
use crate::data_asset::{PalSprite, PalSpriteDepth};

pub struct PropertiesDialog {
    pub image_changed: bool,
    pub open: bool,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub num_frames: u32,
    pub depth: PalSpriteDepth,
    pub sel_color: u8,
}

impl PropertiesDialog {
    pub fn new() -> Self {
        PropertiesDialog {
            image_changed: false,
            open: false,
            name: String::new(),
            width: 0,
            height: 0,
            num_frames: 0,
            depth: PalSpriteDepth::Bpp1,
            sel_color: 0,
        }
    }

    pub fn id() -> egui::Id {
        egui::Id::new("dlg_pal_sprite_properties")
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, pal_sprite: &PalSprite, sel_color: u8) {
        self.name.clear();
        self.name.push_str(&pal_sprite.asset.name);
        self.width = pal_sprite.width;
        self.height = pal_sprite.height;
        self.num_frames = pal_sprite.num_frames;
        self.depth = pal_sprite.depth;
        self.sel_color = sel_color;
        self.open = true;
        wc.set_window_open(Self::id(), self.open);
    }

    fn confirm(&mut self, pal_sprite: &mut PalSprite) {
        pal_sprite.asset.name.clear();
        pal_sprite.asset.name.push_str(&self.name);

        let width = self.width;
        let height = self.height;
        let num_frames = self.num_frames;
        let depth = self.depth;
        if num_frames != pal_sprite.num_frames || width != pal_sprite.width || height != pal_sprite.height || depth != pal_sprite.depth {
            pal_sprite.resize(width, height, num_frames, self.sel_color);
            pal_sprite.width = width;
            pal_sprite.height = height;
            pal_sprite.num_frames = num_frames;
            pal_sprite.depth = self.depth;
            if (pal_sprite.depth.num_colors() as usize) < pal_sprite.palette.len() {
                let num_colors = pal_sprite.depth.num_colors() as usize;
                for color in pal_sprite.palette[num_colors..].iter_mut() { *color = 0; }
                pal_sprite.recalculate_color_to_palette_index_map();
                pal_sprite.force_palette();
            }
            self.image_changed = true;
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, pal_sprite: &mut PalSprite) -> bool {
        if ! self.open { return false; }

        if egui::Modal::new(Self::id()).show(wc.egui.ctx, |ui| {
            wc.sys_dialogs.block_ui(ui);
            ui.set_width(300.0);
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                ui.heading("Paletted Sprite Properties");
                ui.separator();

                egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                    egui::Grid::new(format!("editor_panel_{}_prop_grid", pal_sprite.asset.id))
                        .num_columns(2)
                        .spacing([8.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(&mut self.name);
                            ui.end_row();

                            ui.label("Width:");
                            ui.add(egui::Slider::new(&mut self.width, 1..=512));
                            ui.end_row();

                            ui.label("Height:");
                            ui.add(egui::Slider::new(&mut self.height, 1..=512));
                            ui.end_row();

                            ui.label("Num frames:");
                            ui.add(egui::Slider::new(&mut self.num_frames, 1..=255));
                            ui.end_row();

                            ui.label("Color Depth:");
                            egui::ComboBox::from_id_salt(format!("editor_{}_depth_combo", pal_sprite.asset.id))
                                .selected_text(format!("{} bpp ({} colors)", self.depth.bits_per_pixel(), self.depth.num_colors()))
                                .width(50.0)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.depth, PalSpriteDepth::Bpp1, "1 bpp (2 colors)");
                                    ui.selectable_value(&mut self.depth, PalSpriteDepth::Bpp2, "2 bpp (4 colors)");
                                    ui.selectable_value(&mut self.depth, PalSpriteDepth::Bpp4, "4 bpp (16 colors)");
                                });
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
