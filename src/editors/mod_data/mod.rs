mod properties;
mod export_sample;

use std::io::Error;
use egui_extras::{TableBuilder, Column};

use crate::app::{WindowContext, SysDialogResponse};
use crate::misc::{IMAGES, mod_utils, wav_utils};
use crate::sound::SoundPlayer;
use crate::data_asset::{ModData, DataAssetId, GenericAsset};

use properties::PropertiesDialog;
use export_sample::ExportSampleDialog;
use super::widgets::SfxEditorWidget;

const MOD_PATTERN_CELL_NAMES: &[&str] = &[ "note", "spl", "fx" ];

enum EditorTabs {
    Samples,
    Patterns,
}

pub struct ModDataEditor {
    pub asset: super::DataAssetEditor,
    editor: Editor,
    dialogs: Dialogs,
}

impl ModDataEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        ModDataEditor {
            asset: super::DataAssetEditor::new(id, open),
            editor: Editor::new(id),
            dialogs: Dialogs::new(),
        }
    }

    pub fn prepare_for_saving(&mut self, _mod_data: &mut ModData) {
    }

    pub fn show(&mut self, wc: &mut WindowContext, mod_data: &mut ModData, sound_player: &mut SoundPlayer) {
        self.dialogs.show(wc, &mut self.editor, mod_data);

        let title = format!("{} - MOD", mod_data.asset.name);
        let window = super::DataAssetEditor::create_window(&mut self.asset, wc, &title);
        window.min_size([600.0, 300.0]).default_size([600.0, 300.0]).show(wc.egui.ctx, |ui| {
            self.editor.show(ui, wc, &mut self.dialogs, mod_data, sound_player);
        });
    }
}

struct Dialogs {
    properties_dialog: PropertiesDialog,
    export_sample_dialog: ExportSampleDialog,
}

impl Dialogs {
    fn new() -> Self {
        Dialogs {
            properties_dialog: PropertiesDialog::new(),
            export_sample_dialog: ExportSampleDialog::new(),
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, _editor: &mut Editor, mod_data: &mut ModData) {
        if self.properties_dialog.open {
            self.properties_dialog.show(wc, mod_data);
        }
        if self.export_sample_dialog.open {
            self.export_sample_dialog.show(wc, mod_data);
        }
    }
}

struct Editor {
    asset_id: DataAssetId,
    selected_tab: EditorTabs,
    selected_sample: usize,
    selected_song_position: usize,
    sfx_editor: SfxEditorWidget,
    play_volume: f32,
    play_freq: f32,
}

impl Editor {
    pub fn new(asset_id: DataAssetId) -> Self {
        Editor {
            asset_id,
            selected_tab: EditorTabs::Samples,
            selected_sample: 0,
            selected_song_position: 0,
            sfx_editor: SfxEditorWidget::new(),
            play_volume: 0.5,
            play_freq: 11025.0,
        }
    }

    fn get_pattern_sample_to_play(cell_index: usize, mod_data: &ModData) -> Option<usize> {
        let row_stride = mod_data.num_channels as usize;
        let cell_row = cell_index / row_stride;
        let first_row = cell_row / (64 * row_stride) * (64 * row_stride);
        for row in (first_row..=cell_row).rev() {
            let cell = mod_data.pattern[row * row_stride + cell_index % row_stride];
            if cell.sample == 0 { continue; }
            let sample_index = cell.sample as usize - 1;
            if sample_index >= mod_data.samples.len() { return None; }
            return Some(sample_index);
        }
        None
    }

    fn select_sample(&mut self, selected_sample: usize) {
        self.selected_sample = selected_sample;
        self.sfx_editor.reset();
    }

