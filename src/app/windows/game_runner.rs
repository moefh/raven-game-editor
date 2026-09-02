use crate::misc::IMAGES;
use crate::data_asset::{
    self,
    DataAssetStore,
    DataAssetId,
};

use super::{
    AppWindowBase,
    AppWindowAction,
};
use super::super::{
    get_setting_zoom,
    WindowContext,
};
use super::super::widgets::{
    GameRunnerWidget,
};

pub struct GameRunnerWindow {
    pub base: AppWindowBase,
    display: GameDisplay,
}

impl GameRunnerWindow {
    pub fn new(base: AppWindowBase) -> Self {
        GameRunnerWindow {
            base,
            display: GameDisplay::new(),
        }
    }

    pub fn reset(&mut self) {
        self.display.game.reset();
    }

    pub fn open(&mut self, ctx: &egui::Context) {
        self.base.open = true;
        if self.base.open {
            self.base.bring_to_top(ctx);
        }
    }

    pub fn set_room(&mut self, room_id: DataAssetId, store: &DataAssetStore) {
        self.display.game.set_room(Some(room_id), store);
    }

    pub fn show(&mut self, wc: &mut WindowContext, store: &DataAssetStore) -> AppWindowAction {
        let default_zoom = get_setting_zoom(wc.settings.game_runner_default_zoom);
        let default_size = GameRunnerWidget::SCREEN_SIZE * default_zoom + egui::Vec2::new(18.0, 80.0);
        let default_rect = self.base.default_rect(wc, default_size.x, default_size.y);
        self.base.show_window(wc, default_rect, [340.0, 320.0], |ui, wc, base| {
            let action = base.show_title_bar(ui, Some(IMAGES.pico), "Game Tester");
            self.display.show(ui, wc, base.id, store);
            action
        })
    }
}

struct GameDisplay {
    game: GameRunnerWidget,
    sorted_room_ids: Vec<DataAssetId>,
}

impl GameDisplay {
    fn new() -> Self {
        GameDisplay {
            game: GameRunnerWidget::new(),
            sorted_room_ids: Vec::new(),
        }
    }

    fn sort_ids(&mut self, store: &DataAssetStore) {
        if self.sorted_room_ids.len() != store.asset_ids.rooms.len() {
            store.asset_ids.rooms.copy_to(&mut self.sorted_room_ids);
            data_asset::utils::sort_asset_ids_by_name(&mut self.sorted_room_ids, &store.assets.rooms);
        }
    }

    fn show_room_combo(&mut self, ui: &mut egui::Ui, store: &DataAssetStore) {
        let cur_room_name = if let Some(room) = self.game.room_id.and_then(|room_id| store.assets.rooms.get(&room_id)) {
            &room.asset.name
        } else {
            "--"
        };
        let mut sel_room_id = self.game.room_id;
        egui::ComboBox::from_id_salt("game_runner_toolbar_room_combo")
            .selected_text(cur_room_name)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut sel_room_id, None, "--");
                for room_id in self.sorted_room_ids.iter() {
                    if let Some(room) = store.assets.rooms.get(room_id) {
                        ui.selectable_value(&mut sel_room_id, Some(room.asset.id), &room.asset.name);
                    }
                }
            });
        if self.game.room_id != sel_room_id {
            self.game.set_room(sel_room_id, store);
        }
    }

    fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, window_id: egui::Id, store: &DataAssetStore) {
        self.sort_ids(store);
        egui::Panel::top("game_runner_toolbar").show(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.label("Room:");
                self.show_room_combo(ui, store);

                if ui.add(egui::Button::image(IMAGES.reload)).on_hover_text("Restart room").clicked() {
                    self.game.set_room(self.game.room_id, store);
                }
            });
            ui.add_space(0.0);
        });

        egui::CentralPanel::default().show(ui, |ui| {
            self.game.show(ui, wc, window_id, store);
        });
    }
}
