mod context;
mod sys_dialogs;
mod dialogs;
mod windows;
mod editors;
mod settings;
mod recent_projects;
mod asset_exporter;
pub mod checker;
pub mod project_comparer;
pub mod widgets;
pub mod gamepad_settings;

use std::sync::{Arc, Mutex};

use crate::include_ref_image;
use crate::platform;
use crate::data_asset::{
    self,
    DataAssetType,
    DataAssetId,
    DataAssetStore,
    StringLogger,
};
use crate::misc::asset_defs::{
    ASSET_DEFS,
    get_asset_type_display_name,
};
use crate::misc::IMAGES;
use crate::image::TextureManager;
use crate::sound::SoundPlayer;

use widgets::{
    menu_item,
    menu_item_no_image,
    menu_item_no_image_with_submenu,
};
use editors::{
    ImageClipboardData,
    MapClipboardData,
};

pub use editors::{
    EditorStore,
    EditorAction,
};
pub use context::{
    WindowContext,
    WindowEguiContext,
    AppWindowTracker,
};
pub use sys_dialogs::{
    SysDialogs,
    SysDialogResponse,
    SysDialogOpenFile,
};
pub use dialogs::{
    AppDialogs,
    DialogResult,
};
pub use windows::{
    AppWindows,
    AppWindowAction,
};
pub use settings::{
    get_setting_zoom,
    AppSettings,
};
pub use asset_exporter::AssetExporter;
pub use crate::platform::KeyboardPressed;

enum ConfirmationDialogAction {
    NewProject,
    DeleteAsset(DataAssetId),
}

enum TextInputDialogAction {
    RenameAsset(DataAssetType, DataAssetId),
    RenameAssetFolder(DataAssetType, widgets::AssetTreeNodeId),
}

pub enum AssetTreeAction {
    None,
    RenameAsset(DataAssetId),
    RemoveAsset(DataAssetId),
    DuplicateAsset(DataAssetId),
    OpenEditor(DataAssetId),
    AddAsset(widgets::AssetTreeNodeId),
    RenameFolder(widgets::AssetTreeNodeId),
}

pub struct RavenEditorApp {
    pub is_wasm: bool,
    pub wasm_editor_is_dirty: Arc<Mutex<bool>>,
    reset_egui_context: bool,
    store: DataAssetStore,
    path: Option<std::path::PathBuf>,
    path_changed: bool,
    logger: StringLogger,
    sys_dialogs: SysDialogs,
    dialogs: AppDialogs,
    windows: AppWindows,
    editors: EditorStore,
    image_clipboard: ImageClipboardData,
    map_clipboard: MapClipboardData,
    tex_manager: TextureManager,
    sound_player: SoundPlayer,
    settings: AppSettings,
    recent_projects: recent_projects::RecentProjects,
    confirmation_dialog_action: Option<ConfirmationDialogAction>,
    text_input_dialog_action: Option<TextInputDialogAction>,
    keyboard_pressed: Option<KeyboardPressed>,
    window_tracker: AppWindowTracker,
    asset_tree: widgets::StoreAssetTree,
    asset_exporter: AssetExporter,
    user_confirmed_close: bool,
}

impl RavenEditorApp {
    const OPEN_PROJECT_SYS_DLG_ID: &str = "open_project";
    const SAVE_PROJECT_SYS_DLG_ID: &str = "save_project_as";
    const EXPORT_HEADER_SYS_DLG_ID: &str = "export_header";
    const ASSET_TREE_PANEL_WIDTH: f32 = 200.0;

    pub fn new(cc: &eframe::CreationContext<'_>, is_wasm: bool, logger: StringLogger, settings: AppSettings) -> Self {
        let mut app = RavenEditorApp {
            is_wasm,
            logger,
            settings,
            reset_egui_context: false,
            store: DataAssetStore::new(),
            path: None,
            path_changed: true,
            sys_dialogs: sys_dialogs::SysDialogs::new(cc.egui_ctx.clone()),
            dialogs: dialogs::AppDialogs::new(),
            editors: EditorStore::new(),
            windows: windows::AppWindows::new(),
            image_clipboard: ImageClipboardData::Empty,
            map_clipboard: MapClipboardData::Empty,
            tex_manager: TextureManager::new(),
            sound_player: SoundPlayer::new(),
            confirmation_dialog_action: None,
            text_input_dialog_action: None,
            keyboard_pressed: None,
            window_tracker: AppWindowTracker::new(),
            asset_tree: widgets::StoreAssetTree::new(),
            recent_projects: recent_projects::RecentProjects::new(),
            asset_exporter: AssetExporter::new(),
            user_confirmed_close: false,
            wasm_editor_is_dirty: Arc::new(Mutex::new(false)),
        };
        app.window_tracker.reset(&app.editors.egui_id_to_asset_id, app.windows.get_ids());
        if ! is_wasm {
            app.recent_projects.load(&mut app.logger);
            app.sys_dialogs.load_paths(&mut app.logger);
        }
        app.logger.log(app.sound_player.init_info());
        app.setup_egui_context(&cc.egui_ctx);
        app
    }

    fn activate_window(ctx: &egui::Context, window_id: egui::Id) {
        ctx.move_to_top(egui::LayerId::new(egui::Order::Middle, window_id));
    }

    fn activate_asset_editor(&mut self, ctx: &egui::Context, asset_id: DataAssetId) {
        if let Some(editor) = self.editors.get_editor_mut(asset_id) {
            if ! editor.open {
                editor.open = true;
            } else {
                Self::activate_window(ctx, editor.egui_id);
            }
        }
    }

