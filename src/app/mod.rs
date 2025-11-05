mod dialogs;
mod settings;
mod log_window;

use std::collections::HashMap;
use crate::data_asset::{DataAssetType, DataAssetId, DataAssetStore, StringLogger};
use crate::image_table::IMAGES;
use crate::asset_defs::ASSET_DEFS;
use crate::include_ref_image;
use crate::editors::{
    DataAssetEditor, TilesetEditor, MapDataEditor, RoomEditor,
    SpriteEditor, SpriteAnimationEditor, SfxEditor, ModDataEditor,
    FontEditor, PropFontEditor
};

const MENU_HEIGHT: f32 = 22.0;
const ASSET_TREE_PANEL_WIDTH: f32 = 200.0;

struct AppDialogs {
    about_open: bool,
    message_box_open: bool,
    message_box_title: String,
    message_box_text: String,
}

impl AppDialogs {

    fn new() -> Self {
        AppDialogs {
            about_open: false,
            message_box_open: false,
            message_box_title: "".to_owned(),
            message_box_text: "".to_owned(),
        }
    }
    
    fn open_message_box(&mut self, title: impl AsRef<str>, text: impl AsRef<str>) {
        self.message_box_open = true;
        self.message_box_text = text.as_ref().to_owned();
        self.message_box_title = title.as_ref().to_owned();
    }

    fn open_about(&mut self) {
        self.about_open = true;
    }
    
    fn show_about(&mut self, ctx: &egui::Context) {
        if dialogs::show_about_dialog(ctx).should_close() {
            self.about_open = false;
        }
    }

    fn show_message_box(&mut self, ctx: &egui::Context) {
        if dialogs::show_message_box(ctx, &self.message_box_title, &self.message_box_text).should_close() {
            self.message_box_open = false;
        }
    }
    
}

struct AppWindows {
    settings_open: bool,
    log_window_open: bool,
}

impl AppWindows {

    fn new() -> Self {
        AppWindows {
            settings_open: false,
            log_window_open: false,
        }
    }
    
    fn show_settings(&mut self, ctx: &egui::Context, window_space: egui::Rect) {
        settings::show_editor_settings(ctx, window_space, &mut self.settings_open);
    }
    
    fn show_log_window(&mut self, ctx: &egui::Context, window_space: egui::Rect, log_text: &String) {
        log_window::show_log_window(ctx, window_space, &mut self.log_window_open, log_text);
    }
    
}

struct AssetEditors {
    tilesets: HashMap<DataAssetId, TilesetEditor>,
    maps: HashMap<DataAssetId, MapDataEditor>,
    rooms: HashMap<DataAssetId, RoomEditor>,
    sprites: HashMap<DataAssetId, SpriteEditor>,
    animations: HashMap<DataAssetId, SpriteAnimationEditor>,
    sfxs: HashMap<DataAssetId, SfxEditor>,
    mods: HashMap<DataAssetId, ModDataEditor>,
    fonts: HashMap<DataAssetId, FontEditor>,
    prop_fonts: HashMap<DataAssetId, PropFontEditor>,
}

impl AssetEditors {

    fn new() -> Self {
        AssetEditors {
            tilesets: HashMap::new(),
            maps: HashMap::new(),
            rooms: HashMap::new(),
            sprites: HashMap::new(),
            animations: HashMap::new(),
            sfxs: HashMap::new(),
            mods: HashMap::new(),
            fonts: HashMap::new(),
            prop_fonts: HashMap::new(),
        }
    }

    fn clear(&mut self) {
        self.tilesets = HashMap::new();
        self.maps = HashMap::new();
        self.rooms = HashMap::new();
        self.sprites = HashMap::new();
        self.animations = HashMap::new();
        self.sfxs = HashMap::new();
        self.mods = HashMap::new();
        self.fonts = HashMap::new();
        self.prop_fonts = HashMap::new();
    }

    fn create_editors_for_new_store(&mut self, store: &DataAssetStore) {
        for &id in store.asset_ids.tilesets.iter() { self.add_tileset(id); }
        for &id in store.asset_ids.maps.iter() { self.add_map(id); }
        for &id in store.asset_ids.rooms.iter() { self.add_room(id); }
        for &id in store.asset_ids.sprites.iter() { self.add_sprite(id); }
        for &id in store.asset_ids.animations.iter() { self.add_animation(id); }
        for &id in store.asset_ids.sfxs.iter() { self.add_sfx(id); }
        for &id in store.asset_ids.mods.iter() { self.add_mod(id); }
        for &id in store.asset_ids.fonts.iter() { self.add_font(id); }
        for &id in store.asset_ids.prop_fonts.iter() { self.add_prop_font(id); }
    }
    
