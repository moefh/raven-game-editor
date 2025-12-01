use std::collections::{HashMap, HashSet};
use crate::image::TextureManager;
use crate::app::{AppDialogs, SysDialogs, AppSettings};
use crate::data_asset::{DataAssetId, StringLogger};
use crate::editors::ClipboardData;

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

- `editor_ids` are the editor windows;

- `non_asset_ids` are other app windows that are not asset editors
  (e.g. the log window), which always block editors from processing
  shortcuts (if above the editors in EGUI order);

- `open_ids` are the dialog windows, which block editors only if they
  are visible (i.e., open).
*/
pub struct AppWindowTracker {
    pub open_ids: HashMap<egui::Id, bool>,
    pub editor_ids: HashMap<egui::Id, DataAssetId>,
    pub non_asset_ids: HashSet<egui::Id>,
}

impl AppWindowTracker {
    pub fn new() -> Self {
        AppWindowTracker {
            open_ids: HashMap::new(),
            editor_ids: HashMap::new(),
            non_asset_ids: HashSet::new(),
        }
    }

    /**
     * Mark a dialog window as open or closed. This must be called
     * whenever a dialog window opens or closes.
     */
    pub fn set_open(&mut self, id: egui::Id, open: bool) {
        self.open_ids.insert(id, open);
    }

    /**
     * Reset the state of all windows.
     */
    pub fn reset(&mut self, editor_ids: &HashMap<egui::Id, DataAssetId>, non_asset_ids: &[egui::Id]) {
        self.editor_ids.clear();
        self.open_ids.clear();
        self.non_asset_ids.clear();
        for (egui_id, asset_id) in editor_ids {
            self.editor_ids.insert(*egui_id, *asset_id);
        }
        for egui_id in non_asset_ids {
            self.non_asset_ids.insert(*egui_id);
        }
    }

    /**
     * Return the asset id of the editor that should process keyboard
     * shortcuts, `None` if no editor should process shortcuts.
     */
    pub fn get_top_editor_asset_id(&self, ctx: &egui::Context) -> Option<DataAssetId> {
        ctx.memory(|mem| {
            mem.layer_ids().fold(None, |top, layer_id| {
                if let Some(true) = self.open_ids.get(&layer_id.id) {  // open dialog
                    None
                } else if self.non_asset_ids.contains(&layer_id.id) {  // non-editor window
                    None
                } else {
                    self.editor_ids.get(&layer_id.id).copied().or(top)
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
    pub egui: WindowEguiContext<'a>,
    pub tex_man: &'a mut TextureManager,
    pub dialogs: &'a mut AppDialogs,
    pub sys_dialogs: &'a mut SysDialogs,
    pub logger: &'a mut StringLogger,
    pub settings: &'a mut AppSettings,
    pub window_tracker: &'a mut AppWindowTracker,
    pub clipboard: Option<ClipboardData>,
    pub keyboard_pressed: Option<KeyboardPressed>,
}

impl<'a> WindowContext<'a> {
    pub fn request_marching_ants_repaint(&self) {
        self.egui.ctx.request_repaint_after(std::time::Duration::from_millis(self.settings.marching_ants_delay as u64));
    }

    pub fn is_editor_on_top(&self, id: DataAssetId) -> bool {
        self.window_tracker.get_top_editor_asset_id(self.egui.ctx) == Some(id)
    }

    pub fn set_window_open(&mut self, window_id: egui::Id, open: bool) {
        self.window_tracker.set_open(window_id, open);
    }

    pub fn open_message_box(&mut self, title: impl AsRef<str>, text: impl AsRef<str>) {
        self.dialogs.open_message_box(self.window_tracker, title, text);
    }
}
