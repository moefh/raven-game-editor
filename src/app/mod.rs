mod dialogs;
mod windows;
mod editors;

use crate::include_ref_image;
use crate::data_asset::{DataAssetType, DataAssetId, DataAssetStore, StringLogger};
use crate::misc::asset_defs::ASSET_DEFS;
use crate::misc::IMAGES;
use crate::misc::WindowContext;
use crate::misc::TextureManager;
use crate::misc::SoundPlayer;

pub use dialogs::{AppDialogs, ConfirmationDialogResult};
pub use windows::AppWindows;

const MENU_HEIGHT: f32 = 22.0;
const FOOTER_HEIGHT: f32 = 26.0;
const ASSET_TREE_PANEL_WIDTH: f32 = 200.0;

pub const IMAGE_MENU_SIZE: f32 = 14.0;
pub const NO_IMAGE_TREE_SIZE: f32 = 25.0;
pub const IMAGE_TREE_SIZE: f32 = 20.0;

#[derive(Clone, Copy)]
enum ConfirmationDialogAction {
    None,
    NewProject,
}

pub struct RavenEditorApp {
    reset_egui_context: bool,
    store: DataAssetStore,
    filename: Option<std::path::PathBuf>,
    logger: StringLogger,
    dialogs: AppDialogs,
    windows: AppWindows,
    editors: editors::AssetEditors,
    tex_manager: TextureManager,
    sound_player: SoundPlayer,
    confirmation_dialog_action: ConfirmationDialogAction,
}

