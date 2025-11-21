mod properties;

use std::io::Error;
use crate::IMAGES;
use crate::app::{WindowContext, SysDialogResponse};
use crate::sound::SoundPlayer;
use crate::misc::wav_utils;
use crate::data_asset::{Sfx, DataAssetId, GenericAsset};

use properties::PropertiesDialog;

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

    fn import_wav(&mut self, wc: &mut WindowContext, filename: &std::path::Path, sfx: &mut Sfx) {
        let result = wav_utils::WavFile::read(filename).and_then(|mut wav_file| {
            if wav_file.channels.is_empty() { return Err(Error::other("WAV with no channels!?")); }
            let samples = wav_file.channels.remove(0);
            sfx.len = samples.len() as u32;
            sfx.samples = samples;
            sfx.bits_per_sample = wav_file.bits_per_sample;
            sfx.loop_start = 0;
            sfx.loop_len = 0;
            Ok(())
        });

        if let Err(e) = result {
            wc.logger.log(format!("ERROR reading WAV file from {}:", filename.display()));
            wc.logger.log(format!("{}", e));
            wc.dialogs.open_message_box("Error importing Sfx", "Error importing WAV file.\n\nConsult the log window for more information.");
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, sfx: &mut Sfx, sound_player: &mut SoundPlayer) {
        let asset_id = sfx.asset.id;
        if let Some(SysDialogResponse::File(filename)) = wc.sys_dialogs.get_response_for(format!("editor_{}", asset_id)) {
            self.import_wav(wc, &filename, sfx);
        }
        if self.properties_dialog.open {
            self.properties_dialog.show(wc, sfx);
        }

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
                            ui.add(egui::Image::new(IMAGES.import).max_width(14.0).max_height(14.0));
                            if ui.button("Import...").clicked() {
                                wc.sys_dialogs.open_file(format!("editor_{}", asset_id),
                                                         "Import WAVE file",
                                                         &[
                                                             ("WAVE files (*.wav)", &["wav"]),
                                                             ("All files (*.*)", &["*"]),
                                                         ]);
                            }
                        });
                        ui.separator();
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
                egui::CollapsingHeader::new("Sample").default_open(true).show(ui, |ui| {
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

                        ui.label("Length:");
                        ui.label(format!("{}", sfx.len));
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

        sfx.loop_start = loop_start.max(0.0) as u32;
        sfx.loop_len = (loop_end - loop_start).max(0.0) as u32;
    }
}
