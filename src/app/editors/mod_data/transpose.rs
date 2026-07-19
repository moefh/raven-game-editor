use crate::app::WindowContext;
use crate::data_asset::{ModData, ModCell, MOD_PERIOD_TABLE};
use crate::misc::mod_utils;
use super::super::AssetEditorBase;

#[derive(PartialEq)]
pub enum NoteFilter {
    Any,
    AtOrBelow,
    AtOrAbove,
}

impl NoteFilter {
    const TEXT: &[&str] = &[
        "Any note",
        "Only notes at/below:",
        "Only notes at/above:",
    ];

    fn text(&self) -> &'static str {
        match self {
            NoteFilter::Any       => Self::TEXT[0],
            NoteFilter::AtOrBelow => Self::TEXT[1],
            NoteFilter::AtOrAbove => Self::TEXT[2],
        }
    }
}

#[derive(PartialEq)]
pub enum SampleFilter {
    AnyOrNoSample,
    NoSample,
    Sample,
}

impl SampleFilter {
    const TEXT: &[&str] = &[
        "Any/no sample",
        "No sample",
        "This sample:",
    ];

    fn text(&self) -> &'static str {
        match self {
            SampleFilter::AnyOrNoSample => Self::TEXT[0],
            SampleFilter::NoSample      => Self::TEXT[1],
            SampleFilter::Sample        => Self::TEXT[2],
        }
    }
}

#[derive(PartialEq)]
pub enum ChannelFilter {
    Any,
    Channel,
}

impl ChannelFilter {
    const TEXT: &[&str] = &[
        "Any channel",
        "This channel:",
    ];

    fn text(&self) -> &'static str {
        match self {
            ChannelFilter::Any     => Self::TEXT[0],
            ChannelFilter::Channel => Self::TEXT[1],
        }
    }
}

#[derive(PartialEq)]
pub enum ReplaceSampleOption {
    DontReplace,
    ReplaceWithEmpty,
    ReplaceWithSample,
}

impl ReplaceSampleOption {
    const TEXT: &[&str] = &[
        "Don't change the sample",
        "Replace with no sample",
        "Replace with sample:",
    ];

    fn text(&self) -> &'static str {
        match self {
            ReplaceSampleOption::DontReplace       => Self::TEXT[0],
            ReplaceSampleOption::ReplaceWithEmpty  => Self::TEXT[1],
            ReplaceSampleOption::ReplaceWithSample => Self::TEXT[2],
        }
    }
}

pub struct TransposeDialog {
    pub open: bool,
    transpose_amount: i32,
    chan_filter_cmp: ChannelFilter,
    chan_filter_channel: u8,
    note_filter_cmp: NoteFilter,
    note_filter_note: i32,
    note_filter_octave: i32,
    sample_filter_cmp: SampleFilter,
    sample_filter_sample: u8,
    replace_sample_opt: ReplaceSampleOption,
    replace_sample_val: u8,
}

impl TransposeDialog {
    const INTERVAL_NAMES: &[&str] = &[
        "",
        "semitone",
        "tone",
        "minor third",
        "major third",
        "fourth",
        "tritone",
        "fifth",
        "minor sixth",
        "major sixth",
        "minor seventh",
        "major seventh",
    ];
    const TRANSPOSE_AMOUNT_MIN: i32 = -3*12;
    const TRANSPOSE_AMOUNT_MAX: i32 =  3*12;

    pub fn new() -> Self {
        TransposeDialog {
            open: false,
            transpose_amount: 0,
            chan_filter_cmp: ChannelFilter::Any,
            chan_filter_channel: 0,
            note_filter_cmp: NoteFilter::Any,
            note_filter_note: 0,
            note_filter_octave: 3,
            sample_filter_cmp: SampleFilter::AnyOrNoSample,
            sample_filter_sample: 1,
            replace_sample_opt: ReplaceSampleOption::DontReplace,
            replace_sample_val: 1,
        }
    }

    fn get_transpose_amount_text(semitones: i32) -> String {
        if semitones == 0 {
            String::new()
        } else if semitones.abs() == 12 {
            "(octave)".to_owned()
        } else if semitones.abs() % 12 == 0 {
            format!("({} octaves)", semitones.abs() / 12)
        } else if semitones.abs() < 12 {
            format!("({})", Self::INTERVAL_NAMES[semitones.unsigned_abs() as usize])
        } else if semitones.abs() <= 24 {
            format!("({} and 1 octave)", Self::INTERVAL_NAMES[(semitones.abs() % 12) as usize])
        } else {
            let octaves = semitones.abs() / 12;
            format!("({} and {} octaves)", Self::INTERVAL_NAMES[(semitones.abs() % 12) as usize], octaves)
        }
    }

    pub fn id() -> egui::Id {
        egui::Id::new("dlg_mod_transpose")
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, _mod_data: &ModData) {
        self.chan_filter_cmp = ChannelFilter::Any;
        self.chan_filter_channel = 0;
        self.note_filter_cmp = NoteFilter::Any;
        self.note_filter_note = 0;
        self.note_filter_octave = 3;
        self.sample_filter_cmp = SampleFilter::AnyOrNoSample;
        self.sample_filter_sample = 1;
        self.transpose_amount = 0;
        self.replace_sample_opt = ReplaceSampleOption::DontReplace;
        self.replace_sample_val = 1;
        self.open = true;
        wc.set_dialog_open(Self::id(), self.open);
    }