impl RavenEditorApp {

    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let app = RavenEditorApp {
            reset_egui_context: false,
            store: DataAssetStore::new(),
            filename: None,
            logger: StringLogger::new(false),
            dialogs: dialogs::AppDialogs::new(),
            windows: windows::AppWindows::new(),
            editors: editors::AssetEditors::new(),
            tex_manager: TextureManager::new(),
            sound_player: SoundPlayer::new(),
            confirmation_dialog_action: ConfirmationDialogAction::None,
        };
        app.setup_egui_context(&cc.egui_ctx);
        app
    }

    pub fn from_file<P: AsRef<std::path::Path>>(cc: &eframe::CreationContext<'_>, path: P) -> Self {
        let mut app = Self::new(cc);
        app.open(&path);
        app
    }

    pub fn setup_egui_context(&self, ctx: &egui::Context) {
        egui_extras::install_image_loaders(ctx);
        crate::add_font(ctx);
        ctx.set_zoom_factor(1.5);
        ctx.options_mut(|opt: &mut egui::Options| {
            opt.zoom_with_keyboard = false;
        });
        ctx.set_theme(egui::ThemePreference::Light);
    }

    pub fn open<P: AsRef<std::path::Path>>(&mut self, path: P) {
        let mut store = crate::data_asset::DataAssetStore::new();
        match crate::data_asset::reader::read_project(path.as_ref(), &mut store, &mut self.logger) {
            Ok(()) => {
                self.load_project(store);
                self.filename = Some(path.as_ref().to_path_buf());
            },
            Err(_) => {
                self.open_message_box("Error Reading Project",
                                      "Error reading project.\n\nConsult the log window for details.");
                self.windows.log_window_open = true;
            }
        }
    }

    fn write_project(&mut self, path: &std::path::Path) -> bool {
        match crate::data_asset::writer::write_project(path, &self.store, &mut self.logger) {
            Ok(()) => true,
            Err(_) => {
                self.open_message_box("Error Writing Project",
                                      "Error writing project.\n\nConsult the log window for details.");
                self.windows.log_window_open = true;
                false
            }
        }
    }

    pub fn save_as(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Save Project As")
            .add_filter("Raven project files (*.h)", &["h"])
            .add_filter("All files (*)", &[""])
            .save_file() &&
            self.write_project(&path) {
                self.filename = Some(path.to_path_buf());
            }
    }

    pub fn save(&mut self) {
        match &self.filename {
            Some(p) => { self.write_project(&p.clone()); }
            None => { self.save_as(); }
        }
    }

    fn load_project(&mut self, store: DataAssetStore) {
        self.editors.clear();
        self.tex_manager.clear();
        self.store = store;
        self.editors.create_editors_for_new_store(&self.store);
        self.reset_egui_context = true;
    }

    fn new_asset_name(&self, asset_type: DataAssetType) -> String {
        if let Some(prefix) = ASSET_DEFS.iter().find(|def| def.asset_type == asset_type).map(|def| def.default_name_prefix) {
            let mut num = 1;
            loop {
                let name = format!("{}{}", prefix, num);
                if ! self.store.asset_ids
                    .ids_of_type(asset_type)
                    .any(|&id| self.store.assets.get_asset(id).is_some_and(|a| a.name == name)) {
                        return name;
                    }
                num += 1;
            }
        }
        format!("{:?}", asset_type)
    }

    fn remove_asset(&mut self, id: DataAssetId) {
        if let Some(editor) = self.editors.get_editor(id) && editor.open {
            self.open_message_box("Editor Open", "This asset is open for editing.\n\nClose the editor to delete it.");
        } else if self.store.assets.asset_has_dependents(id) {
            self.open_message_box("Asset Has Dependents", "This asset is being used.");
        } else {
            self.store.remove_asset(id);
            self.editors.remove_editor(id);
        }
    }

    fn add_asset(&mut self, asset_type: DataAssetType) {
        match asset_type {
            DataAssetType::Tileset => {
                if let Some(id) = self.store.add_tileset(self.new_asset_name(asset_type)) {
                    self.editors.add_tileset(id);
                }
            },
            DataAssetType::MapData => {
                if let Some(tileset_id) = self.store.asset_ids.tilesets.get_first() {
                    if let Some(id) = self.store.add_map(self.new_asset_name(asset_type), tileset_id) {
                        self.editors.add_map(id);
                    }
                } else {
                    self.open_message_box("No Tileset Available", "You must create a tileset first!");
                }
            },
            DataAssetType::Room => {
                if let Some(id) = self.store.add_room(self.new_asset_name(asset_type)) {
                    self.editors.add_room(id);
                }
            },
            DataAssetType::Sprite => {
                if let Some(id) = self.store.add_sprite(self.new_asset_name(asset_type)) {
                    self.editors.add_sprite(id);
                }
            },
            DataAssetType::SpriteAnimation => {
                if let Some(sprite_id) = self.store.asset_ids.sprites.get_first() {
                    if let Some(id) = self.store.add_animation(self.new_asset_name(asset_type), sprite_id) {
                        self.editors.add_animation(id);
                    }
                } else {
                    self.open_message_box("No Sprite Available", "You must create a sprite first!");
                }
            },
            DataAssetType::Sfx => {
                if let Some(id) = self.store.add_sfx(self.new_asset_name(asset_type)) {
                    self.editors.add_sfx(id);
                }
            },
            DataAssetType::ModData => {
                if let Some(id) = self.store.add_mod(self.new_asset_name(asset_type)) {
                    self.editors.add_mod(id);
                }
            },
            DataAssetType::Font => {
                if let Some(id) = self.store.add_font(self.new_asset_name(asset_type)) {
                    self.editors.add_font(id);
                }
            },
            DataAssetType::PropFont => {
                if let Some(id) = self.store.add_prop_font(self.new_asset_name(asset_type)) {
                    self.editors.add_prop_font(id);
                }
            },
        }
    }

    fn open_message_box(&mut self, title: &str, text: &str) {
        self.dialogs.open_message_box(title, text);
    }

    fn open_confirmation_dialog_for(&mut self, action: ConfirmationDialogAction) {
        self.confirmation_dialog_action = action;
        match action {
            ConfirmationDialogAction::NewProject => {
                self.dialogs.open_confirmation_dialog("New Project", "Close current project and start a new one?", "Yes", "No");
            }
            ConfirmationDialogAction::None => {}
        };
    }

    fn open_about_dialog(&mut self) {
        self.dialogs.open_about();
    }

    fn update_dialogs(&mut self, ctx: &egui::Context) {
        if self.dialogs.about_open {
            self.dialogs.show_about(ctx);
        }

        if self.dialogs.message_box_open {
            self.dialogs.show_message_box(ctx);
        }

        if self.dialogs.confirmation_dialog_open &&
            matches!(self.dialogs.show_confirmation_dialog(ctx), ConfirmationDialogResult::Yes) {
                match self.confirmation_dialog_action {
                    ConfirmationDialogAction::NewProject => {
                        self.load_project(crate::data_asset::DataAssetStore::new());
                        self.filename = None;
                    }
                    ConfirmationDialogAction::None => {}
                };
                self.confirmation_dialog_action = ConfirmationDialogAction::None;
            }
    }

    fn update_menu(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("main_menu").show(ctx, |ui| {
            let file_save_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::S);
            if ui.input_mut(|i| i.consume_shortcut(&file_save_shortcut)) {
                self.save();
            }
            //let file_quit_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::Q);
            //if ui.input_mut(|i| i.consume_shortcut(&file_quit_shortcut)) {
            //    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            //}

            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.new).max_size(egui::Vec2::splat(IMAGE_MENU_SIZE)));
                        if ui.button("New").clicked() {
                            self.open_confirmation_dialog_for(ConfirmationDialogAction::NewProject);
                        }
                    });
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.open).max_size(egui::Vec2::splat(IMAGE_MENU_SIZE)));
                        if ui.button("Open...").clicked() &&
                            let Some(path) = rfd::FileDialog::new()
                            .set_title("Open Project")
                            .add_filter("Raven project files (*.h)", &["h"])
                            .add_filter("All files (*)", &[""])
                            .pick_file() {
                                self.open(&path);
                            }
                    });

                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.save).max_size(egui::Vec2::splat(IMAGE_MENU_SIZE)));
                        if ui.add(egui::Button::new("Save").shortcut_text(ui.ctx().format_shortcut(&file_save_shortcut))).clicked() {
                            self.save();
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.save).max_size(egui::Vec2::splat(IMAGE_MENU_SIZE)));
                        if ui.button("Save As...").clicked() {
                            self.save_as();
                        }
                    });
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.properties).max_size(egui::Vec2::splat(IMAGE_MENU_SIZE)));
                        if ui.button("Settings...").clicked() {
                            self.windows.settings_open = true;
                        }
                    });
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.chicken).max_size(egui::Vec2::splat(IMAGE_MENU_SIZE)));
                        if ui.button("Quit").clicked() {
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                });
                ui.menu_button("Project", |ui| {
                    for asset_def in ASSET_DEFS {
                        ui.horizontal(|ui| {
                            ui.add(egui::Image::new(include_ref_image!(asset_def.image)).max_size(egui::Vec2::splat(IMAGE_MENU_SIZE)));
                            if ui.button(asset_def.add_menu_item).clicked() {
                                self.add_asset(asset_def.asset_type);
                            }
                        });
                    }
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.properties).max_size(egui::Vec2::splat(IMAGE_MENU_SIZE)));
                        if ui.button("Properties...").clicked() {
                            self.windows.properties_open = true;
                        }
                    });
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.add_space(NO_IMAGE_TREE_SIZE);
                        if ui.button("Log Window").clicked() {
                            self.windows.log_window_open = true;
                        }
                    });
                });
                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.open_about_dialog();
                    }
                });
            });
        });
    }

    fn update_footer(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.add_space(5.0);
            ui.label(format!("{} bytes", self.store.assets.data_size()));
        });
    }

    fn update_asset_tree(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("asset_tree").resizable(false).exact_width(ASSET_TREE_PANEL_WIDTH).show(ctx, |ui| {
            egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                for asset_def in ASSET_DEFS {
                    let mut remove_asset: Option<DataAssetId> = None;
                    let mut toggle_open: Option<DataAssetId> = None;
                    let mut add_asset = false;
                    let tree_node_id = ui.make_persistent_id(asset_def.id);

                    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), tree_node_id, true).show_header(ui, |ui| {
                        let header = ui.add(egui::Label::new(asset_def.tree_root_item).selectable(false).sense(egui::Sense::click()));
                        egui::Popup::context_menu(&header).show(|ui| {
                            ui.horizontal(|ui| {
                                ui.add(egui::Image::new(include_ref_image!(asset_def.image)).max_width(16.0).max_height(16.0));
                                if ui.button(asset_def.add_menu_item).clicked() {
                                    add_asset = true;
                                }
                            });
                        });
                    }).body(|ui| {
                        for &id in self.store.asset_ids.ids_of_type(asset_def.asset_type) {
                            if let Some(asset) = self.store.assets.get_asset_mut(id) {
                                ui.horizontal(|ui| {
                                    ui.add(egui::Image::new(include_ref_image!(asset_def.image)).max_size(egui::Vec2::splat(IMAGE_TREE_SIZE)));
                                    let button = ui.button(&asset.name);
                                    if button.clicked() {
                                        toggle_open = Some(id);
                                    }
                                    egui::Popup::context_menu(&button).show(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.add(egui::Image::new(include_ref_image!(asset_def.image)).max_width(16.0).max_height(16.0));
                                            if ui.button(asset_def.add_menu_item).clicked() {
                                                add_asset = true;
                                            }
                                        });
                                        ui.separator();
                                        ui.horizontal(|ui| {
                                            ui.add_space(NO_IMAGE_TREE_SIZE);
                                            if ui.button(asset_def.remove_menu_item).clicked() {
                                                remove_asset = Some(id);
                                            }
                                        });
                                    });
                                });
                            }
                        }
                    });
                    if add_asset {
                        self.add_asset(asset_def.asset_type);
                    }
                    if let Some(toggle_open_id) = toggle_open && let Some(editor) = self.editors.get_editor_mut(toggle_open_id) {
                        editor.open = !editor.open;
                    }
                    if let Some(remove_asset_id) = remove_asset {
                        self.remove_asset(remove_asset_id);
                    }
                }
            });
        });
    }

    fn update_windows(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |_ui| {
            // big empty space where project windows will hover
        });
        let content_rect = ctx.content_rect();
        let window_space = egui::Rect {
            min: egui::Pos2 {
                x: content_rect.min.x + ASSET_TREE_PANEL_WIDTH,
                y: content_rect.min.y + MENU_HEIGHT,
            },
            max: egui::Pos2 {
                x: content_rect.max.x,
                y: content_rect.max.y - FOOTER_HEIGHT,
            },
        };
        let mut win_ctx = WindowContext::new(window_space, ctx, &mut self.tex_manager, &mut self.dialogs);

        for tileset in self.store.assets.tilesets.iter_mut() {
            if let Some(editor) = self.editors.tilesets.get_mut(&tileset.asset.id) {
                editor.show(&mut win_ctx, tileset);
            }
        }
        for map in self.store.assets.maps.iter_mut() {
            if let Some(editor) = self.editors.maps.get_mut(&map.asset.id) {
                editor.show(&mut win_ctx, map, &self.store.asset_ids.tilesets, &self.store.assets.tilesets);
            }
        }
        for room in self.store.assets.rooms.iter_mut() {
            let assets = crate::editors::RoomEditorAssetLists::new(
                &self.store.assets.maps,
                &self.store.assets.tilesets,
                &self.store.assets.animations,
                &self.store.assets.sprites);
            if let Some(editor) = self.editors.rooms.get_mut(&room.asset.id) {
                editor.show(&mut win_ctx, room, &self.store.asset_ids, &assets);
            }
        }
        for sprite in self.store.assets.sprites.iter_mut() {
            if let Some(editor) = self.editors.sprites.get_mut(&sprite.asset.id) {
                editor.show(&mut win_ctx, sprite);
            }
        }
        for anim in self.store.assets.animations.iter_mut() {
            if let Some(editor) = self.editors.animations.get_mut(&anim.asset.id) {
                editor.show(&mut win_ctx, anim, &self.store.asset_ids.sprites, &mut self.store.assets.sprites);
            }
        }
        for sfx in self.store.assets.sfxs.iter_mut() {
            if let Some(editor) = self.editors.sfxs.get_mut(&sfx.asset.id) {
                editor.show(&win_ctx, sfx, &mut self.sound_player);
            }
        }
        for mod_data in self.store.assets.mods.iter_mut() {
            if let Some(editor) = self.editors.mods.get_mut(&mod_data.asset.id) {
                editor.show(&win_ctx, mod_data, &mut self.sound_player);
            }
        }
        for font in self.store.assets.fonts.iter_mut() {
            if let Some(editor) = self.editors.fonts.get_mut(&font.asset.id) {
                editor.show(&mut win_ctx, font);
            }
        }
        for pfont in self.store.assets.prop_fonts.iter_mut() {
            if let Some(editor) = self.editors.prop_fonts.get_mut(&pfont.asset.id) {
                editor.show(&mut win_ctx, pfont);
            }
        }

        if self.windows.settings_open {
            self.windows.show_settings(ctx, window_space);
        }
        if self.windows.properties_open {
            self.windows.show_properties(ctx, window_space, &mut self.store.vga_sync_bits, &mut self.store.project_prefix);
        }
        if self.windows.log_window_open {
            self.windows.show_log_window(ctx, window_space, self.logger.modify());
        }
    }
}

impl eframe::App for RavenEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.reset_egui_context {
            ctx.memory_mut(|mem| {
                // is this enough?
                mem.reset_areas();
                mem.data.clear();
            });
            //ctx.memory_mut(|mem| *mem = Default::default()); // is this needed?
            //self.setup_egui_context(ctx);
            self.reset_egui_context = false;
        }
        self.update_dialogs(ctx);
        self.update_menu(ctx);
        self.update_footer(ctx);
        self.update_asset_tree(ctx);
        self.update_windows(ctx);
    }
}
