use crate::platform::GamepadManager;
use crate::platform::gamepad;
use crate::platform::gamepad_buttons::{*};

use super::{
    create_dialog_window,
    SysDialogs,
    AppWindowTracker,
    AppSettings,
};

const BUTTON_LIST: &[u32] = &[
    GAMEPAD_UP,
    GAMEPAD_DOWN,
    GAMEPAD_LEFT,
    GAMEPAD_RIGHT,
    GAMEPAD_SNES_B,
    GAMEPAD_SNES_A,
    GAMEPAD_SNES_Y,
    GAMEPAD_SNES_X,
    GAMEPAD_SELECT,
    GAMEPAD_START,
    GAMEPAD_LB,
    GAMEPAD_RB,
    GAMEPAD_HOME,
];

#[derive(Clone, Copy, PartialEq)]
enum GamepadLayout {
    Playstation,
    Snes,
    Xbox,
}

impl GamepadLayout {
    fn text(self) -> &'static str {
        match self {
            GamepadLayout::Playstation => { "Playstation" }
            GamepadLayout::Snes        => { "Super Nintendo" }
            GamepadLayout::Xbox        => { "XBox" }
        }
    }

    fn get_layout_button_name(self, button: u32) -> Option<&'static str> {
        match self {
            GamepadLayout::Playstation => { match button {
                GAMEPAD_PS_TRIANGLE => { Some("Triangle") }
                GAMEPAD_PS_SQUARE => { Some("Square") }
                GAMEPAD_PS_X => { Some("X") }
                GAMEPAD_PS_CIRCLE => { Some("Circle") }
                _ => { None }
            }}
            GamepadLayout::Snes => { match button {
                GAMEPAD_SNES_A => { Some("A") }
                GAMEPAD_SNES_B => { Some("B") }
                GAMEPAD_SNES_X => { Some("X") }
                GAMEPAD_SNES_Y => { Some("Y") }
                _ => { None }
            }}
            GamepadLayout::Xbox => { match button {
                GAMEPAD_XBOX_A => { Some("A") }
                GAMEPAD_XBOX_B => { Some("B") }
                GAMEPAD_XBOX_X => { Some("X") }
                GAMEPAD_XBOX_Y => { Some("Y") }
                _ => { None }
            }}
        }
    }

    fn get_button_name(self, button: u32) -> &'static str {
        match button {
            GAMEPAD_UP     => { "Up:" }
            GAMEPAD_DOWN   => { "Down:" }
            GAMEPAD_LEFT   => { "Left:" }
            GAMEPAD_RIGHT  => { "Right:" }
            GAMEPAD_LB     => { "Left bumper:" }
            GAMEPAD_LT     => { "Left trigger:" }
            GAMEPAD_L3     => { "Left stick:" }
            GAMEPAD_RB     => { "Right bumper:" }
            GAMEPAD_RT     => { "Right trigger:" }
            GAMEPAD_R3     => { "Right stick:" }
            GAMEPAD_SELECT => { "Select:" }
            GAMEPAD_START  => { "Start:" }
            GAMEPAD_HOME   => { "Home:" }

            GAMEPAD_SNES_A | GAMEPAD_SNES_B | GAMEPAD_SNES_X | GAMEPAD_SNES_Y => {
                self.get_layout_button_name(button).unwrap_or("?")
            }

            _ => { "?" }
        }
    }
}

pub struct GamepadMappingDialog {
    id: egui::Id,
    id_layout_combo: egui::Id,
    id_buttons_scroll: egui::Id,
    open: bool,
    display_layout: GamepadLayout,
    cur_mapping: gamepad::Mapping,
    cur_config_button: u32,
    auto_advance_button: bool,
}

impl GamepadMappingDialog {
    const WINDOW_WIDTH: f32 = 450.0;
    const BUTTONS_LIST_HEIGHT: f32 = 150.0;

    pub fn new() -> Self {
        GamepadMappingDialog {
            id: egui::Id::new("dlg_gamepad_mapping_window"),
            id_layout_combo: egui::Id::new("dlg_gamepad_mapping_layout_combo"),
            id_buttons_scroll: egui::Id::new("dlg_gamepad_mapping_buttons_scroll"),
            open: false,
            display_layout: GamepadLayout::Playstation,
            cur_config_button: 0,
            auto_advance_button: true,
            cur_mapping: gamepad::Mapping::new(),
        }
    }

    pub fn set_open(&mut self, wt: &mut AppWindowTracker) {
        self.cur_config_button = 0;
        self.cur_mapping.clear();
        self.open = true;
        wt.set_dialog_open(self.id, self.open);
    }

