use crate::data_store::PlayerChar;

use super::super::{
    AssetEditorBase,
    WindowContext,
};

pub struct PropertiesDialog {
    dlg_id: egui::Id,
    pub open: bool,
    pub name: String,
}

impl PropertiesDialog {
    pub fn new() -> Self {
        PropertiesDialog {
            dlg_id: egui::Id::unique("dlg_player_char_properties"),
            open: false,
            name: String::new(),
        }
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, player_char: &PlayerChar) {
        self.name.replace_range(.., &player_char.asset.name);
        self.open = true;
        wc.set_dialog_open(self.dlg_id, self.open);
    }

    fn confirm(&mut self, player_char: &mut PlayerChar) {
        player_char.asset.name.replace_range(.., &self.name);
    }

    pub fn show(&mut self, wc: &mut WindowContext, player_char: &mut PlayerChar) {
        if AssetEditorBase::show_dialog_window(wc, self.dlg_id, 300.0, "Character Properties", |ui, _wc| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                egui::Grid::new(format!("editor_panel_{}_prop_grid", player_char.asset.id))
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.name);
                        ui.end_row();
                    });
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if ui.button("Cancel").clicked() {
                    ui.close();
                }
                if ui.button("Ok").clicked() {
                    self.confirm(player_char);
                    ui.close();
                }
            });
        }).should_close() {
            self.open = false;
            wc.set_dialog_open(self.dlg_id, self.open);
        }
    }
}