    fn get_editor(&self, id: DataAssetId) -> Option<&DataAssetEditor> {
        if let Some(editor) = self.tilesets.get(&id) { return Some(&editor.asset); }
        if let Some(editor) = self.maps.get(&id) { return Some(&editor.asset); }
        if let Some(editor) = self.rooms.get(&id) { return Some(&editor.asset); }
        if let Some(editor) = self.sprites.get(&id) { return Some(&editor.asset); }
        if let Some(editor) = self.animations.get(&id) { return Some(&editor.asset); }
        if let Some(editor) = self.sfxs.get(&id) { return Some(&editor.asset); }
        if let Some(editor) = self.mods.get(&id) { return Some(&editor.asset); }
        if let Some(editor) = self.fonts.get(&id) { return Some(&editor.asset); }
        if let Some(editor) = self.prop_fonts.get(&id) { return Some(&editor.asset); }
        None
    }
    
    fn get_editor_mut(&mut self, id: DataAssetId) -> Option<&mut DataAssetEditor> {
        if let Some(editor) = self.tilesets.get_mut(&id) { return Some(&mut editor.asset); }
        if let Some(editor) = self.maps.get_mut(&id) { return Some(&mut editor.asset); }
        if let Some(editor) = self.rooms.get_mut(&id) { return Some(&mut editor.asset); }
        if let Some(editor) = self.sprites.get_mut(&id) { return Some(&mut editor.asset); }
        if let Some(editor) = self.animations.get_mut(&id) { return Some(&mut editor.asset); }
        if let Some(editor) = self.sfxs.get_mut(&id) { return Some(&mut editor.asset); }
        if let Some(editor) = self.mods.get_mut(&id) { return Some(&mut editor.asset); }
        if let Some(editor) = self.fonts.get_mut(&id) { return Some(&mut editor.asset); }
        if let Some(editor) = self.prop_fonts.get_mut(&id) { return Some(&mut editor.asset); }
        None
    }

    fn remove_editor(&mut self, id: DataAssetId) -> bool {
        if self.tilesets.remove(&id).is_some() { return true; }
        if self.maps.remove(&id).is_some() { return true; }
        if self.rooms.remove(&id).is_some() { return true; }
        if self.sprites.remove(&id).is_some() { return true; }
        if self.animations.remove(&id).is_some() { return true; }
        if self.sfxs.remove(&id).is_some() { return true; }
        if self.mods.remove(&id).is_some() { return true; }
        if self.fonts.remove(&id).is_some() { return true; }
        if self.prop_fonts.remove(&id).is_some() { return true; }
        false
    }
    
    fn add_tileset(&mut self, id: DataAssetId) {
        self.tilesets.insert(id, TilesetEditor::new(id, false));
    }
    
    fn add_map(&mut self, id: DataAssetId) {
        self.maps.insert(id, MapDataEditor::new(id, false));
    }
    
    fn add_room(&mut self, id: DataAssetId) {
        self.rooms.insert(id, RoomEditor::new(id, false));
    }
    
    fn add_sprite(&mut self, id: DataAssetId) {
        self.sprites.insert(id, SpriteEditor::new(id, false));
    }
    
    fn add_animation(&mut self, id: DataAssetId) {
        self.animations.insert(id, SpriteAnimationEditor::new(id, false));
    }
    
    fn add_sfx(&mut self, id: DataAssetId) {
        self.sfxs.insert(id, SfxEditor::new(id, false));
    }
    
    fn add_mod(&mut self, id: DataAssetId) {
        self.mods.insert(id, ModDataEditor::new(id, false));
    }
    
    fn add_font(&mut self, id: DataAssetId) {
        self.fonts.insert(id, FontEditor::new(id, false));
    }
    
    fn add_prop_font(&mut self, id: DataAssetId) {
        self.prop_fonts.insert(id, PropFontEditor::new(id, false));
    }
    
}

pub struct RavenEditorApp {
    store: DataAssetStore,
    logger: StringLogger,
    dialogs: AppDialogs,
    windows: AppWindows,
    editors: AssetEditors,
}

impl RavenEditorApp {

    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        RavenEditorApp {
            store: DataAssetStore::new(),
            logger: StringLogger::new(false),
            dialogs: AppDialogs::new(),
            windows: AppWindows::new(),
            editors: AssetEditors::new(),
        }
    }

    fn set_store(&mut self, store: DataAssetStore) {
        self.editors.clear();
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
}

