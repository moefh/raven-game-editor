use crate::misc::wav_utils;
use crate::data_asset::Sfx;

use super::super::{
    AssetEditorBase,
    WindowContext,
    SysDialogResponse,
    SysDialogOpenFile,
};

const ALLOWED_SAMPLE_RATES: &[u32] = &[ 8000, 11025, 22050, 44100 ];
const ALLOWED_BITS_PER_SAMPLE: &[u16] = &[ 8, 16 ];

pub struct ExportDialog {
    pub open: bool,
    pub dlg_window_id: egui::Id,
    pub sample_rate: u32,
    pub bits_per_sample: u16,
    pub export_sys_dlg_id: String,
}

impl ExportDialog {
    pub fn new() -> Self {
        ExportDialog {
            open: false,
            dlg_window_id: egui::Id::new("dlg_sfx_export"),
            bits_per_sample: 0,
            sample_rate: 0,
            export_sys_dlg_id: String::new(),
        }
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, sfx: &Sfx, sample_rate: u32) {
        self.bits_per_sample = sfx.bits_per_sample;
        self.sample_rate = sample_rate;
        self.export_sys_dlg_id.replace_range(.., &format!("editor_{}_export_sfx", sfx.asset.id));
        self.open = true;
        wc.set_dialog_open(self.dlg_window_id, self.open);
    }

    fn close(&mut self, wc: &mut WindowContext) {
        self.open = false;
        wc.set_dialog_open(self.dlg_window_id, self.open);
    }

    fn confirm(&mut self, file: SysDialogOpenFile, wc: &mut WindowContext, sfx: &mut Sfx) -> bool {
        if let Err(e) = wav_utils::WavFile::write(self.sample_rate, self.bits_per_sample, &sfx.samples)
            .and_then(|data| file.write_data(data)) {
                wc.logger.log(format!("ERROR writing WAVE file to {}:", file.filename()));
                wc.logger.log(format!("{}", e));
                wc.open_message_box(
                    "Error Writing Sample",
                    "Error exporting WAVE file.\n\nConsult the log window for more information."
                );
                false
            } else {
                true
            }
    }

    pub fn show(&mut self, wc: &mut WindowContext, sfx: &mut Sfx) {
        if ! self.open { return; }
        if let Some(SysDialogResponse::File(file)) = wc.sys_dialogs.get_response_for(&self.export_sys_dlg_id) &&
            self.confirm(file, wc, sfx) {
                self.close(wc);
            }

        if AssetEditorBase::show_dialog_window(wc, self.dlg_window_id, 300.0, "Export Sfx", |ui, wc| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                egui::Grid::new(format!("editor_panel_{}_export_sfx_grid", sfx.asset.id))
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
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
                if ui.button("Save File").clicked() {
                    wc.sys_dialogs.save_file(
                        Some(wc.egui.window),
                        self.export_sys_dlg_id.clone(),
                        "sfx",
                        "Export Sfx",
                        &[
                            ("WAVE files (*.wav)", &["wav"]),
                            ("All files (*.*)", &["*"]),
                        ]
                    );
                }
                if ui.button("Cancel").clicked() {
                    ui.close();
                }
            });
        }).should_close() {
            self.close(wc);
        }
    }
}