    pub fn setup_egui_context(&self, ctx: &egui::Context) {
        if self.settings.start_maximized {
            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
        }
        egui_extras::install_image_loaders(ctx);
        crate::add_font(ctx);
        ctx.options_mut(|opt: &mut egui::Options| {
            opt.zoom_with_keyboard = false;
        });
        ctx.style_mut_of(egui::Theme::Dark, |style: &mut egui::Style| {
            style.visuals.window_highlight_topmost = true;
            style.visuals.window_fill = egui::Color32::BLACK;
            style.visuals.window_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 0x40, 0x80));
            style.visuals.panel_fill = egui::Color32::from_rgb(0x10, 0x10, 0x10);
            style.visuals.faint_bg_color = egui::Color32::from_rgb(0x08, 0x10, 0x20);
            style.visuals.extreme_bg_color = egui::Color32::BLACK;
            style.visuals.widgets.open.weak_bg_fill = egui::Color32::from_rgb(0x10, 0x20, 0x60);
        });
        ctx.style_mut_of(egui::Theme::Light, |style: &mut egui::Style| {
            style.visuals.faint_bg_color = style.visuals.window_fill;
        });
        ctx.set_zoom_factor(get_setting_zoom(self.settings.zoom));
        match self.settings.theme.as_str() {
            "light" => ctx.set_theme(egui::ThemePreference::Light),
            "dark" => ctx.set_theme(egui::ThemePreference::Dark),
            "system" => ctx.set_theme(egui::ThemePreference::System),
            _ => {}
        }
    }

    pub fn run_project_check(&mut self, ctx: &egui::Context) {
        if self.windows.collection.check.base.open {
            Self::activate_window(ctx, self.windows.collection.check.base.id);
        } else {
            self.windows.open_check();
        }
        self.windows.run_check(&self.store);
    }

    pub fn export_header(&mut self, file: SysDialogOpenFile) -> bool {
        match crate::data_asset::write_header_def(&self.store.project_prefix).and_then(|content| file.write_string(content)) {
            Ok(()) => {
                self.logger.log(format!("Exported header to {}", file.filename()));
                true
            }
            Err(e) => {
                self.logger.log(format!("ERROR writing header content to {}:\n{}", file.filename(), e));
                self.open_message_box(
                    "Error Exportint Header",
                    "Error exporting header.\n\nConsult the log window for details."
                );
                self.windows.open_log_window();
                false
            }
        }
    }

    pub fn open(&mut self, file: SysDialogOpenFile) {
        if let Some(path) = file.path() && let Some(dir) = path.parent() {
            self.sys_dialogs.set_path_for_id("project", dir);
        }
        self.logger.log(format!("READING FILE {}", file.filename()));
        match file.read_string().and_then(|content| data_asset::deserialize_project(&content, &mut self.logger)) {
            Ok(store) => {
                self.logger.log("DONE: project read");
                self.load_project(store);
                if let Some(path) = file.path() {
                    self.recent_projects.add(path);
                    self.set_path(Some(path.to_owned()));
                }
            },
            Err(e) => {
                self.logger.log(format!("ERROR: {}", e));
                self.open_message_box(
                    "Error Reading Project",
                    &format!("Error reading project: {}.\n\nConsult the log window for details.", e)
                );
                self.windows.open_log_window();
            }
        }
    }

    /// Some editors may have assets in a transient state that must
    /// be resolved before saving (e.g. images with detached
    /// floating selections).
    ///
    /// All assets will be ready for saving when this function
    /// returns.
    fn prepare_for_saving(&mut self) {
        for tileset in self.store.assets.tilesets.iter_mut() {
            if let Some(editor) = self.editors.tilesets.get_mut(&tileset.asset.id) { editor.prepare_for_saving(tileset); }
        }
        for map in self.store.assets.maps.iter_mut() {
            if let Some(editor) = self.editors.maps.get_mut(&map.asset.id) { editor.prepare_for_saving(map); }
        }
        for room in self.store.assets.rooms.iter_mut() {
            if let Some(editor) = self.editors.rooms.get_mut(&room.asset.id) { editor.prepare_for_saving(room); }
        }
        for world in self.store.assets.worlds.iter_mut() {
            if let Some(editor) = self.editors.worlds.get_mut(&world.asset.id) { editor.prepare_for_saving(world); }
        }
        for sprite in self.store.assets.sprites.iter_mut() {
            if let Some(editor) = self.editors.sprites.get_mut(&sprite.asset.id) { editor.prepare_for_saving(sprite); }
        }
        for pal_sprite in self.store.assets.pal_sprites.iter_mut() {
            if let Some(editor) = self.editors.pal_sprites.get_mut(&pal_sprite.asset.id) { editor.prepare_for_saving(pal_sprite); }
        }
        for anim in self.store.assets.animations.iter_mut() {
            if let Some(editor) = self.editors.animations.get_mut(&anim.asset.id) {
                editor.prepare_for_saving(anim, &mut self.store.assets.sprites);
            }
        }
        for tanim in self.store.assets.tile_anims.iter_mut() {
            if let Some(editor) = self.editors.tile_anims.get_mut(&tanim.asset.id) {
                editor.prepare_for_saving(tanim);
            }
        }
        for sfx in self.store.assets.sfxs.iter_mut() {
            if let Some(editor) = self.editors.sfxs.get_mut(&sfx.asset.id) { editor.prepare_for_saving(sfx); }
        }
        for mod_data in self.store.assets.mods.iter_mut() {
            if let Some(editor) = self.editors.mods.get_mut(&mod_data.asset.id) { editor.prepare_for_saving(mod_data); }
        }
        for font in self.store.assets.fonts.iter_mut() {
            if let Some(editor) = self.editors.fonts.get_mut(&font.asset.id) { editor.prepare_for_saving(font); }
        }
        for pfont in self.store.assets.prop_fonts.iter_mut() {
            if let Some(editor) = self.editors.prop_fonts.get_mut(&pfont.asset.id) { editor.prepare_for_saving(pfont); }
        }
    }

    fn write_project(&mut self, file: SysDialogOpenFile) -> bool {
        self.logger.log("WRITING PROJECT");
        self.prepare_for_saving();
        match self.store.serialize_project(&mut self.logger).and_then(|content| file.write_string(content)) {
            Ok(()) => {
                self.logger.log(format!("DONE: project saved to {}", file.filename()));
                if let Some(path) = file.path() {
                    self.recent_projects.add(path);
                }
                self.editors.clear_dirty_flags(&self.store);
                true
            }
            Err(e) => {
                self.logger.log(format!("ERROR:\n{}", e));
                self.open_message_box(
                    "Error Writing Project",
                    "Error writing project.\n\nConsult the log window for details."
                );
                self.windows.open_log_window();
                false
            }
        }
    }

    pub fn save_as(&mut self, window: &eframe::Frame) {
        self.sys_dialogs.save_file(
            Some(window),
            Self::SAVE_PROJECT_SYS_DLG_ID.to_owned(),
            "project",
            if self.is_wasm { "Save Project" } else { "Save Project As" },
            &[
                ("Raven project files (*.h)", &["h"]),
                ("All files (*.*)", &["*"]),
            ]
        );
    }

    pub fn save(&mut self, window: &eframe::Frame) {
        if let Some(path) = &self.path && let Some(file) = SysDialogOpenFile::create(path) {
            self.write_project(file);
        } else {
            self.save_as(window);
        }
    }

    fn new_project(&mut self) {
        self.load_project(crate::data_asset::DataAssetStore::new());
        self.set_path(None);
    }

    fn load_project(&mut self, store: DataAssetStore) {
        self.editors.clear();
        self.tex_manager.clear();
        self.windows.clear_project();
        self.store = store;
        self.tex_manager.set_bits_per_pixel(self.store.vga_bits_per_pixel);
        self.editors.create_editors_for_new_store(&self.store);
        self.window_tracker.reset(&self.editors.egui_id_to_asset_id, self.windows.get_ids());
        self.reset_egui_context = true;
    }

    fn set_path(&mut self, path: Option<std::path::PathBuf>) {
        self.path = path;
        self.path_changed = true;
    }

    fn make_unique_asset_name(&self, asset_type: DataAssetType, name: &str, force_number: bool) -> String {
        let mut num = 1;
        let mut unique_name = String::from(name);
        if force_number {
            unique_name.push_str(&format!("{}", num));
            num += 1;
        }
        loop {
            if ! self.store.asset_ids
                .ids_of_type(asset_type)
                .any(|&id| self.store.assets.get_asset(id).is_some_and(|a| a.name == unique_name)) {
                    return unique_name;
                }
            unique_name.replace_range(.., &format!("{}{}", name, num));
            num += 1;
        }
    }

    fn new_asset_name(&self, asset_type: DataAssetType, given_prefix: Option<String>) -> String {
        let name = if let Some(prefix) = ASSET_DEFS.iter().find(|def| def.asset_type == asset_type).map(|def| def.default_name_prefix) {
            match &given_prefix {
                Some(s) => { format!("{}/new_{}", s, prefix) }
                None => { format!("new_{}", prefix) }
            }
        } else {
            format!("{:?}", asset_type)
        };
        self.make_unique_asset_name(asset_type, &name, true)
    }

    fn rename_asset_folder(&mut self, asset_type: DataAssetType, tree_node_id: widgets::AssetTreeNodeId, new_name: String) {
        if let Some(old_name) = self.asset_tree.get_node_name(asset_type, tree_node_id) {
            for &asset_id in self.store.asset_ids.ids_of_type(asset_type) {
                if let Some(asset) = self.store.assets.get_asset_mut(asset_id) && asset.name.starts_with(&old_name) {
                    asset.name.replace_range(0..old_name.len(), &new_name);
                }
            }
        }
    }

    fn request_remove_asset(&mut self, id: DataAssetId) {
        if let Some(editor) = self.editors.get_editor(id) && editor.open {
            self.open_message_box("Editor Open", "This asset is open for editing.\n\nClose the editor to delete it.");
        } else if self.store.assets.asset_has_dependents(id) {
            self.open_message_box("Asset Has Dependents", "This asset is being used.");
        } else {
            self.open_confirmation_dialog_for(ConfirmationDialogAction::DeleteAsset(id));
        }
    }

    fn remove_asset(&mut self, id: DataAssetId) {
        self.store.remove_asset(id);
        self.editors.remove_editor(id);
        self.window_tracker.remove_editor(id);
    }

    fn duplicate_asset(&mut self, id: DataAssetId, asset_type: DataAssetType) {
        let dup_name = if let Some(asset) = self.store.assets.get_asset(id) {
            self.make_unique_asset_name(asset_type, &format!("{}_copy", asset.name), false)
        } else {
            self.new_asset_name(asset_type, None)
        };
        if let Some(dup_id) = self.store.duplicate_asset(id, &dup_name) {
            let egui_id = self.editors.add_asset(dup_id, asset_type);
            self.window_tracker.add_editor(egui_id, dup_id);
        } else {
            self.open_message_box("Error", "Couldn't duplicate asset.");
        }
    }

    fn add_asset(&mut self, asset_type: DataAssetType, name_prefix: Option<String>) {
        let added = match asset_type {
            DataAssetType::Tileset => {
                self.store.add_tileset(self.new_asset_name(asset_type, name_prefix)).map(|id| (id, self.editors.add_tileset(id)))
            }
            DataAssetType::MapData => {
                if let Some(tileset_id) = self.store.asset_ids.tilesets.get_first() {
                    self.store.add_map(self.new_asset_name(asset_type, name_prefix), tileset_id).map(|id| (id, self.editors.add_map(id)))
                } else {
                    self.open_message_box("No Tileset Available", "You must create a tileset first!");
                    None
                }
            }
            DataAssetType::Room => {
                self.store.add_room(self.new_asset_name(asset_type, name_prefix)).map(|id| (id, self.editors.add_room(id)))
            }
            DataAssetType::World => {
                self.store.add_world(self.new_asset_name(asset_type, name_prefix)).map(|id| (id, self.editors.add_world(id)))
            }
            DataAssetType::Sprite => {
                self.store.add_sprite(self.new_asset_name(asset_type, name_prefix)).map(|id| (id, self.editors.add_sprite(id)))
            }
            DataAssetType::PalSprite => {
                self.store.add_pal_sprite(self.new_asset_name(asset_type, name_prefix)).map(|id| (id, self.editors.add_pal_sprite(id)))
            }
            DataAssetType::SpriteAnimation => {
                if let Some(sprite_id) = self.store.asset_ids.sprites.get_first() {
                    self.store.add_animation(self.new_asset_name(asset_type, name_prefix), sprite_id).map(|id| {
                        (id, self.editors.add_animation(id))
                    })
                } else {
                    self.open_message_box("No Sprite Available", "You must create a sprite first!");
                    None
                }
            }
            DataAssetType::TileAnimation => {
                if let Some(tileset_id) = self.store.asset_ids.tilesets.get_first() {
                    self.store.add_tile_anim(self.new_asset_name(asset_type, name_prefix), tileset_id, tileset_id).map(|id| {
                        (id, self.editors.add_tile_anim(id))
                    })
                } else {
                    self.open_message_box("No Tileset Available", "You must create a tileset first!");
                    None
                }
            }
            DataAssetType::Sfx => {
                self.store.add_sfx(self.new_asset_name(asset_type, name_prefix)).map(|id| (id, self.editors.add_sfx(id)))
            }
            DataAssetType::ModData => {
                self.store.add_mod(self.new_asset_name(asset_type, name_prefix)).map(|id| (id, self.editors.add_mod(id)))
            }
            DataAssetType::Font => {
                self.store.add_font(self.new_asset_name(asset_type, name_prefix)).map(|id| (id, self.editors.add_font(id)))
            }
            DataAssetType::PropFont => {
                self.store.add_prop_font(self.new_asset_name(asset_type, name_prefix)).map(|id| (id, self.editors.add_prop_font(id)))
            }
        };

        if let Some((asset_id, egui_id)) = added {
            self.window_tracker.add_editor(egui_id, asset_id)
        }
    }

    fn open_message_box(&mut self, title: &str, text: &str) {
        self.dialogs.open_message_box(&mut self.window_tracker, title, text);
    }

    fn open_confirm_exit_dialog(&mut self) {
        self.dialogs.open_confirm_exit_dialog(&mut self.window_tracker);
    }

    fn open_confirmation_dialog_for(&mut self, action: ConfirmationDialogAction) {
        match action {
            ConfirmationDialogAction::NewProject => {
                self.dialogs.open_confirmation_dialog(
                    &mut self.window_tracker,
                    "Unsaved Changes",
                    "The current project has unsaved changes.\n\nDiscard changes?",
                    "Yes",
                    "No"
                );
            }
            ConfirmationDialogAction::DeleteAsset(asset_id) => {
                if let Some(asset) = self.store.assets.get_asset(asset_id) {
                    self.dialogs.open_confirmation_dialog(
                        &mut self.window_tracker,
                        "Remove Asset",
                        format!("Remove asset '{}'?", asset.name),
                        "Yes",
                        "No"
                    );
                } else {
                    self.confirmation_dialog_action = None;
                }
            }
        }
        self.confirmation_dialog_action = Some(action);
    }

    fn open_text_input_dialog_for(&mut self, action: TextInputDialogAction) {
        match action {
            TextInputDialogAction::RenameAssetFolder(asset_type, tree_node_id) => {
                if let Some(name) = self.asset_tree.get_node_name(asset_type, tree_node_id) {
                    self.dialogs.open_text_input_dialog(
                        &mut self.window_tracker,
                        "Rename Folder",
                        "Name:",
                        name,
                        "Ok",
                        "Cancel"
                    );
                } else {
                    self.open_message_box("Error", "Error renaming asset folder: folder not found.");
                    return; // don't open the text input dialog
                }
            }

            TextInputDialogAction::RenameAsset(asset_type, asset_id) => {
                if let Some(asset) = self.store.assets.get_asset(asset_id) {
                    self.dialogs.open_text_input_dialog(
                        &mut self.window_tracker,
                        format!("Rename {}", get_asset_type_display_name(asset_type).unwrap_or("Asset")),
                        "Name:",
                        &asset.name,
                        "Ok",
                        "Cancel"
                    );
                } else {
                    self.open_message_box("Error", "Error renaming asset: asset not found.");
                    return; // don't open the text input dialog
                }
            }
        }
        self.text_input_dialog_action = Some(action);
    }

    fn show_dialogs(&mut self, ui: &mut egui::Ui) {
        self.dialogs.show_non_response_dialogs(ui, &mut self.window_tracker, &self.sys_dialogs, &mut self.settings);

        // confirm exit dialog
        let confirm_exit_dialog_result = self.dialogs.show_confirm_exit_dialog(ui, &mut self.window_tracker, &self.sys_dialogs);
        if confirm_exit_dialog_result == DialogResult::Yes {
            self.user_confirmed_close = true;
            ui.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        // confirmation dialog
        let confirmation_dialog_result = self.dialogs.show_confirmation_dialog(ui, &mut self.window_tracker, &self.sys_dialogs);
        if confirmation_dialog_result != DialogResult::None {
            if let Some(action) = &self.confirmation_dialog_action {
                match action {
                    ConfirmationDialogAction::NewProject => {
                        if confirmation_dialog_result == DialogResult::Yes {
                            self.new_project();
                        }
                    }
                    ConfirmationDialogAction::DeleteAsset(asset_id) => {
                        if confirmation_dialog_result == DialogResult::Yes {
                            self.remove_asset(*asset_id);
                        }
                    }
                }
            }
            self.confirmation_dialog_action = None;
        }

        // text input dialog
        let text_input_dialog_result = self.dialogs.show_text_input_dialog(ui, &mut self.window_tracker, &self.sys_dialogs);
        if text_input_dialog_result != DialogResult::None {
            if let Some(action) = &self.text_input_dialog_action {
                match action {
                    TextInputDialogAction::RenameAssetFolder(asset_type, tree_node_id) => {
                        if text_input_dialog_result == DialogResult::Yes {
                            let new_name = self.dialogs.get_text_input_dialog_input();
                            self.rename_asset_folder(*asset_type, *tree_node_id, new_name);
                        }
                    }
                    TextInputDialogAction::RenameAsset(_asset_type, asset_id) => {
                        if text_input_dialog_result == DialogResult::Yes &&
                            let Some(asset) = self.store.assets.get_asset_mut(*asset_id) {
                                asset.name = self.dialogs.get_text_input_dialog_input();
                            }
                    }
                }
            }
            self.text_input_dialog_action = None;
        }
    }

    fn show_menu_bar(&mut self, ui: &mut egui::Ui, window: &mut eframe::Frame) {
        egui::Panel::top("main_menu").show(ui, |ui| {
            self.sys_dialogs.block_ui(ui);

            let file_save_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::S);
            if ui.input_mut(|i| i.consume_shortcut(&file_save_shortcut)) {
                self.save(window);
            }
            let file_quit_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::Q);
            if ui.input_mut(|i| i.consume_shortcut(&file_quit_shortcut)) {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
            let run_check_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::F2);
            if ui.input_mut(|i| i.consume_shortcut(&run_check_shortcut)) {
                self.run_project_check(ui.ctx());
            }

            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.add(menu_item(IMAGES.new, " New")).clicked() {
                        if self.editors.is_dirty() {
                            self.open_confirmation_dialog_for(ConfirmationDialogAction::NewProject);
                        } else {
                            self.new_project();
                        }
                    }
                    ui.separator();
                    if ui.add(menu_item(IMAGES.open, " Open...")).clicked() {
                        self.sys_dialogs.open_file(
                            Some(window),
                            Self::OPEN_PROJECT_SYS_DLG_ID.to_owned(),
                            "project",
                            "Open Project",
                            &[
                                ("Raven project files (*.h)", &["h"]),
                                ("All files (*.*)", &["*"]),
                            ]
                        );
                    };
                    if self.is_wasm {
                        if ui.add(menu_item(IMAGES.save, " Save As...")).clicked() {
                            self.save_as(window);
                        }
                    } else {
                        ui.add_enabled_ui(self.recent_projects.num_files() > 0, |ui| {
                            let mut selected_project = None;
                            menu_item_no_image_with_submenu(" Open Recent", ui, |ui| {
                                for (index, path) in self.recent_projects.files().enumerate() {
                                    if ui.button(path.to_string_lossy()).clicked() {
                                        selected_project = Some(index);
                                    }
                                }
                            });
                            if let Some(index) = selected_project &&
                            let Some(path) = self.recent_projects.file(index) &&
                            let Some(file) = SysDialogOpenFile::create(path) {
                                self.open(file);
                            }
                        });
                        if ui.add(menu_item(IMAGES.save, " Save")).clicked() {
                            self.save(window);
                        }
                        if ui.add(menu_item_no_image(" Save As...")).clicked() {
                            self.save_as(window);
                        }
                    }
                    ui.separator();
                    if ui.add(menu_item(IMAGES.properties, " Settings")).clicked() {
                        self.windows.open_settings();
                    }
                    ui.separator();
                    if ui.add(menu_item(IMAGES.chicken, " Quit")).clicked() {
                        if self.is_wasm {
                            self.open_message_box(
                                "Nope",
                                "You can't quit the Web, silly :)"
                            );
                        } else {
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                });
                ui.menu_button("Project", |ui| {
                    for asset_def in ASSET_DEFS {
                        if ui.add(menu_item(include_ref_image!(asset_def.image), asset_def.add_menu_item)).clicked() {
                            self.add_asset(asset_def.asset_type, None);
                        }
                    }
                    ui.separator();
                    if ui.add(menu_item(IMAGES.properties, " Properties")).clicked() {
                        self.windows.open_properties();
                    }
                });
                ui.menu_button("Tools", |ui| {
                    if ui.add(menu_item(IMAGES.export, " Export header...")).clicked() {
                        self.sys_dialogs.save_file(
                            Some(window),
                            Self::EXPORT_HEADER_SYS_DLG_ID.to_owned(),
                            "project",
                            "Export Header File",
                            &[
                                ("Header files (*.h)", &["h"]),
                                ("All files (*.*)", &["*"]),
                            ]
                        );
                    }

                    ui.separator();

                    if ui.add(menu_item(IMAGES.compare, " Compare Project")).clicked() {
                        self.windows.open_project_comparer();
                    }
                    if ui.add(menu_item(IMAGES.ok, " Check Project")).clicked() {
                        self.run_project_check(ui.ctx());
                    }
                });
                ui.menu_button("Help", |ui| {
                    if ui.add(menu_item(IMAGES.info, " Status")).clicked() {
                        self.windows.open_status();
                    }
                    ui.separator();
                    if ui.add(menu_item(IMAGES.pico, " About")).clicked() {
                        self.dialogs.open_about(&mut self.window_tracker);
                    }
                });
            });
        });
    }

    fn show_toolbar(&mut self, ui: &mut egui::Ui, window: &mut eframe::Frame) {
        egui::Panel::top("main_toolbar").show(ui, |ui| {
            self.sys_dialogs.block_ui(ui);

            ui.horizontal(|ui| {
                let spacing = ui.spacing().item_spacing;
                ui.spacing_mut().item_spacing = egui::Vec2::new(1.0, 0.0);

                if ui.add(egui::Button::image(IMAGES.new).frame_when_inactive(false)).on_hover_text("New Project").clicked() {
                    if self.editors.is_dirty() {
                        self.open_confirmation_dialog_for(ConfirmationDialogAction::NewProject);
                    } else {
                        self.new_project();
                    }
                }
                if ui.add(egui::Button::image(IMAGES.open).frame_when_inactive(false)).on_hover_text("Open Project").clicked() {
                    self.sys_dialogs.open_file(
                        Some(window),
                        Self::OPEN_PROJECT_SYS_DLG_ID.to_owned(),
                        "project",
                        "Open Project",
                        &[
                            ("Raven project files (*.h)", &["h"]),
                            ("All files (*.*)", &["*"]),
                        ]
                    );
                }
                let save_label = if self.is_wasm { "Save Project" } else { "Save Project (Ctrl+S)" };
                if ui.add(egui::Button::image(IMAGES.save).frame_when_inactive(false)).on_hover_text(save_label).clicked() {
                    self.save(window);
                }

                ui.add_space(5.0);
                ui.separator();
                ui.add_space(5.0);

                if ui.add(
                    egui::Button::image_and_text(IMAGES.log, "Log")
                        .selected(self.windows.collection.log_window.base.open)
                        .frame_when_inactive(self.windows.collection.log_window.base.open)
                ).on_hover_text("Log Window").clicked() {
                    self.windows.collection.log_window.toggle_open();
                }

                ui.add_space(5.0);
                ui.separator();
                ui.add_space(5.0);

                if ui.add(
                    egui::Button::new("\u{23f5}")
                        .selected(self.windows.collection.game_runner.base.open)
                        .frame_when_inactive(true)
                ).on_hover_text("Open Game Tester").clicked() {
                    self.windows.collection.game_runner.open(ui.ctx());
                }

                ui.spacing_mut().item_spacing = spacing;
            });

            ui.add_space(0.0);
        });
    }

    fn show_footer(&mut self, ui: &mut egui::Ui) {
        egui::Panel::bottom("footer").show(ui, |ui| {
            self.sys_dialogs.block_ui(ui);
            ui.add_space(5.0);

            let dirty = if self.editors.is_dirty() { " (modified)" } else { "" };
            ui.label(format!("{} bytes [{} assets]{}", self.store.assets.data_size(), self.store.num_assets(), dirty));
        });
    }

    fn show_asset_tree(&mut self, ui: &mut egui::Ui) {
        self.asset_tree.update(&self.store);
        egui::Panel::left("asset_tree").resizable(false).exact_size(Self::ASSET_TREE_PANEL_WIDTH).show(ui, |ui| {
            ui.add_space(2.0);
            self.sys_dialogs.block_ui(ui);
            egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                for asset_def in ASSET_DEFS {
                    let mut folder_action = AssetTreeAction::None;
                    let mut item_action = AssetTreeAction::None;
                    if let Some(tree) = self.asset_tree.get_tree_of_type(asset_def.asset_type) {
                        let is_folder_open = |folder: &widgets::AssetTreeContainer| { folder.node_id == tree.node_id };
                        let mut show_folder = |ui: &mut egui::Ui, folder: &widgets::AssetTreeContainer| -> egui::Response {
                            ui.horizontal(|ui| {
                                let button = if folder.level == 0 {
                                    egui::Button::image_and_text(include_ref_image!(asset_def.image), &folder.name).gap(8.0)
                                } else {
                                    egui::Button::new(&folder.name)
                                };
                                let header = ui.add(button.frame_when_inactive(false));
                                egui::Popup::context_menu(&header).show(|ui| {
                                    if folder.level != 0 {
                                        if ui.add(menu_item_no_image("Rename folder")).clicked() {
                                            folder_action = AssetTreeAction::RenameFolder(folder.node_id);
                                        }
                                        ui.separator();
                                    }
                                    if ui.add(menu_item(include_ref_image!(asset_def.image), asset_def.add_menu_item)).clicked() {
                                        folder_action = AssetTreeAction::AddAsset(folder.node_id);
                                    }
                                });
                                header
                            }).inner
                        };
                        let mut show_item = |ui: &mut egui::Ui, folder: &widgets::AssetTreeContainer, asset_item: &widgets::AssetTreeItem| {
                            ui.horizontal(|ui| {
                                if folder.level == 0 {
                                    ui.add_space(6.0);
                                }
                                let button = ui.button(&asset_item.name);
                                if button.clicked() {
                                    item_action = AssetTreeAction::OpenEditor(asset_item.id);
                                }
                                egui::Popup::context_menu(&button).show(|ui| {
                                    if ui.add(menu_item_no_image(asset_def.rename_menu_item)).clicked() {
                                        item_action = AssetTreeAction::RenameAsset(asset_item.id);
                                    }
                                    if ui.add(menu_item(IMAGES.copy, asset_def.duplicate_menu_item)).clicked() {
                                        item_action = AssetTreeAction::DuplicateAsset(asset_item.id);
                                    }
                                    ui.separator();
                                    if ui.add(menu_item(IMAGES.trash, asset_def.remove_menu_item)).clicked() {
                                        item_action = AssetTreeAction::RemoveAsset(asset_item.id);
                                    }
                                });
                            });
                        };
                        tree.show("project", ui, &is_folder_open, &mut show_folder, &mut show_item);
                    }
                    for action in &[folder_action, item_action] {
                        match action {
                            AssetTreeAction::AddAsset(tree_node_id) => {
                                self.add_asset(
                                    asset_def.asset_type,
                                    self.asset_tree.get_node_name(asset_def.asset_type, *tree_node_id)
                                );
                            }
                            AssetTreeAction::RenameFolder(tree_node_id) => {
                                self.open_text_input_dialog_for(TextInputDialogAction::RenameAssetFolder(asset_def.asset_type, *tree_node_id))
                            }
                            AssetTreeAction::RenameAsset(asset_id) => {
                                self.open_text_input_dialog_for(TextInputDialogAction::RenameAsset(asset_def.asset_type, *asset_id))
                            }
                            AssetTreeAction::OpenEditor(asset_id) => {
                                if let Some(editor) = self.editors.get_editor_mut(*asset_id) {
                                    editor.toggle_open(ui.ctx());
                                }
                            }
                            AssetTreeAction::DuplicateAsset(asset_id) => {
                                self.duplicate_asset(*asset_id, asset_def.asset_type);
                            }
                            AssetTreeAction::RemoveAsset(asset_id) => {
                                self.request_remove_asset(*asset_id);
                            }
                            AssetTreeAction::None => {},
                        }
                    }
                }
            });
        });
    }

    fn show_windows(&mut self, ui: &mut egui::Ui, window: &eframe::Frame) -> Vec<AppWindowAction> {
        let frame = egui::Frame::NONE.fill(ui.visuals().panel_fill);
        let window_space = egui::CentralPanel::default().frame(frame).show(ui, |ui| {
            self.sys_dialogs.block_ui(ui);
            // big empty space where project windows will be placed
            ui.available_rect_before_wrap()
        }).inner;
        let mut win_ctx = WindowContext {
            window_space,
            vga_bits_per_pixel: self.store.vga_bits_per_pixel,
            tiles_per_world_block: self.store.tiles_per_world_block,
            egui: WindowEguiContext::new(ui.ctx(), window),
            tex_man: &mut self.tex_manager,
            sys_dialogs: &mut self.sys_dialogs,
            dialogs: &mut self.dialogs,
            logger: &mut self.logger,
            settings: &mut self.settings,
            window_tracker: &mut self.window_tracker,
            map_clipboard: self.map_clipboard.take(),
            image_clipboard: self.image_clipboard.take(),
            keyboard_pressed: self.keyboard_pressed.take(),
            editor_actions: Vec::new(),
        };

        self.editors.refresh_room_names(&self.store.assets.rooms);

        // if some window was closed, focus the window that's closest to the top (if any)
        if self.editors.iter().any(|e| e.closed_last_frame) || self.windows.some_closed_last_frame {
            // try editor windows
            let open_editor_ids = self.editors.get_open_ids().collect();
            if win_ctx.bring_topmost_to_top(&open_editor_ids).is_none() {
                // try non-editor windows
                let open_window_ids = self.windows.get_open_ids().collect();
                win_ctx.bring_topmost_to_top(&open_window_ids);
            }
            for e in self.editors.iter_mut() {
                e.closed_last_frame = false;
            }
        }

        for tileset in self.store.assets.tilesets.iter_mut() {
            if let Some(editor) = self.editors.tilesets.get_mut(&tileset.asset.id) {
                editor.show(&mut win_ctx, tileset);
            }
        }
        for map in self.store.assets.maps.iter_mut() {
            if let Some(editor) = self.editors.maps.get_mut(&map.asset.id) {
                let assets = editors::MapDataEditorAssetLists::new(
                    &self.store.assets.tilesets,
                    &self.store.assets.tile_anims
                );
                editor.show(&mut win_ctx, map, &self.store.asset_ids, &assets);
            }
        }
        for room in self.store.assets.rooms.iter_mut() {
            if let Some(editor) = self.editors.rooms.get_mut(&room.asset.id) {
                let assets = editors::RoomEditorAssetLists::new(
                    &self.store.assets.maps,
                    &self.store.assets.tilesets,
                    &self.store.assets.tile_anims,
                    &self.store.assets.animations,
                    &self.store.assets.sprites,
                    &self.editors.room_names);
                editor.show(&mut win_ctx, room, &self.store.asset_ids, &assets);
            }
        }
        for world in self.store.assets.worlds.iter_mut() {
            if let Some(editor) = self.editors.worlds.get_mut(&world.asset.id) {
                let mut assets = editors::WorldEditorAssetLists::new(
                    &mut self.store.assets.rooms,
                    &self.store.assets.maps,
                    &self.store.assets.tilesets);
                editor.show(&mut win_ctx, world, &mut assets);
            }
        }
        for sprite in self.store.assets.sprites.iter_mut() {
            if let Some(editor) = self.editors.sprites.get_mut(&sprite.asset.id) {
                editor.show(&mut win_ctx, sprite);
            }
        }
        for pal_sprite in self.store.assets.pal_sprites.iter_mut() {
            if let Some(editor) = self.editors.pal_sprites.get_mut(&pal_sprite.asset.id) {
                editor.show(&mut win_ctx, pal_sprite);
            }
        }
        for anim in self.store.assets.animations.iter_mut() {
            if let Some(editor) = self.editors.animations.get_mut(&anim.asset.id) {
                editor.show(&mut win_ctx, anim, &self.store.asset_ids, &mut self.store.assets.sprites);
            }
        }
        for tanim in self.store.assets.tile_anims.iter_mut() {
            if let Some(editor) = self.editors.tile_anims.get_mut(&tanim.asset.id) {
                editor.show(&mut win_ctx, tanim, &self.store.asset_ids, &mut self.store.assets.tilesets);
            }
        }
        for sfx in self.store.assets.sfxs.iter_mut() {
            if let Some(editor) = self.editors.sfxs.get_mut(&sfx.asset.id) {
                editor.show(&mut win_ctx, sfx, &mut self.sound_player);
            }
        }
        for mod_data in self.store.assets.mods.iter_mut() {
            if let Some(editor) = self.editors.mods.get_mut(&mod_data.asset.id) {
                editor.show(&mut win_ctx, mod_data, &mut self.sound_player);
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

        let window_actions = self.windows.show(&mut win_ctx, &mut self.store);

        for editor_action in std::mem::take(&mut win_ctx.editor_actions) {
            editor_action.run(&mut win_ctx, &mut self.store, &mut self.editors, &mut self.windows, &mut self.asset_exporter);
        }

        self.map_clipboard = win_ctx.map_clipboard.take();
        self.image_clipboard = win_ctx.image_clipboard.take();

        window_actions
    }
}

impl eframe::App for RavenEditorApp {
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        // This is a hack.  Egui eats copy/cut/paste keyboard
        // shortcuts (cmd+c/cmd+x/cmd+v) and transforms them into
        // events (Copy/Cut/Paste), so we check for these and store
        // the info away to be used in the WindowContext.
        for event in &raw_input.events {
            if let Some(key_pressed) = platform::get_event_key(event) {
                self.keyboard_pressed = Some(key_pressed);
            }
        }
    }

    fn on_exit(&mut self) {
        self.recent_projects.save(&mut self.logger);
        self.sys_dialogs.save_paths(&mut self.logger);
    }

    fn ui(&mut self, ui: &mut egui::Ui, window: &mut eframe::Frame) {
        if let Some(SysDialogResponse::File(file)) = self.sys_dialogs.get_response_for(Self::SAVE_PROJECT_SYS_DLG_ID) {
            let path = file.path().map(|path| path.to_owned());
            if self.write_project(file) {
                self.set_path(path);
            }
        }
        if let Some(SysDialogResponse::File(file)) = self.sys_dialogs.get_response_for(Self::OPEN_PROJECT_SYS_DLG_ID) {
            self.open(file);
        }
        if let Some(SysDialogResponse::File(file)) = self.sys_dialogs.get_response_for(Self::EXPORT_HEADER_SYS_DLG_ID) {
            self.export_header(file);
        }
        if self.asset_exporter.check_dialog_response(&mut self.sys_dialogs, &self.store, &mut self.logger) {
            self.open_message_box(
                "Error Exportint Asset",
                "Error exporting asset.\n\nConsult the log window for details."
            );
            self.windows.open_log_window();
        }

        if self.is_wasm {
            // to confirm close when dirty in wasm, we update the shared dirty status used by the "beforeunload" closure:
            *self.wasm_editor_is_dirty.lock().unwrap() = self.editors.is_dirty();
        } else if ui.ctx().input(|i| i.viewport().close_requested()) && self.editors.is_dirty() && ! self.user_confirmed_close {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.open_confirm_exit_dialog();
        }
        if self.reset_egui_context {
            ui.ctx().memory_mut(|mem| {
                mem.reset_areas();
                mem.data.clear();
            });
            self.reset_egui_context = false;
        }
        if self.path_changed {
            let title = match &self.path {
                Some(path) => match path.as_path().file_name() {
                    Some(filename) => format!("[{}] - Raven Game Editor", filename.display()).to_string(),
                    None => "[???] - Raven Game Editor".to_owned(),
                }
                None => "<unnamed> - Raven Game Editor".to_owned()
            };
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Title(title));
            self.path_changed = false;
        }

        self.editors.update_dirty_flags(&self.store);
        self.show_dialogs(ui);
        self.show_menu_bar(ui, window);
        self.show_toolbar(ui, window);
        self.show_footer(ui);
        self.show_asset_tree(ui);

        for window_action in self.show_windows(ui, window).into_iter() {
            match window_action {
                AppWindowAction::None => {
                }
                AppWindowAction::CloseWindow(window_id) => {
                    self.windows.close(window_id);
                }
                AppWindowAction::ActivateAssetEditor(asset_id) => {
                    self.activate_asset_editor(ui.ctx(), asset_id);
                }
            }
        }
    }
}
