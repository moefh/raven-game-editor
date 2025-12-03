use std::path::PathBuf;

use crate::app::{WindowContext, SysDialogResponse};
use crate::misc::wav_utils;
use crate::data_asset::Sfx;

const ALLOWED_SAMPLE_RATES: &[u32] = &[ 8000, 11025, 22050, 44100 ];
const ALLOWED_BITS_PER_SAMPLE: &[u16] = &[ 8, 16 ];

pub struct ExportDialog {
    pub open: bool,
    pub filename: Option<PathBuf>,
    pub display_filename: Option<String>,
    pub sample_rate: u32,
    pub bits_per_sample: u16,
}

impl ExportDialog {
    pub fn new() -> Self {
        ExportDialog {
            open: false,
            filename: None,
            display_filename: None,
            bits_per_sample: 0,
            sample_rate: 0,
        }
    }

    pub fn id() -> egui::Id {
        egui::Id::new("dlg_sfx_export")
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, sfx: &Sfx, sample_rate: u32) {
        self.filename = None;
        self.display_filename = None;
        self.bits_per_sample = sfx.bits_per_sample;
        self.sample_rate = sample_rate;
        self.open = true;
        wc.set_window_open(Self::id(), self.open);
    }

    fn confirm(&mut self, wc: &mut WindowContext, sfx: &mut Sfx) -> bool {
        if let Some(filename) = &self.filename {
            if let Err(e) = wav_utils::WavFile::write(filename, self.sample_rate, self.bits_per_sample, &sfx.samples) {
                wc.logger.log(format!("ERROR writing WAVE file to {}:", filename.display()));
                wc.logger.log(format!("{}", e));
                wc.open_message_box("Error Writing Sample",
                                    "Error exporting WAVE file.\n\nConsult the log window for more information.");
            }
            true
        } else {
            wc.open_message_box("Filename Needed", "You need to select a filename to export.");
            false
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, sfx: &mut Sfx) {
        if ! self.open { return; }
        let export_dlg_id = format!("editor_{}_export_sfx", sfx.asset.id);
        if let Some(SysDialogResponse::File(filename)) = wc.sys_dialogs.get_response_for(&export_dlg_id) {
            self.display_filename = Some(filename.as_path().file_name().map(|f| f.display().to_string()).unwrap_or("?".to_owned()));
            self.filename = Some(filename);
        }

        if egui::Modal::new(Self::id()).show(wc.egui.ctx, |ui| {
            ui.set_width(300.0);
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                ui.heading("Export Sfx");
                ui.separator();

                egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                    egui::Grid::new(format!("editor_panel_{}_export_sfx_grid", sfx.asset.id))
                        .num_columns(2)
                        .spacing([8.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("File name:");
                            ui.horizontal(|ui| {
                                if let Some(display_filename) = &self.display_filename {
                                    ui.add(egui::Label::new(display_filename).truncate());
                                } else {
                                    ui.label("");
                                }
                                if ui.button("...").clicked() {
                                    wc.sys_dialogs.save_file(
                                        Some(wc.egui.window),
                                        export_dlg_id,
                                        "Export Sfx",
                                        &[
                                            ("WAVE files (*.wav)", &["wav"]),
                                            ("All files (*.*)", &["*"]),
                                        ]
                                    );
                                }
                            });
                            ui.end_row();

                            ui.label("Sample rate:");
                            ui.horizontal(|ui| {
                                egui::ComboBox::from_id_salt(format!("editor_{}_export_sfx_sample_rate", sfx.asset.id))
                                    .selected_text(self.sample_rate.to_string())
                                    .width(60.0)
                                    .show_ui(ui, |ui| {
                                        for sample_rate in ALLOWED_SAMPLE_RATES {
                                            ui.selectable_value(&mut self.sample_rate, *sample_rate, sample_rate.to_string());
                                        }
                                    });
                                ui.label("Hz");
                            });
                            ui.end_row();

                            ui.label("Bits/sample:");
                            egui::ComboBox::from_id_salt(format!("editor_{}_export_sfx_bits_per_sample", sfx.asset.id))
                                .selected_text(self.bits_per_sample.to_string())
                                .width(60.0)
                                .show_ui(ui, |ui| {
                                    for bits_per_sample in ALLOWED_BITS_PER_SAMPLE {
                                        ui.selectable_value(&mut self.bits_per_sample, *bits_per_sample, bits_per_sample.to_string());
                                    }
                                });
                            ui.end_row();
                        });
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui.button("Cancel").clicked() {
                        ui.close();
                    }
                    if ui.button("Ok").clicked() && self.confirm(wc, sfx) {
                        ui.close();
                    }
                });
            });
        }).should_close() {
            self.open = false;
            wc.set_window_open(Self::id(), self.open);
        }
    }
}
