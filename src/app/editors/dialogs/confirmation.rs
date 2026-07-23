use crate::data_asset::DataAssetId;

use super::super::{
    AssetEditorBase,
    WindowContext,
    DialogResult,
};

pub struct ConfirmationDialog {
    id: egui::Id,
    pub open: bool,
    pub title: String,
    pub text: String,
    pub yes_label: String,
    pub no_label: String,
}

impl ConfirmationDialog {
    pub fn new(asset_id: DataAssetId) -> Self {
        ConfirmationDialog {
            id: egui::Id::new(format!("editor_{}_dlg_confirmation", asset_id)),
            title: String::new(),
            text: String::new(),
            yes_label: String::new(),
            no_label: String::new(),
            open: false,
        }
    }

    pub fn set_open(
        &mut self,
        wc: &mut WindowContext,
        title: impl AsRef<str>,
        text: impl AsRef<str>,
        yes_label: impl AsRef<str>,
        no_label: impl AsRef<str>,
    ) {
        self.title.replace_range(.., title.as_ref());
        self.text.replace_range(.., text.as_ref());
        self.yes_label.replace_range(.., yes_label.as_ref());
        self.no_label.replace_range(.., no_label.as_ref());
        self.open = true;
        wc.set_dialog_open(self.id, self.open);
    }

    pub fn show(&mut self, wc: &mut WindowContext) -> DialogResult {
        let resp = AssetEditorBase::show_dialog_window(wc, self.id, 300.0, &self.title, |ui, _wc| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                ui.label(&self.text);
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if ui.button(&self.yes_label).clicked() {
                    ui.close();
                    DialogResult::Yes
                } else if ui.button(&self.no_label).clicked() {
                    ui.close();
                    DialogResult::No
                } else {
                    DialogResult::None
                }
            }).inner
        });
        if resp.should_close() {
            self.open = false;
            wc.set_dialog_open(self.id, self.open);
            if resp.inner == DialogResult::None {
                // closing because the user clicked outside the window
                return DialogResult::Cancel;
            }
        }
        resp.inner
    }
}
