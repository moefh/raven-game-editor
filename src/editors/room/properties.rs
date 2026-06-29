use crate::app::WindowContext;
use crate::data_asset::Room;
use super::super::AssetEditorBase;

pub struct PropertiesDialog {
    pub open: bool,
    pub name: String,
}

impl PropertiesDialog {
    pub fn new() -> Self {
        PropertiesDialog {
            open: false,
            name: String::new(),
        }
    }

    pub fn id() -> egui::Id {
        egui::Id::new("dlg_room_properties")
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, room: &Room) {
        self.name.clear();
        self.name.push_str(&room.asset.name);
        self.open = true;
        wc.set_dialog_open(Self::id(), self.open);
    }

    fn confirm(&mut self, room: &mut Room) {
        room.asset.name.clear();
        room.asset.name.push_str(&self.name);
    }

    pub fn show(&mut self, wc: &mut WindowContext, room: &mut Room) {
        if AssetEditorBase::show_dialog_window(wc, Self::id(), 350.0, "Room Properties", |ui, _wc| {
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
            wc.set_dialog_open(Self::id(), self.open);
        }
    }
}
