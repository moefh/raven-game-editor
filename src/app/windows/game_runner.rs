use crate::misc::IMAGES;
use crate::data_asset::{
    DataAssetStore,
    DataAssetId,
};

use super::{
    AppWindowBase,
    AppWindowAction,
};
use super::super::{
    WindowContext,
};
use super::super::widgets::{
    GameRunnerWidget,
};

pub struct GameRunnerWindow {
    pub base: AppWindowBase,
    game: GameDisplay,
}

impl GameRunnerWindow {
    pub fn new(base: AppWindowBase) -> Self {
        let id = base.id;
        GameRunnerWindow {
            base,
            game: GameDisplay::new(id),
        }
    }

    pub fn open(&mut self, wc: &mut WindowContext) {
        self.base.open = true;
        if self.base.open {
            self.base.bring_to_top(wc);
        }
    }

    pub fn reset(&mut self, room_id: DataAssetId, store: &DataAssetStore) {
        self.game.game.reset(room_id, store);
    }

    pub fn show(&mut self, wc: &mut WindowContext, store: &DataAssetStore) -> AppWindowAction {
        let zoom = self.game.game.zoom;
        let default_rect = self.base.default_rect(wc, GameRunnerWidget::WIDTH * zoom + 20.0, GameRunnerWidget::HEIGHT * zoom + 40.0);
        let (width, height) = (default_rect.width(), default_rect.height());
        self.base.show_window(wc, default_rect, [width, height], |ui, wc, base| {
            let action = base.show_title_bar(ui, Some(IMAGES.pico), "Game Test");
            egui::CentralPanel::default().show(ui, |ui| {
                self.game.show(ui, wc, store);
            });
            action
        })
    }
}

struct GameDisplay {
    game: GameRunnerWidget,
}

impl GameDisplay {
    fn new(id: egui::Id) -> Self {
        GameDisplay {
            game: GameRunnerWidget::new(id),
        }
    }

    fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, store: &DataAssetStore) {
        self.game.show(ui, wc, store);
    }
}
