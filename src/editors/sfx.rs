use crate::IMAGES;
use crate::misc::{WindowContext, SoundPlayer};
use crate::data_asset::{Sfx, DataAssetId, GenericAsset};

struct PropertiesDialog {
    open: bool,
    name: String,
}

impl PropertiesDialog {
    fn new() -> Self {
        PropertiesDialog {
            open: false,
            name: String::new(),
        }
    }

    fn set_open(&mut self, sfx: &Sfx) {
        self.name.clear();
        self.name.push_str(&sfx.asset.name);
        self.open = true;
    }

    fn confirm(&mut self, sfx: &mut Sfx) {
        sfx.asset.name.clear();
        sfx.asset.name.push_str(&self.name);
    }

    fn show(&mut self, wc: &WindowContext, sfx: &mut Sfx) {
        if egui::Modal::new(egui::Id::new("dlg_sfx_properties")).show(wc.egui.ctx, |ui| {
            ui.set_width(250.0);
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                ui.heading("Sfx Properties");
                ui.add_space(16.0);

                egui::Grid::new(format!("editor_panel_{}_prop_grid", sfx.asset.id))
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.name);
                        ui.end_row();
                    });

                ui.add_space(16.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui.button("Cancel").clicked() {
                        ui.close();
                    }
                    if ui.button("Ok").clicked() {
                        self.confirm(sfx);
                        ui.close();
                    }
                });
            });
        }).should_close() {
            self.open = false;
        }
    }
}

pub struct SfxEditor {
    pub asset: super::DataAssetEditor,
    sfx_display_state: super::widgets::SfxDisplayState,
    properties_dialog: PropertiesDialog,
    play_volume: f32,
    play_freq: f32,
}

impl SfxEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        SfxEditor {
            asset: super::DataAssetEditor::new(id, open),
            sfx_display_state: super::widgets::SfxDisplayState::new(),
            properties_dialog: PropertiesDialog::new(),
            play_volume: 0.5,
            play_freq: 11025.0,
        }
    }

    pub fn show(&mut self, wc: &WindowContext, sfx: &mut Sfx, sound_player: &mut SoundPlayer) {
        if self.properties_dialog.open {
            self.properties_dialog.show(wc, sfx);
        }

        let asset_id = sfx.asset.id;
        let title = format!("{} - Sfx", sfx.asset.name);
        let window = super::create_editor_window(asset_id, &title, wc);

        let mut loop_start = sfx.loop_start as f32;
        let mut loop_end = (sfx.loop_start + sfx.loop_len) as f32;

        window.open(&mut self.asset.open).min_size([400.0, 220.0]).default_size([500.0, 220.0]).show(wc.egui.ctx, |ui| {
            // header:
            egui::TopBottomPanel::top(format!("editor_panel_{}_top", asset_id)).show_inside(ui, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("Sfx", |ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::Image::new(IMAGES.properties).max_width(14.0).max_height(14.0));
                            if ui.button("Properties...").clicked() {
                                self.properties_dialog.set_open(sfx);
                            }
                        });
                    });
                });
            });

            // footer:
            egui::TopBottomPanel::bottom(format!("editor_panel_{}_bottom", asset_id)).show_inside(ui, |ui| {
                ui.add_space(5.0);
                ui.label(format!("{} bytes", sfx.data_size()));
            });

            // properties
            egui::SidePanel::left(format!("editor_panel_{}_left", asset_id)).resizable(false).show_inside(ui, |ui| {
                ui.add_space(5.0);
                egui::CollapsingHeader::new("Properties").default_open(true).show(ui, |ui| {
                    egui::Grid::new(format!("editor_{}_loop_grid", asset_id)).num_columns(2).show(ui, |ui| {
                        ui.label("Bits/sample:");
                        egui::ComboBox::from_id_salt(format!("editor_{}_bps_combo", asset_id))
                            .selected_text(format!("{}", sfx.bits_per_sample))
                            .width(50.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut sfx.bits_per_sample, 8, "8");
                                ui.selectable_value(&mut sfx.bits_per_sample, 16, "16");
                            });
                        ui.end_row();

                        ui.label("Loop start:");
                        ui.add(egui::DragValue::new(&mut loop_start).speed(1.0).range(0.0..=sfx.samples.len() as f32));
                        ui.end_row();

                        ui.label("Loop end:");
                        ui.add(egui::DragValue::new(&mut loop_end).speed(1.0).range(loop_start..=sfx.samples.len() as f32));
                        ui.end_row();
                    });
                });
                egui::CollapsingHeader::new("Test").default_open(sound_player.is_available()).show(ui, |ui| {
                    if sound_player.is_available() {
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::new(&mut self.play_freq).speed(25.0).range(8000.0..=44100.0));
                            ui.label("Hz");
                            ui.add_space(5.0);
                            if ui.button("▶ Play ").clicked() {
                                sound_player.play_s16(&sfx.samples, self.play_freq, self.play_volume);
                            }
                        });
                        ui.add(egui::Slider::new(&mut self.play_volume, 0.0..=2.0)).on_hover_ui(|ui| {
                            ui.style_mut().interaction.selectable_labels = true;
                            ui.label("Volume");
                        });
                    } else {
                        ui.label("Sound playback not available :(");
                    }
                });
            });

            // body:
            egui::CentralPanel::default().show_inside(ui, |ui| {
                super::widgets::sfx_display(ui, &mut self.sfx_display_state, &sfx.samples, &mut loop_start, &mut loop_end, 0.0);
            });
        });

        sfx.loop_start = loop_start as u32;
        sfx.loop_len = (loop_end - loop_start) as u32;
    }
}
