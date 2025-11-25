use crate::app::WindowContext;
use crate::image::ImageCollection;
use crate::data_asset::Sprite;

pub struct PropertiesDialog {
    pub image_changed: bool,
    pub open: bool,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub num_frames: u32,
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
            sel_color: 0,
        }
    }

    pub fn set_open(&mut self, sprite: &Sprite, sel_color: u8) {
        self.name.clear();
        self.name.push_str(&sprite.asset.name);
        self.width = sprite.width;
        self.height = sprite.height;
        self.num_frames = sprite.num_frames;
        self.sel_color = sel_color;
        self.open = true;
    }

    fn confirm(&mut self, sprite: &mut Sprite) {
        sprite.asset.name.clear();
        sprite.asset.name.push_str(&self.name);

        let width = self.width;
        let height = self.height;
        let num_frames = self.num_frames;
        if num_frames != sprite.num_frames || width != sprite.width || height != sprite.height {
            let image = ImageCollection::from_asset(sprite);
            image.resize(width, height, num_frames, &mut sprite.data, self.sel_color);
            sprite.width = width;
            sprite.height = height;
            sprite.num_frames = num_frames;
            self.image_changed = true;
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, sprite: &mut Sprite) -> bool {
        if ! self.open { return false; }

        if egui::Modal::new(egui::Id::new("dlg_sprite_properties")).show(wc.egui.ctx, |ui| {
            ui.set_width(300.0);
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                ui.heading("Sprite Properties");
                ui.separator();

                egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                    egui::Grid::new(format!("editor_panel_{}_prop_grid", sprite.asset.id))
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
        }
        if self.image_changed {
            self.image_changed = false;
            true
        } else {
            false
        }
    }
}