impl eframe::App for RavenEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        // ============================================
        // DIALOGS
        if self.dialogs.about_open {
            self.dialogs.show_about(ctx);
        }

        if self.dialogs.message_box_open {
            self.dialogs.show_message_box(ctx);
        }

        // ============================================
        // MAIN MENU
        egui::TopBottomPanel::top("main_menu").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(22.0);
                        if ui.button("Open...").clicked() && let Some(path) = rfd::FileDialog::new().pick_file() {
                            let mut store = crate::data_asset::DataAssetStore::new();
                            match crate::data_asset::reader::read_project(&path, &mut store, &mut self.logger) {
                                Ok(()) => {
                                    self.set_store(store);
                                },
                                Err(_) => {
                                    self.dialogs.open_message_box("Error Reading Project",
                                                                  "Error reading project.\n\nConsult the log window for details.");
                                    self.windows.log_window_open = true;
                                }
                            }
                        }
                    });
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.properties).max_width(14.0).max_height(14.0));
                        if ui.button("Settings...").clicked() {
                            self.windows.settings_open = true;
                        }
                    });
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.chicken).max_width(14.0).max_height(14.0));
                        if ui.button("Quit").clicked() {
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                });
                ui.menu_button("Project", |ui| {
                    for asset_def in ASSET_DEFS {
                        ui.horizontal(|ui| {
                            ui.add(egui::Image::new(include_ref_image!(asset_def.image)).max_width(14.0).max_height(14.0));
                            if ui.button(asset_def.add_menu_item).clicked() {
                                self.add_asset(asset_def.asset_type);
                            }
                        });
                    }
                    ui.separator();
                    ui.horizontal(|ui| {
                        //ui.add(egui::Image::new(IMAGES.properties).max_width(14.0).max_height(14.0));
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

        // ============================================
        // ASSET TREE
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
                                ui.add(egui::Image::new(include_ref_image!(asset_def.image)).max_width(14.0).max_height(14.0));
                                if ui.button(asset_def.add_menu_item).clicked() {
                                    add_asset = true;
                                }
                            });
                        });
                    }).body(|ui| {
                        for &id in self.store.asset_ids.ids_of_type(asset_def.asset_type) {
                            if let Some(asset) = self.store.assets.get_asset_mut(id) {
                                ui.horizontal(|ui| {
                                    ui.add(egui::Image::new(include_ref_image!(asset_def.image)).max_width(20.0).max_height(20.0));
                                    let button = ui.button(&asset.name);
                                    if button.clicked() {
                                        toggle_open = Some(id);
                                    }
                                    egui::Popup::context_menu(&button).show(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.add(egui::Image::new(include_ref_image!(asset_def.image)).max_width(14.0).max_height(14.0));
                                            if ui.button(asset_def.add_menu_item).clicked() {
                                                add_asset = true;
                                            }
                                        });
                                        ui.separator();
                                        ui.horizontal(|ui| {
                                            ui.add_space(22.0);
                                            if ui.button(asset_def.remove_menu_item).clicked() {
                                                remove_asset = Some(id)
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
        
        // ============================================
        // WINDOWS
        egui::CentralPanel::default().show(ctx, |_ui| {
            // big empty space where project windows will hover
        });
        let content_rect = ctx.content_rect();
        let window_space = egui::Rect {
            min: egui::Pos2 {
                x: content_rect.min.x + ASSET_TREE_PANEL_WIDTH,
                y: content_rect.min.y + MENU_HEIGHT,
            },
            max: content_rect.max,
        };

        for tileset in self.store.assets.tilesets.iter_mut() {
            if let Some(editor) = self.editors.tilesets.get_mut(&tileset.asset.id) {
                editor.show(ctx, window_space, tileset);
            }
        }
        for map in self.store.assets.maps.iter_mut() {
            if let Some(editor) = self.editors.maps.get_mut(&map.asset.id) {
                editor.show(ctx, window_space, map, &self.store.asset_ids.tilesets, &self.store.assets.tilesets);
            }
        }
        for room in self.store.assets.rooms.iter_mut() {
            if let Some(editor) = self.editors.rooms.get_mut(&room.asset.id) {
                editor.show(ctx, window_space, room, &self.store.asset_ids, &self.store.assets.maps, &self.store.assets.animations);
            }
        }
        for sprite in self.store.assets.sprites.iter_mut() {
            if let Some(editor) = self.editors.sprites.get_mut(&sprite.asset.id) {
                editor.show(ctx, window_space, sprite);
            }
        }
        for anim in self.store.assets.animations.iter_mut() {
            if let Some(editor) = self.editors.animations.get_mut(&anim.asset.id) {
                editor.show(ctx, window_space, anim);
            }
        }
        for sfx in self.store.assets.sfxs.iter_mut() {
            if let Some(editor) = self.editors.sfxs.get_mut(&sfx.asset.id) {
                editor.show(ctx, window_space, sfx);
            }
        }
        for mod_data in self.store.assets.mods.iter_mut() {
            if let Some(editor) = self.editors.mods.get_mut(&mod_data.asset.id) {
                editor.show(ctx, window_space, mod_data);
            }
        }
        for font in self.store.assets.fonts.iter_mut() {
            if let Some(editor) = self.editors.fonts.get_mut(&font.asset.id) {
                editor.show(ctx, window_space, font);
            }
        }
        for pfont in self.store.assets.prop_fonts.iter_mut() {
            if let Some(editor) = self.editors.prop_fonts.get_mut(&pfont.asset.id) {
                editor.show(ctx, window_space, pfont);
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
