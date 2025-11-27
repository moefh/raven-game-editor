use crate::image::TextureManager;
use crate::app::{AppDialogs, SysDialogs, AppSettings};
use crate::data_asset::{DataAssetId, StringLogger};
use crate::editors::ClipboardData;

pub enum KeyboardPressed {
    CtrlC,
    CtrlV,
    CtrlX,
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
    pub top_editor_asset_id: Option<DataAssetId>,
    pub clipboard: Option<ClipboardData>,
    pub keyboard_pressed: Option<KeyboardPressed>,
}

impl<'a> WindowContext<'a> {
    pub fn request_marching_ants_repaint(&self) {
        self.egui.ctx.request_repaint_after(std::time::Duration::from_millis(self.settings.marching_ants_delay as u64));
    }

    pub fn is_editor_on_top(&self, id: DataAssetId) -> bool {
        match self.top_editor_asset_id {
            Some(top_id) => top_id == id,
            None => false,
        }
    }
}
