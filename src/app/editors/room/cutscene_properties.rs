use crate::data_store::{
    Room,
};
use super::super::{
    AssetEditorBase,
    WindowContext,
};

pub struct CutscenePropertiesDialog {
    pub open: bool,
    pub dlg_id: egui::Id,
    pub name: String,
    pub cutscene_index: usize,
}

impl CutscenePropertiesDialog {
    pub fn new() -> Self {
        CutscenePropertiesDialog {
            open: false,
            dlg_id: egui::Id::unique("dlg_room_cutscene_properties"),
            name: String::new(),
            cutscene_index: 0,
        }
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, room: &Room, cutscene_index: usize) {
        if let Some(cutscene) = room.cutscenes.get(cutscene_index) {
            self.cutscene_index = cutscene_index;
            self.name.replace_range(.., &cutscene.name_id);
            self.open = true;
            wc.set_dialog_open(self.dlg_id, self.open);
        }
    }

    fn confirm(&mut self, room: &mut Room) {
        if let Some(cutscene) = room.cutscenes.get_mut(self.cutscene_index) {
            cutscene.name_id.replace_range(.., &self.name);
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, room: &mut Room) {
        if ! self.open { return; }

        if AssetEditorBase::show_dialog_window(wc, self.dlg_id, 400.0, "Cutscene Properties", |ui, _wc| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                egui::Grid::new(format!("editor_panel_{}_prop_grid", room.asset.id))
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
                    self.confirm(room);
                    ui.close();
                }
            });
        }).should_close() {
            self.open = false;
            wc.set_dialog_open(self.dlg_id, self.open);
        }
    }
}
