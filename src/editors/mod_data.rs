use crate::misc::IMAGES;
use crate::misc::{WindowContext, SoundPlayer};
use crate::data_asset::{ModData, DataAssetId, GenericAsset};

use super::widgets::SfxDisplayState;

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

    fn set_open(&mut self, mod_data: &ModData) {
        self.name.clear();
        self.name.push_str(&mod_data.asset.name);
        self.open = true;
    }

    fn confirm(&mut self, mod_data: &mut ModData) {
        mod_data.asset.name.clear();
        mod_data.asset.name.push_str(&self.name);
    }

    fn show(&mut self, wc: &WindowContext, mod_data: &mut ModData) {
        if egui::Modal::new(egui::Id::new("dlg_about")).show(wc.egui.ctx, |ui| {
            ui.set_width(250.0);
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                ui.heading("MOD Properties");
                ui.add_space(16.0);

                egui::Grid::new(format!("editor_panel_{}_prop_grid", mod_data.asset.id))
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
                        self.confirm(mod_data);
                        ui.close();
                    }
                });
            });
        }).should_close() {
            self.open = false;
        }
    }
}

enum EditorTabs {
    Samples,
    Patterns,
}

pub struct ModDataEditor {
    pub asset: super::DataAssetEditor,
    properties_dialog: PropertiesDialog,
    selected_tab: EditorTabs,
    selected_sample: usize,
    sfx_display_state: SfxDisplayState,
    play_volume: f32,
    play_freq: f32,
}

impl ModDataEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        ModDataEditor {
            asset: super::DataAssetEditor::new(id, open),
            properties_dialog: PropertiesDialog::new(),
            selected_tab: EditorTabs::Samples,
            selected_sample: 0,
            sfx_display_state: SfxDisplayState::new(),
            play_volume: 0.3,
            play_freq: 11025.0,
        }
    }

    fn samples_tab(&mut self, ui: &mut egui::Ui, _wc: &WindowContext, mod_data: &mut ModData, sound_player: &mut SoundPlayer) {
        let asset_id = mod_data.asset.id;
        egui::SidePanel::left(format!("editor_panel_{}_samples_left", asset_id)).resizable(false).show_inside(ui, |ui| {
            let mut sample_name = String::new();
            egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                for (sample_index, sample) in mod_data.samples.iter().enumerate() {
                    sample_name.clear();
                    if sample.len == 0 {
                        sample_name.push_str(&format!("sample {} (empty)", sample_index + 1));
                    } else {
                        sample_name.push_str(&format!("sample {}", sample_index + 1));
                    };
                    if ui.selectable_label(self.selected_sample == sample_index, &sample_name).clicked() {
                        self.selected_sample = sample_index;
                        self.sfx_display_state = SfxDisplayState::new();
                    }
                }
            });
        });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            if let Some(sample) = mod_data.samples.get_mut(self.selected_sample) {
                let samples = if let Some(data) = &sample.data { &data[..] } else { &[] };
                let mut loop_start = sample.loop_start as f32;
                let mut loop_end = (sample.loop_start + sample.loop_len) as f32;
                super::widgets::sfx_display(ui, &mut self.sfx_display_state, samples, &mut loop_start, &mut loop_end, 150.0);
                sample.loop_start = loop_start as u32;
                sample.loop_len = (loop_end - loop_start) as u32;

                if sound_player.is_available() {
                    ui.add_space(5.0);
                    egui::Grid::new(format!("editor_panel_{}_play_grid", asset_id)).num_columns(2).show(ui, |ui| {
                        ui.label("Volume:");
                        ui.add(egui::Slider::new(&mut self.play_volume, 0.0..=1.0));
                        ui.end_row();

                        ui.label("Frequency:");
                        ui.add(egui::Slider::new(&mut self.play_freq, 8000.0..=44100.0));
                        ui.end_row();

                        ui.horizontal(|_ui| {});
                        if ui.button("▶ Play ").clicked() {
                            sound_player.play_s16(samples, self.play_freq, self.play_volume);
                        }
                        ui.end_row();
                    });
                }
            }
        });
    }

    fn patterns_tab(&mut self, ui: &mut egui::Ui, _wc: &WindowContext, mod_data: &mut ModData, _sound_player: &mut SoundPlayer) {
        let _asset_id = mod_data.asset.id;
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.add(egui::Image::new(IMAGES.mod_data).max_width(32.0));
            ui.label("TODO: PATTERNS");
        });
    }

    pub fn show(&mut self, wc: &WindowContext, mod_data: &mut ModData, sound_player: &mut SoundPlayer) {
        if self.properties_dialog.open {
            self.properties_dialog.show(wc, mod_data);
        }

        let asset_id = mod_data.asset.id;
        let title = format!("{} - MOD", mod_data.asset.name);
        let window = super::create_editor_window(asset_id, &title, wc);
        let mut asset_open = self.asset.open;
        window.open(&mut asset_open).min_size([400.0, 220.0]).default_size([600.0, 300.0]).show(wc.egui.ctx, |ui| {
            // header:
            egui::TopBottomPanel::top(format!("editor_panel_{}_top", asset_id)).show_inside(ui, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("MOD", |ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::Image::new(IMAGES.properties).max_width(14.0).max_height(14.0));
                            if ui.button("Properties...").clicked() {
                                self.properties_dialog.set_open(mod_data);
                            }
                        });
                    });
                });
            });

            // footer:
            egui::TopBottomPanel::bottom(format!("editor_panel_{}_bottom", asset_id)).show_inside(ui, |ui| {
                ui.add_space(5.0);
                ui.label(format!("{} bytes", mod_data.data_size()));
            });

            // tabs:
            egui::TopBottomPanel::top(format!("editor_panel_{}_tabs", asset_id)).show_inside(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    if ui.selectable_label(matches!(self.selected_tab, EditorTabs::Samples), "Samples").clicked() {
                        self.selected_tab = EditorTabs::Samples;
                    }
                    if ui.selectable_label(matches!(self.selected_tab, EditorTabs::Patterns), "Patterns").clicked() {
                        self.selected_tab = EditorTabs::Patterns;
                    }
                });
            });

            match self.selected_tab {
                EditorTabs::Samples => self.samples_tab(ui, wc, mod_data, sound_player),
                EditorTabs::Patterns => self.patterns_tab(ui, wc, mod_data, sound_player),
            };
        });
        self.asset.open = asset_open;
    }
}
