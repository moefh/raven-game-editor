use crate::misc::TextureManager;
use crate::app::AppDialogs;

pub struct WindowEguiContext<'a> {
    pub ctx: &'a egui::Context,
}

impl<'a> WindowEguiContext<'a> {
    pub fn new(ctx: &'a egui::Context) -> Self {
        WindowEguiContext {
            ctx
        }
    }
}

pub struct WindowContext<'a> {
    pub window_space: egui::Rect,
    pub egui: WindowEguiContext<'a>,
    pub tex_man: &'a mut TextureManager,
    pub dialogs: &'a mut AppDialogs,
}

impl<'a> WindowContext<'a> {
    pub fn new(window_space: egui::Rect, ctx: &'a egui::Context, tex_man: &'a mut TextureManager, dialogs: &'a mut AppDialogs) -> Self {
        WindowContext {
            window_space,
            tex_man,
            dialogs,
            egui: WindowEguiContext::new(ctx),
        }
    }
}