    fn filter_cell(&self, cell: ModCell, cell_chan: u8, filter_period: u16) -> bool {
        if cell.period == 0 { return false; }
        match self.note_filter_cmp {
            NoteFilter::AtOrAbove if cell.period > filter_period => { return false; }  // note_1 > note_2 => period_1 < period_2
            NoteFilter::AtOrBelow if cell.period < filter_period => { return false; }  // note_1 < note_2 => period_1 > period_2
            _ => {}
        }
        match self.sample_filter_cmp {
            SampleFilter::NoSample if cell.sample != 0 => { return false; }
            SampleFilter::Sample if cell.sample != self.sample_filter_sample => { return false; }
            _ => {}
        }
        match self.chan_filter_cmp {
            ChannelFilter::Channel if cell_chan != self.chan_filter_channel => { return false; }
            _ => {}
        }
        true
    }

    fn transpose_note_period(period: u16, transpose_amount: i32) -> u16 {
        if transpose_amount == 0 { return period; }

        let (mut note, mut octave) = ModData::get_period_note(period);
        note += transpose_amount;
        while note > 12 { note -= 12; octave += 1; }
        while note < 12 { note += 12; octave -= 1; }
        ModData::get_note_period(note, octave)
    }

    fn confirm(&mut self, mod_data: &mut ModData) {
        let filter_period = ModData::get_note_period(self.note_filter_note, self.note_filter_octave);

        for (cell_index, cell) in mod_data.pattern.iter_mut().enumerate() {
            let cell_chan = (cell_index % mod_data.num_channels as usize) as u8;
            if self.filter_cell(*cell, cell_chan, filter_period) {
                cell.period = Self::transpose_note_period(cell.period, self.transpose_amount);
                match self.replace_sample_opt {
                    ReplaceSampleOption::ReplaceWithEmpty => { cell.sample = 0; }
                    ReplaceSampleOption::ReplaceWithSample => { cell.sample = self.replace_sample_val; }
                    _ => {} // don't replace
                }
            }
        }
    }

    fn visibility_ui_builder(visible: bool) -> egui::UiBuilder {
        if visible {
            egui::UiBuilder::new()
        } else {
            egui::UiBuilder::new().invisible()
        }
    }

    fn get_sample_name(mod_data: &ModData, sample: u8) -> String {
        if sample == 0 { return "INVALID".to_owned(); }
        if mod_data.samples[(sample - 1) as usize].len == 0 {
            format!("sample {} (empty)", sample)
        } else {
            format!("sample {}", sample)
        }
    }

