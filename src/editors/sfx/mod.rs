mod properties;
mod export;

use std::io::Error;
use crate::IMAGES;
use crate::app::{WindowContext, SysDialogResponse};
use crate::sound::SoundPlayer;
use crate::misc::wav_utils;
use crate::data_asset::{Sfx, DataAssetId, GenericAsset};

use properties::PropertiesDialog;
use export::ExportDialog;
use super::DataAssetEditor;
use super::widgets::SfxEditorWidget;

pub struct SfxEditor {
    pub asset: DataAssetEditor,
    editor: Editor,
    dialogs: Dialogs,
}

impl SfxEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        SfxEditor {
            asset: super::DataAssetEditor::new(id, open),
            editor: Editor::new(id),
            dialogs: Dialogs::new(),
        }
    }

    pub fn prepare_for_saving(&mut self, _sfx: &mut Sfx) {
    }

    pub fn show(&mut self, wc: &mut WindowContext, sfx: &mut Sfx, sound_player: &mut SoundPlayer) {
        self.dialogs.show(wc, &mut self.editor, sfx, sound_player);

        let title = format!("{} - Sfx", sfx.asset.name);
        let window = super::DataAssetEditor::create_window(&mut self.asset, wc, &title);
        window.min_size([400.0, 220.0]).default_size([500.0, 220.0]).show(wc.egui.ctx, |ui| {
            self.editor.show(ui, wc, &mut self.dialogs, sfx, sound_player);
        });
    }
}

struct Dialogs {
    properties_dialog: PropertiesDialog,
    export_dialog: ExportDialog,
}

impl Dialogs {
    fn new() -> Self {
        Dialogs {
            properties_dialog: PropertiesDialog::new(),
            export_dialog: ExportDialog::new(),
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, _editor: &mut Editor, sfx: &mut Sfx, _sound_player: &mut SoundPlayer) {
        if self.properties_dialog.open {
            self.properties_dialog.show(wc, sfx);
        }
        if self.export_dialog.open {
            self.export_dialog.show(wc, sfx);
        }
    }
}

struct Editor {
    asset_id: DataAssetId,
    sfx_editor: SfxEditorWidget,
    play_volume: f32,
    play_freq: f32,
}

impl Editor {
    pub fn new(asset_id: DataAssetId) -> Self {
        Editor {
            asset_id,
            sfx_editor: SfxEditorWidget::new(),
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

    pub fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, dialogs: &mut Dialogs, sfx: &mut Sfx, sound_player: &mut SoundPlayer) {
        if let Some(SysDialogResponse::File(filename)) = wc.sys_dialogs.get_response_for(format!("editor_{}", self.asset_id)) {
            self.import_wav(wc, &filename, sfx);
        }

        let mut loop_start = sfx.loop_start as f32;
        let mut loop_end = (sfx.loop_start + sfx.loop_len) as f32;

        // header:
        egui::TopBottomPanel::top(format!("editor_panel_{}_top", self.asset_id)).show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Sfx", |ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.import).max_width(14.0).max_height(14.0));
                        if ui.button("Import...").clicked() {
                            wc.sys_dialogs.open_file(Some(wc.egui.window), format!("editor_{}", self.asset_id),
                                                     "Import WAVE file",
                                                     &[
                                                         ("WAVE files (*.wav)", &["wav"]),
                                                         ("All files (*.*)", &["*"]),
                                                     ]);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.import).max_width(14.0).max_height(14.0));
                        if ui.button("Export...").clicked() {
                            dialogs.export_dialog.set_open(sfx, 22050);
                        }
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.properties).max_width(14.0).max_height(14.0));
                        if ui.button("Properties...").clicked() {
                            dialogs.properties_dialog.set_open(sfx);
                        }
                    });
                });
            });
        });

        // footer:
        egui::TopBottomPanel::bottom(format!("editor_panel_{}_bottom", self.asset_id)).show_inside(ui, |ui| {
            ui.add_space(5.0);
            ui.label(format!("{} bytes", sfx.data_size()));
        });

        // properties
        egui::SidePanel::left(format!("editor_panel_{}_left", self.asset_id)).resizable(false).show_inside(ui, |ui| {
            ui.add_space(5.0);
            egui::CollapsingHeader::new("Sample").default_open(true).show(ui, |ui| {
                egui::Grid::new(format!("editor_{}_loop_grid", self.asset_id)).num_columns(2).show(ui, |ui| {
                    ui.label("Bits/sample:");
                    egui::ComboBox::from_id_salt(format!("editor_{}_bps_combo", self.asset_id))
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
            self.sfx_editor.show(ui, &sfx.samples, &mut loop_start, &mut loop_end, 0.0);
        });

        sfx.loop_start = loop_start.max(0.0) as u32;
        sfx.loop_len = (loop_end - loop_start).max(0.0) as u32;
    }
}
