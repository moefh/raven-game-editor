use crate::misc::texture_manager::TextureManager;

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
}

impl<'a> WindowContext<'a> {
    pub fn new(window_space: egui::Rect, ctx: &'a egui::Context, tex_man: &'a mut TextureManager) -> Self {
        WindowContext {
            window_space,
            tex_man,
            egui: WindowEguiContext::new(ctx),
        }
    }
}