    fn create_gamepad_mapping(&mut self, gpman: &mut GamepadManager, settings: &mut AppSettings) {
        if let Some(gamepad) = gpman.active_gamepad() {
            settings.gamepad_mappings.insert(gamepad.id.clone(), std::mem::replace(&mut self.cur_mapping, gamepad::Mapping::new()));
        }
    }

    fn show_gamepad_mapping_editor(&mut self, ui: &mut egui::Ui, gpman: &mut GamepadManager) {
        ui.request_repaint();
        let mut jumping_to_next_button = false;
        let events = gpman.get_raw_events();
        for ev in events {
            if self.cur_config_button != 0 && ev.add_to_mapping(self.cur_config_button, &mut self.cur_mapping) {
                if self.auto_advance_button {
                    self.cur_config_button = BUTTON_LIST
                        .iter()
                        .position(|&b| b == self.cur_config_button)
                        .and_then(|index| {
                            BUTTON_LIST.get(index+1)
                        })
                        .copied()
                        .unwrap_or(0);
                    if self.cur_config_button != 0 {
                        jumping_to_next_button = true;
                    }
                } else {
                    self.cur_config_button = 0;
                }
            }
        }

        egui::Grid::new("dlg_gamepad_mapping_buttons_grid")
            .num_columns(2)
            .spacing([8.0, 8.0])
            .show(ui, |ui| {
                ui.label("Detected gamepad:");
                if let Some(gamepad) = gpman.active_gamepad() {
                    ui.add(egui::Label::new(&gamepad.id).truncate());
                } else {
                    ui.label("(none)");
                }
                ui.end_row();

                ui.label("Button names for:");
                egui::ComboBox::from_id_salt(self.id_layout_combo)
                    .selected_text(self.display_layout.text())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.display_layout,
                            GamepadLayout::Playstation,
                            GamepadLayout::Playstation.text()
                        );
                        ui.selectable_value(
                            &mut self.display_layout,
                            GamepadLayout::Snes,
                            GamepadLayout::Snes.text()
                        );
                        ui.selectable_value(
                            &mut self.display_layout,
                            GamepadLayout::Xbox,
                            GamepadLayout::Xbox.text()
                        );
                    });
                ui.end_row();

                ui.label("Auto advance button:");
                ui.checkbox(&mut self.auto_advance_button, "");
                ui.end_row();
            });

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label("Button mappings:");
        });
        ui.add_space(4.0);
        egui::ScrollArea::vertical().id_salt(self.id_buttons_scroll)
            .max_height(Self::BUTTONS_LIST_HEIGHT)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Grid::new("dlg_gamepad_mapping_buttons_grid")
                    .num_columns(3)
                    .spacing([4.0, 4.0])
                    .show(ui, |ui| {
                        for &button in BUTTON_LIST {
                            ui.label("");
                            ui.label(self.display_layout.get_button_name(button));
                            let start_config = if button == self.cur_config_button {
                                ui.horizontal(|ui| {
                                    let response = ui.label("PRESS GAMEPAD BUTTON");
                                    if jumping_to_next_button {
                                        response.scroll_to_me(Some(egui::Align::Center));
                                    }
                                    if ui.button("Cancel").clicked() {
                                        self.cur_config_button = 0;
                                    }
                                });
                                false
                            } else if self.cur_mapping.is_button_set(button) {
                                ui.button("re-configure").clicked()
                            } else {
                                ui.button("configure").clicked()
                            };
                            ui.end_row();
                            if start_config {
                                self.cur_config_button = button;
                            }
                        }
                    });
            });
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        wt: &mut AppWindowTracker,
        sys_dialogs: &SysDialogs,
        settings: &mut AppSettings,
        gpman: &mut GamepadManager
    ) {
        if ! self.open { return; }

        if create_dialog_window(sys_dialogs, ui, self.id, Self::WINDOW_WIDTH, "Gamepad Configuration", |ui| {
            egui::Frame::NONE.inner_margin(4.0).show(ui, |ui| {
                self.show_gamepad_mapping_editor(ui, gpman);
                ui.add_space(5.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui.add_enabled(gpman.active_gamepad().is_some(), egui::Button::new("Create")).clicked() {
                        self.create_gamepad_mapping(gpman, settings);
                        ui.close();
                    }
                    if ui.button("Close").clicked() {
                        ui.close();
                    }
                });
            });
        }).should_close() {
            self.open = false;
            wt.set_dialog_open(self.id, self.open);
        }
    }
}
