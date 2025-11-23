use crate::misc::TextureManager;
use crate::app::{AppDialogs, SysDialogs, AppSettings};
use crate::data_asset::StringLogger;

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
}
