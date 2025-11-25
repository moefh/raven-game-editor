use crate::app::WindowContext;
use crate::data_asset::{
    SpriteAnimation, Sprite,
    DataAssetId, AssetList, AssetIdList,
};

fn fix_animation_loop_indices(animation: &mut SpriteAnimation, sprite: &Sprite) {
    if sprite.num_frames == 0 { return; }
    for aloop in &mut animation.loops {
        for frame in &mut aloop.frame_indices {
            if frame.head_index.is_some_and(|i| i as u32 >= sprite.num_frames) {
                frame.head_index.replace(sprite.num_frames as u8 - 1);
            }
            if frame.foot_index.is_some_and(|i| i as u32 >= sprite.num_frames) {
                frame.foot_index.replace(sprite.num_frames as u8 - 1);
            }
        }
    }
}

pub struct PropertiesDialog {
    pub open: bool,
    pub sprite_id: DataAssetId,
    pub name: String,
}

impl PropertiesDialog {
    pub fn new(sprite_id: DataAssetId) -> Self {
        PropertiesDialog {
            open: false,
            sprite_id,
            name: String::new(),
        }
    }

    pub fn set_open(&mut self, animation: &SpriteAnimation) {
        self.name.clear();
        self.name.push_str(&animation.asset.name);
        self.sprite_id = animation.sprite_id;
        self.open = true;
    }

    fn confirm(&mut self, animation: &mut SpriteAnimation, sprites: &AssetList<Sprite>) {
        animation.asset.name.clear();
        animation.asset.name.push_str(&self.name);
        if animation.sprite_id != self.sprite_id && let Some(sprite) = sprites.get(&self.sprite_id) {
            animation.sprite_id = self.sprite_id;
            fix_animation_loop_indices(animation, sprite);
        }
    }

    pub fn show(&mut self, wc: &WindowContext, animation: &mut SpriteAnimation,
                sprite_ids: &AssetIdList, sprites: &AssetList<Sprite>) {
        if ! self.open { return; }

        if egui::Modal::new(egui::Id::new("dlg_animation_properties")).show(wc.egui.ctx, |ui| {
            ui.set_width(300.0);
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                ui.heading("Animation Properties");
                ui.separator();

                egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                    egui::Grid::new(format!("editor_panel_{}_prop_grid", animation.asset.id))
                        .num_columns(2)
                        .spacing([8.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(&mut self.name);
                            ui.end_row();

                            ui.label("Sprite:");
                            let cur_sprite_name = if let Some(cur_sprite) = sprites.get(&self.sprite_id) {
                                &cur_sprite.asset.name
                            } else {
                                "??"
                            };
                            egui::ComboBox::from_id_salt(format!("anim_editor_sprite_combo_{}", animation.asset.id))
                                .selected_text(cur_sprite_name)
                                .show_ui(ui, |ui| {
                                    for sprite_id in sprite_ids.iter() {
                                        if let Some(sprite) = sprites.get(sprite_id) {
                                            ui.selectable_value(&mut self.sprite_id, sprite.asset.id, &sprite.asset.name);
                                        }
                                    }
                                });
                            ui.end_row();
                        });
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui.button("Cancel").clicked() {
                        ui.close();
                    }
                    if ui.button("Ok").clicked() {
                        self.confirm(animation, sprites);
                        ui.close();
                    }
                });
            });
        }).should_close() {
            self.open = false;
        }
    }
}
