use std::collections::{HashMap, HashSet};
use crate::image::TextureManager;
use crate::app::{AppDialogs, SysDialogs, AppSettings};
use crate::data_asset::{DataAssetId, StringLogger};
use crate::editors::{MapClipboardData, ImageClipboardData};

pub enum KeyboardPressed {
    CtrlC,
    CtrlV,
    CtrlX,
}

/**
The `AppWindowTracker` is used to decide whether a window should
process keyboard shortcuts.

We can get an ordered list of EGUI window ids from the EGUI
context. An editor window should process keyboard shortcuts if there's
no active window above it in the EGUI order. We can't simply check if
it's the top window because EGUI keeps all windows it's ever seen in
the list (i.e., even windows that are not open), and worse, the list
also has child-windows of an editor (e.g. widgets).

So we keep 3 types of windows in storage:

- `editor_window_ids` are the editor windows;

- `non_editor_window_ids` are other app windows that are not asset editors
  (e.g. the log window), which always block editors from processing
  shortcuts (if above the editors in EGUI order);

- `open_dialog_ids` are the dialog windows, which block editors only if they
  are visible (i.e., open).
*/
pub struct AppWindowTracker {
    pub open_dialog_ids: HashMap<egui::Id, bool>,
    pub editor_window_ids: HashMap<egui::Id, DataAssetId>,
    pub non_editor_window_ids: HashSet<egui::Id>,
}

impl AppWindowTracker {
    pub fn new() -> Self {
        AppWindowTracker {
            editor_window_ids: HashMap::new(),
            non_editor_window_ids: HashSet::new(),
            open_dialog_ids: HashMap::new(),
        }
    }

    /**
     * Bring the window with the given id to the top.
     */
    pub fn bring_to_top(id: egui::Id, ctx: &egui::Context) {
        if let Some(layer_id) = ctx.memory(|mem| {
            mem.layer_ids().find(|layer_id| { layer_id.id == id })
        }) {
            ctx.move_to_top(layer_id);
        }
    }

    /**
     * Mark a dialog window as open or closed. This must be called
     * whenever a dialog window opens or closes.
     */
    pub fn set_dialog_open(&mut self, id: egui::Id, open: bool) {
        self.open_dialog_ids.insert(id, open);
    }

    /**
     * Reset the state of all windows.
     */
    pub fn reset(&mut self, editor_window_ids: &HashMap<egui::Id, DataAssetId>, non_editor_window_ids: &[egui::Id]) {
        self.editor_window_ids.clear();
        self.non_editor_window_ids.clear();
        self.open_dialog_ids.clear();
        for (egui_id, asset_id) in editor_window_ids {
            self.editor_window_ids.insert(*egui_id, *asset_id);
        }
        for egui_id in non_editor_window_ids {
            self.non_editor_window_ids.insert(*egui_id);
        }
    }

    pub fn add_editor(&mut self, egui_id: egui::Id, asset_id: DataAssetId) {
        self.editor_window_ids.insert(egui_id, asset_id);
    }

    pub fn remove_editor(&mut self, editor_asset_id: DataAssetId) {
        if let Some(egui_id) = self.editor_window_ids.iter().find_map(|(&egui_id, &asset_id)| {
            if asset_id == editor_asset_id {
                Some(egui_id)
            } else {
                None
            }
        }) {
            self.editor_window_ids.remove(&egui_id);
        }
    }

    /**
     * Return the asset id of the editor that should process keyboard
     * shortcuts, `None` if no editor should process shortcuts.
     */
    pub fn get_top_editor_asset_id(&self, ctx: &egui::Context) -> Option<DataAssetId> {
        ctx.memory(|mem| {
            mem.layer_ids().fold(None, |top, layer_id| {
                if let Some(true) = self.open_dialog_ids.get(&layer_id.id) {
                    None
                } else if self.non_editor_window_ids.contains(&layer_id.id) {
                    None
                } else {
                    self.editor_window_ids.get(&layer_id.id).copied().or(top)
                }
            })
        })
    }

    pub fn get_top_non_editor_id(&self, ctx: &egui::Context) -> Option<egui::Id> {
        ctx.memory(|mem| {
            mem.layer_ids().fold(None, |top, layer_id| {
                if let Some(true) = self.open_dialog_ids.get(&layer_id.id) {
                    Some(layer_id.id)
                } else if self.non_editor_window_ids.contains(&layer_id.id) {
                    Some(layer_id.id)
                } else if self.editor_window_ids.contains_key(&layer_id.id) {
                    None
                } else {
                    top
                }
            })
        })
    }

    pub fn get_topmost_of(&self, ids: &HashSet<egui::Id>, ctx: &egui::Context) -> Option<egui::LayerId> {
        ctx.memory(|mem| {
            mem.layer_ids().fold(None, |top, layer_id| {
                if ids.contains(&layer_id.id) {
                    Some(layer_id)
                } else {
                    top
                }
            })
        })
    }
}

pub struct WindowEguiContext<'a> {
    pub ctx: &'a egui::Context,
    pub window: &'a eframe::Frame,
}

impl<'a> WindowEguiContext<'a> {
    pub fn new(ctx: &'a egui::Context, window: &'a eframe::Frame) -> Self {
        WindowEguiContext {
            ctx,
            window,
        }
    }
}

pub struct WindowContext<'a> {
    pub window_space: egui::Rect,
    pub vga_bits_per_pixel: u8,
    pub egui: WindowEguiContext<'a>,
    pub tex_man: &'a mut TextureManager,
    pub dialogs: &'a mut AppDialogs,
    pub sys_dialogs: &'a mut SysDialogs,
    pub logger: &'a mut StringLogger,
    pub settings: &'a mut AppSettings,
    pub window_tracker: &'a mut AppWindowTracker,
    pub image_clipboard: ImageClipboardData,
    pub map_clipboard: MapClipboardData,
    pub keyboard_pressed: Option<KeyboardPressed>,
}

impl<'a> WindowContext<'a> {
    pub fn request_marching_ants_repaint(&self) {
        self.egui.ctx.request_repaint_after(std::time::Duration::from_millis(self.settings.marching_ants_delay as u64));
    }

    pub fn is_editor_on_top(&self, id: DataAssetId) -> bool {
        self.window_tracker.get_top_editor_asset_id(self.egui.ctx) == Some(id)
    }

    pub fn is_window_on_top(&self, window_id: egui::Id) -> bool {
        self.window_tracker.get_top_non_editor_id(self.egui.ctx) == Some(window_id)
    }

    pub fn set_dialog_open(&mut self, dialog_id: egui::Id, open: bool) {
        self.window_tracker.set_dialog_open(dialog_id, open);
    }

    pub fn open_message_box(&mut self, title: impl AsRef<str>, text: impl AsRef<str>) {
        self.dialogs.open_message_box(self.window_tracker, title, text);
    }

    pub fn open_colorset_dialog(&mut self, colorset: usize) {
        self.dialogs.open_colorset_dialog(self.window_tracker, colorset);
    }

    // bring to the top the layer among `ids` that's closest to the top
    pub fn bring_topmost_to_top(&self, ids: &HashSet<egui::Id>) -> Option<egui::Id> {
        let top_layer_id = self.window_tracker.get_topmost_of(ids, self.egui.ctx);
        if let Some(layer_id) = top_layer_id {
            self.egui.ctx.move_to_top(layer_id);
            Some(layer_id.id)
        } else {
            None
        }
    }
}
