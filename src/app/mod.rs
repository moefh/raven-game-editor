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

const MENU_HEIGHT: f32 = 22.0;
const FOOTER_HEIGHT: f32 = 24.0;
const ASSET_TREE_PANEL_WIDTH: f32 = 200.0;

const IMAGE_MENU_SIZE: f32 = 14.0;
const IMAGE_TREE_SIZE: f32 = 20.0;

pub struct RavenEditorApp {
    store: DataAssetStore,
    logger: StringLogger,
    dialogs: dialogs::AppDialogs,
    windows: windows::AppWindows,
    editors: editors::AssetEditors,
    tex_manager: TextureManager,
    sound_player: SoundPlayer,
}

impl RavenEditorApp {

    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        RavenEditorApp {
            store: DataAssetStore::new(),
            logger: StringLogger::new(false),
            dialogs: dialogs::AppDialogs::new(),
            windows: windows::AppWindows::new(),
            editors: editors::AssetEditors::new(),
            tex_manager: TextureManager::new(),
            sound_player: SoundPlayer::new(),
        }
    }

    pub fn from_file<P: AsRef<std::path::Path> + ?Sized>(cc: &eframe::CreationContext<'_>, path: &P) -> Self {
        let mut app = Self::new(cc);
        app.open(&path);
        app
    }

    pub fn open<P: AsRef<std::path::Path> + ?Sized>(&mut self, path: &P) {
        let mut store = crate::data_asset::DataAssetStore::new();
        match crate::data_asset::reader::read_project(path.as_ref(), &mut store, &mut self.logger) {
            Ok(()) => {
                self.set_store(store);
            },
            Err(_) => {
                self.dialogs.open_message_box("Error Reading Project",
                                              "Error reading project.\n\nConsult the log window for details.");
                self.windows.log_window_open = true;  // show log with detailed error
            }
        }
    }

    fn set_store(&mut self, store: DataAssetStore) {
        self.editors.clear();
        self.tex_manager.clear();
        self.store = store;
        self.editors.create_editors_for_new_store(&self.store);
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
            self.dialogs.open_message_box("Editor Open", "This asset is open for editing.\n\nClose the editor to delete it.");
        } else if self.store.assets.asset_has_dependents(id) {
            self.dialogs.open_message_box("Asset Has Dependents", "This asset is being used.");
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
                    self.dialogs.open_message_box("No Tileset Available", "You must create a tileset first!");
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
                    self.dialogs.open_message_box("No Sprite Available", "You must create a sprite first!");
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

    fn update_dialogs(&mut self, ctx: &egui::Context) {
        if self.dialogs.about_open {
            self.dialogs.show_about(ctx);
        }

        if self.dialogs.message_box_open {
            self.dialogs.show_message_box(ctx);
        }
    }

    fn update_menu(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("main_menu").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.open).max_size(egui::Vec2::splat(IMAGE_MENU_SIZE)));
                        if ui.button("Open...").clicked() && let Some(path) = rfd::FileDialog::new().pick_file() {
                            self.open(&path);
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
                        //ui.add(egui::Image::new(IMAGES.properties).max_size(egui::Vec2::splat(IMAGE_MENU_SIZE)));
                        ui.add_space(22.0);
                        if ui.button("Log Window").clicked() {
                            self.windows.log_window_open = true;
                        }
                    });
                });
                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.dialogs.open_about();
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
                                            ui.add_space(22.0);
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
        let mut win_ctx = WindowContext::new(window_space, ctx, &mut self.tex_manager);

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
            if let Some(editor) = self.editors.rooms.get_mut(&room.asset.id) {
                editor.show(&win_ctx, room, &self.store.asset_ids, &self.store.assets.maps, &self.store.assets.animations);
            }
        }
        for sprite in self.store.assets.sprites.iter_mut() {
            if let Some(editor) = self.editors.sprites.get_mut(&sprite.asset.id) {
                editor.show(&mut win_ctx, sprite);
            }
        }
        for anim in self.store.assets.animations.iter_mut() {
            if let Some(editor) = self.editors.animations.get_mut(&anim.asset.id) {
                editor.show(&win_ctx, anim);
            }
        }
        for sfx in self.store.assets.sfxs.iter_mut() {
            if let Some(editor) = self.editors.sfxs.get_mut(&sfx.asset.id) {
                editor.show(&win_ctx, sfx, &mut self.sound_player);
            }
        }
        for mod_data in self.store.assets.mods.iter_mut() {
            if let Some(editor) = self.editors.mods.get_mut(&mod_data.asset.id) {
                editor.show(&win_ctx, mod_data);
            }
        }
        for font in self.store.assets.fonts.iter_mut() {
            if let Some(editor) = self.editors.fonts.get_mut(&font.asset.id) {
                editor.show(&win_ctx, font);
            }
        }
        for pfont in self.store.assets.prop_fonts.iter_mut() {
            if let Some(editor) = self.editors.prop_fonts.get_mut(&pfont.asset.id) {
                editor.show(&win_ctx, pfont);
            }
        }

        if self.windows.settings_open {
            self.windows.show_settings(ctx, window_space);
        }
        if self.windows.log_window_open {
            self.windows.show_log_window(ctx, window_space, self.logger.modify());
        }
    }
}

impl eframe::App for RavenEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_dialogs(ctx);
        self.update_menu(ctx);
        self.update_footer(ctx);
        self.update_asset_tree(ctx);
        self.update_windows(ctx);
    }
}