    fn samples_tab(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, dialogs: &mut Dialogs,
                   mod_data: &mut ModData, sound_player: &mut SoundPlayer) {
        egui::SidePanel::left(format!("editor_panel_{}_samples_left", self.asset_id)).resizable(false).max_width(160.0).show_inside(ui, |ui| {
            let mut sample_name = String::new();
            egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                for (sample_index, sample) in mod_data.samples.iter().enumerate() {
                    sample_name.clear();
                    if sample.len == 0 {
                        sample_name.push_str(&format!("sample {} (empty)", sample_index + 1));
                    } else {
                        sample_name.push_str(&format!("sample {}", sample_index + 1));
                    };
                    let resp = ui.selectable_label(self.selected_sample == sample_index, &sample_name);
                    if resp.clicked() || resp.secondary_clicked() {
                        self.select_sample(sample_index);
                    }
                    egui::Popup::context_menu(&resp).show(|ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::Image::new(IMAGES.import).max_width(14.0).max_height(14.0));
                            if ui.button("Import WAV...").clicked() {
                                wc.sys_dialogs.open_file(
                                    Some(wc.egui.window),
                                    format!("editor_{}_import_sample", self.asset_id),
                                    "Import WAVE file",
                                    &[
                                        ("WAVE files (*.wav)", &["wav"]),
                                        ("All files (*)", &[""]),
                                    ]
                                );
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.add(egui::Image::new(IMAGES.export).max_width(14.0).max_height(14.0));
                            let enabled = sample.len != 0;
                            if ui.add_enabled(enabled, egui::Button::new("Export WAV...")).clicked() {
                                dialogs.export_sample_dialog.set_open(wc, self.selected_sample, 22050, sample.bits_per_sample);
                            }
                        });
                    });
                }
            });
        });

        // sample properties
        egui::TopBottomPanel::bottom(format!("editor_panel_{}_sample_properties", self.asset_id)).show_inside(ui, |ui| {
            if let Some(sample) = mod_data.samples.get_mut(self.selected_sample) {
                let sample_data = if let Some(data) = &sample.data { &data[..] } else { &[] };
                let mut loop_start = sample.loop_start as f32;
                let mut loop_end = (sample.loop_start + sample.loop_len) as f32;
                let mut volume = sample.volume as f32;
                let mut finetune = sample.finetune as f32;

                egui::Grid::new(format!("editor_{}_sample_grid", self.asset_id)).num_columns(2).show(ui, |ui| {
                    // properties
                    egui::CollapsingHeader::new("Sample").default_open(true).show(ui, |ui| {
                        egui::Grid::new(format!("editor_{}_loop_grid", self.asset_id)).num_columns(2).show(ui, |ui| {
                            ui.label("Bits/sample:");
                            egui::ComboBox::from_id_salt(format!("editor_{}_bps_combo", self.asset_id))
                                .selected_text(format!("{}", sample.bits_per_sample))
                                .width(50.0)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut sample.bits_per_sample, 8, "8");
                                    ui.selectable_value(&mut sample.bits_per_sample, 16, "16");
                                });
                            ui.end_row();

                            ui.label("Length:");
                            ui.label(format!("{}", sample.len));
                            ui.end_row();

                            ui.label("Loop start:");
                            ui.add(egui::DragValue::new(&mut loop_start).speed(1.0).range(0.0..=sample_data.len() as f32));
                            ui.end_row();

                            ui.label("Loop end:");
                            ui.add(egui::DragValue::new(&mut loop_end).speed(1.0).range(loop_start..=sample_data.len() as f32));
                            ui.end_row();

                            ui.label("Volume:");
                            ui.add(egui::DragValue::new(&mut volume).speed(1.0).range(0.0..=63.0));
                            ui.end_row();

                            ui.label("Finetune:");
                            ui.add(egui::DragValue::new(&mut finetune).speed(1.0).range(-8.0..=7.0));
                            ui.end_row();
                        });
                    });

                    // playback
                    egui::CollapsingHeader::new("Test").default_open(sound_player.is_available()).show(ui, |ui| {
                        if sound_player.is_available() {
                            ui.horizontal(|ui| {
                                ui.add(egui::DragValue::new(&mut self.play_freq).speed(25.0).range(2000.0..=44100.0));
                                ui.label("Hz");
                                ui.add_space(5.0);
                                if ui.button("▶ Play ").clicked() {
                                    sound_player.play_s16(sample_data, self.play_freq, self.play_volume);
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

                sample.loop_start = loop_start as u32;
                sample.loop_len = (loop_end - loop_start).max(0.0) as u32;
                sample.volume = volume.clamp(0.0, 63.0) as u8;
                sample.finetune = finetune.clamp(-8.0, 7.0) as i8;
            }
        });

        // wave display
        egui::CentralPanel::default().show_inside(ui, |ui| {
            if let Some(sample) = mod_data.samples.get_mut(self.selected_sample) {
                let sample_data = if let Some(data) = &sample.data { &data[..] } else { &[] };
                let mut loop_start = sample.loop_start as f32;
                let mut loop_end = (sample.loop_start + sample.loop_len) as f32;

                self.sfx_editor.show(ui, sample_data, &mut loop_start, &mut loop_end, 0.0);

                sample.loop_start = loop_start.max(0.0) as u32;
                sample.loop_len = (loop_end - loop_start).max(0.0) as u32;
            }
        });
    }

    fn patterns_tab(&mut self, ui: &mut egui::Ui, _wc: &WindowContext, mod_data: &mut ModData, sound_player: &mut SoundPlayer) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let cur_song_position = mod_data.song_positions.get(self.selected_song_position).map_or_else(|| {
                mod_data.song_positions.first().map_or(0, |&v| v)
            }, |&v| v);
            let old_selected_song_position = self.selected_song_position;
            ui.horizontal(|ui| {
                ui.label("Pattern:");
                egui::ComboBox::from_id_salt(format!("editor_{}_song_pos_combo", self.asset_id))
                    .selected_text(format!("{}", cur_song_position))
                    .width(50.0)
                    .show_ui(ui, |ui| {
                        for (i, &song_pos) in mod_data.song_positions.iter().enumerate() {
                            ui.selectable_value(&mut self.selected_song_position, i, format!("{}", song_pos));
                        }
                    });
            });
            ui.add_space(5.0);

            let num_channels = mod_data.num_channels as usize;
            ui.style_mut().spacing.item_spacing = egui::Vec2::ZERO;
            let available_height = ui.available_height();
            let mut table = TableBuilder::new(ui)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::exact(40.0).clip(true));

            if self.selected_song_position != old_selected_song_position {
                // song position changed, go back to top
                table = table.scroll_to_row(0, Some(egui::Align::TOP));
            }

            for _ in 0..num_channels {
                table = table.columns(Column::exact(45.0).clip(true), MOD_PATTERN_CELL_NAMES.len());
            }

            let scroll = table.auto_shrink(false)
                .striped(true)
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                .min_scrolled_height(0.0)
                .max_scroll_height(available_height)
                .header(20.0, |mut header| {
                    header.col(|_ui| {});
                    for _ in 0..num_channels {
                        for h in MOD_PATTERN_CELL_NAMES.iter() {
                            header.col(|ui| { ui.label(*h); });
                        }
                    }
                })
                .body(|body| {
                    body.rows(20.0, 64, |mut row| {
                        let row_index = row.index();
                        row.set_overline(row_index.is_multiple_of(4));
                        row.col(|ui| {
                            ui.label(format!("{:>2}", row_index));
                        });
                        for i in 0..num_channels {
                            let cell_index_in_pattern = row_index * num_channels + i;
                            let pattern_stride = 64 * num_channels;
                            let cell_index = if let Some(&pattern_num) = mod_data.song_positions.get(self.selected_song_position) &&
                                (pattern_stride * pattern_num as usize + cell_index_in_pattern) < mod_data.pattern.len() {
                                    pattern_stride * pattern_num as usize + cell_index_in_pattern
                                } else {
                                    return;
                                };
                            let cell = mod_data.pattern[cell_index];

                            // note
                            row.col(|ui| {
                                if cell.period != 0 {
                                    let (note, octave) = ModData::get_period_note(cell.period);
                                    if note >= 0 {
                                        if sound_player.is_available() {
                                            if ui.button(format!("{:2}{}", mod_utils::get_note_name(note), octave)).clicked() &&
                                                let Some(freq) = mod_utils::get_period_sample_rate(cell.period) &&
                                                let Some(sample_index) = Self::get_pattern_sample_to_play(cell_index, mod_data) &&
                                                let Some(sample_data) = &mod_data.samples[sample_index].data {
                                                    sound_player.play_s16(sample_data, freq, self.play_volume);
                                                    self.play_freq = freq.round();
                                                    self.select_sample(sample_index);
                                                }
                                        } else {
                                            ui.add(egui::Label::new(format!("{:2}{}", mod_utils::get_note_name(note), octave))
                                                   .selectable(false));
                                        }
                                    }
                                }
                            });
                            // sample
                            row.col(|ui| {
                                if cell.period != 0 && cell.sample != 0 {
                                    ui.add(egui::Label::new(cell.sample.to_string()).selectable(false));
                                }
                            });
                            // fx
                            row.col(|ui| {
                                if cell.effect != 0 {
                                    ui.add(egui::Label::new(format!("{:03X}", cell.effect)).selectable(false)).on_hover_ui(|ui| {
                                        let (note, _) = ModData::get_period_note(cell.period);
                                        if let Some(tooltip) = mod_utils::get_effect_description(cell.effect, note, &mod_data.song_positions) {
                                            ui.label(tooltip);
                                        } else {
                                            ui.label("unknown effect");
                                        }
                                    });
                                }
                            });
                        }
                    });
                });

            // draw column separators between MOD cells
            let painter = ui.painter();
            let t_rect = scroll.inner_rect;
            let origin = t_rect.min;
            let stroke = ui.style().visuals.window_stroke;
            for x in 0..5 {
                let x = x as f32;
                painter.vline(origin.x + 35.0 + x * 3.0 * 45.0, t_rect.y_range(), stroke);
            }
        });
    }

    fn import_mod(&mut self, wc: &mut WindowContext, filename: &std::path::Path, mod_data: &mut ModData) {
        match mod_utils::ModFile::read(filename) {
            Ok(mod_file) => {
                mod_data.samples = mod_file.samples;
                mod_data.pattern = mod_file.pattern;
                mod_data.song_positions = mod_file.song_positions;
                mod_data.num_channels = mod_file.num_channels;
            }

            Err(e) => {
                wc.logger.log(format!("ERROR reading MOD file from {}:", filename.display()));
                wc.logger.log(format!("{}", e));
                wc.open_message_box("Error importing MOD", "Error importing MOD file.\n\nConsult the log window for more information.");
            }
        }
    }

    fn export_mod(&mut self, wc: &mut WindowContext, filename: &std::path::Path, mod_data: &ModData) {
        if let Err(e) = mod_utils::ModFile::write_mod_data(filename, mod_data) {
            wc.logger.log(format!("ERROR writing MOD file to {}:", filename.display()));
            wc.logger.log(format!("{}", e));
            wc.open_message_box("Error exporting MOD", "Error exporting MOD file.\n\nConsult the log window for more information.");
        }
    }

    fn import_sample(&mut self, wc: &mut WindowContext, filename: &std::path::Path, mod_data: &mut ModData) {
        let result = wav_utils::WavFile::read(filename).and_then(|mut wav_file| {
            if wav_file.channels.is_empty() { return Err(Error::other("WAV with no channels!?")); }
            let wav_sample = wav_file.channels.remove(0);
            let sample = mod_data.samples.get_mut(self.selected_sample)
                .ok_or(Error::other(format!("can't find selected sample: {}", self.selected_sample)))?;
            sample.len = wav_sample.len() as u32;
            sample.data = Some(wav_sample);
            sample.bits_per_sample = wav_file.bits_per_sample;
            sample.loop_start = 0;
            sample.loop_len = 0;
            Ok(())
        });

        if let Err(e) = result {
            wc.logger.log(format!("ERROR reading WAVE file from {}:", filename.display()));
            wc.logger.log(format!("{}", e));
            wc.open_message_box("Error importing sample", "Error importing WAVE file.\n\nConsult the log window for more information.");
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, dialogs: &mut Dialogs,
                mod_data: &mut ModData, sound_player: &mut SoundPlayer) {
        if let Some(SysDialogResponse::File(filename)) = wc.sys_dialogs.get_response_for(format!("editor_{}_import_sample", self.asset_id)) {
            self.import_sample(wc, &filename, mod_data);
        }
        if let Some(SysDialogResponse::File(filename)) = wc.sys_dialogs.get_response_for(format!("editor_{}_import_mod", self.asset_id)) {
            self.import_mod(wc, &filename, mod_data);
        }
        if let Some(SysDialogResponse::File(filename)) = wc.sys_dialogs.get_response_for(format!("editor_{}_export_mod", self.asset_id)) {
            self.export_mod(wc, &filename, mod_data);
        }

        // header:
        egui::TopBottomPanel::top(format!("editor_panel_{}_top", self.asset_id)).show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("MOD", |ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.import).max_width(14.0).max_height(14.0));
                        if ui.button("Import...").clicked() {
                            wc.sys_dialogs.open_file(
                                Some(wc.egui.window),
                                format!("editor_{}_import_mod", self.asset_id),
                                "Import MOD file",
                                &[
                                    ("MOD files (*.mod)", &["mod"]),
                                    ("All files (*)", &[""]),
                                ]
                            );
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.export).max_width(14.0).max_height(14.0));
                        if ui.button("Export...").clicked() {
                            wc.sys_dialogs.save_file(
                                Some(wc.egui.window),
                                format!("editor_{}_export_mod", self.asset_id),
                                "Export MOD file",
                                &[
                                    ("MOD files (*.mod)", &["mod"]),
                                    ("All files (*)", &[""]),
                                ]
                            );
                        }
                    });
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.properties).max_width(14.0).max_height(14.0));
                        if ui.button("Properties...").clicked() {
                            dialogs.properties_dialog.set_open(wc, mod_data);
                        }
                    });
                });
            });
        });

        // footer:
        egui::TopBottomPanel::bottom(format!("editor_panel_{}_bottom", self.asset_id)).show_inside(ui, |ui| {
            ui.add_space(5.0);
            let num_samples = mod_data.samples.iter().fold(0, |n, spl| {
                n + if spl.len != 0 && spl.data.is_some() { 1 } else { 0 }
            });
            ui.label(format!("{} bytes [samples: {}, song pos: {}]", mod_data.data_size(), num_samples, mod_data.song_positions.len()));
        });

        // tabs:
        egui::TopBottomPanel::top(format!("editor_panel_{}_tabs", self.asset_id)).show_inside(ui, |ui| {
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
            EditorTabs::Samples => { self.samples_tab(ui, wc, dialogs, mod_data, sound_player) }
            EditorTabs::Patterns => { self.patterns_tab(ui, wc, mod_data, sound_player); }
        };
    }
}