    fn show_filters(&mut self, ui: &mut egui::Ui, mod_data: &mut ModData) {
        ui.horizontal(|ui| {
            ui.label("Filter:");
        });
        egui::Grid::new(format!("editor_panel_{}_transp_filter_grid", mod_data.asset.id))
            .num_columns(2)
            .spacing([8.0, 8.0])
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(20.0);
                    ui.label("By channel:");
                });
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_salt(format!("editor_panel_{}_transp_filter_chan_cmp", mod_data.asset.id))
                        .selected_text(self.chan_filter_cmp.text())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.chan_filter_cmp, ChannelFilter::Any,     ChannelFilter::Any.text());
                            ui.selectable_value(&mut self.chan_filter_cmp, ChannelFilter::Channel, ChannelFilter::Channel.text());
                        });
                    ui.scope_builder(Self::visibility_ui_builder(self.chan_filter_cmp == ChannelFilter::Channel), |ui| {
                        egui::ComboBox::from_id_salt(format!("editor_{}_transp_filter_chan_val", mod_data.asset.id))
                            .selected_text(format!("channel {}", self.chan_filter_channel))
                            .width(150.0)
                            .show_ui(ui, |ui| {
                                for chan in 0..mod_data.num_channels {
                                    ui.selectable_value(&mut self.chan_filter_channel, chan, format!("channel {}", chan));
                                }
                            });
                    });
                });
                ui.end_row();

                ui.horizontal(|ui| {
                    ui.add_space(20.0);
                    ui.label("By note:");
                });
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_salt(format!("editor_panel_{}_transp_filter_note_cmp", mod_data.asset.id))
                        .selected_text(self.note_filter_cmp.text())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.note_filter_cmp, NoteFilter::Any,       NoteFilter::Any.text());
                            ui.selectable_value(&mut self.note_filter_cmp, NoteFilter::AtOrAbove, NoteFilter::AtOrAbove.text());
                            ui.selectable_value(&mut self.note_filter_cmp, NoteFilter::AtOrBelow, NoteFilter::AtOrBelow.text());
                        });
                    ui.scope_builder(Self::visibility_ui_builder(self.note_filter_cmp != NoteFilter::Any), |ui| {
                        egui::ComboBox::from_id_salt(format!("editor_{}_transp_filter_note_val", mod_data.asset.id))
                            .selected_text(mod_utils::get_note_name(self.note_filter_note))
                            .width(50.0)
                            .show_ui(ui, |ui| {
                                for (i, &name) in mod_utils::NOTE_NAMES.iter().enumerate() {
                                    ui.selectable_value(&mut self.note_filter_note, i as i32, name);
                                }
                            });
                        ui.label("Octave:");
                        ui.add(egui::DragValue::new(&mut self.note_filter_octave).range(0 ..= (MOD_PERIOD_TABLE.len() as i32)));
                    });
                });
                ui.end_row();

                ui.horizontal(|ui| {
                    ui.add_space(20.0);
                    ui.label("By sample:");
                });
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_salt(format!("editor_panel_{}_transp_filter_sample_cmp", mod_data.asset.id))
                        .selected_text(self.sample_filter_cmp.text())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.sample_filter_cmp,
                                                SampleFilter::AnyOrNoSample,
                                                SampleFilter::AnyOrNoSample.text());
                            ui.selectable_value(&mut self.sample_filter_cmp,
                                                SampleFilter::NoSample,
                                                SampleFilter::NoSample.text());
                            ui.selectable_value(&mut self.sample_filter_cmp,
                                                SampleFilter::Sample,
                                                SampleFilter::Sample.text());
                        });
                    ui.scope_builder(Self::visibility_ui_builder(self.sample_filter_cmp == SampleFilter::Sample),
                                     |ui| {
                                         egui::ComboBox::from_id_salt(format!("editor_panel_{}_transp_filter_sample_val",
                                                                              mod_data.asset.id))
                                             .selected_text(Self::get_sample_name(mod_data, self.sample_filter_sample))
                                             .show_ui(ui, |ui| {
                                                 for sample_index in 0..mod_data.samples.len() {
                                                     ui.selectable_value(&mut self.sample_filter_sample, (sample_index + 1) as u8,
                                                                         Self::get_sample_name(mod_data, (sample_index + 1) as u8));
                                                 }
                                             });
                                     });
                });
                ui.end_row();
            });
    }

    fn show_change(&mut self, ui: &mut egui::Ui, mod_data: &mut ModData) {
        ui.horizontal(|ui| {
            ui.label("Change:");
        });
        egui::Grid::new(format!("editor_panel_{}_transp_action_grid", mod_data.asset.id))
            .num_columns(2)
            .spacing([8.0, 8.0])
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(20.0);
                    ui.label("Transpose by:");
                });
                ui.horizontal(|ui| {
                    ui.add(egui::Slider::new(&mut self.transpose_amount,
                                             Self::TRANSPOSE_AMOUNT_MIN ..= Self::TRANSPOSE_AMOUNT_MAX));
                    ui.label(Self::get_transpose_amount_text(self.transpose_amount));
                });
                ui.end_row();

                ui.horizontal(|ui| {
                    ui.add_space(20.0);
                    ui.label("Replace sample:");
                });
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_salt(format!("editor_panel_{}_transp_repl_sample_opt", mod_data.asset.id))
                        .selected_text(self.replace_sample_opt.text())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.replace_sample_opt,
                                                ReplaceSampleOption::DontReplace,
                                                ReplaceSampleOption::DontReplace.text());
                            ui.selectable_value(&mut self.replace_sample_opt,
                                                ReplaceSampleOption::ReplaceWithEmpty,
                                                ReplaceSampleOption::ReplaceWithEmpty.text());
                            ui.selectable_value(&mut self.replace_sample_opt,
                                                ReplaceSampleOption::ReplaceWithSample,
                                                ReplaceSampleOption::ReplaceWithSample.text());
                        });
                    ui.scope_builder(Self::visibility_ui_builder(self.replace_sample_opt == ReplaceSampleOption::ReplaceWithSample),
                                     |ui| {
                                         egui::ComboBox::from_id_salt(format!("editor_panel_{}_transp_repl_sample_val",
                                                                              mod_data.asset.id))
                                             .selected_text(Self::get_sample_name(mod_data, self.replace_sample_val))
                                             .show_ui(ui, |ui| {
                                                 for sample_index in 0..mod_data.samples.len() {
                                                     ui.selectable_value(&mut self.replace_sample_val, (sample_index + 1) as u8,
                                                                         Self::get_sample_name(mod_data, (sample_index + 1) as u8));
                                                 }
                                             });
                                     });
                });
                ui.end_row();
            });
    }

    pub fn show(&mut self, wc: &mut WindowContext, mod_data: &mut ModData) {
        if AssetEditorBase::show_dialog_window(wc, Self::id(), 600.0, "Transpose", |ui, _wc| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                self.show_filters(ui, mod_data);
                ui.add_space(20.0);
                self.show_change(ui, mod_data);
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if ui.button("Cancel").clicked() {
                    ui.close();
                }
                if ui.button("Ok").clicked() {
                    self.confirm(mod_data);
                    ui.close();
                }
            });
        }).should_close() {
            self.open = false;
            wc.set_dialog_open(Self::id(), self.open);
        }
    }
}
