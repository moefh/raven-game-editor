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
    pub tex_man: &'a mut super::AppTextureManager,
}

impl<'a> WindowContext<'a> {
    pub fn new(window_space: egui::Rect, ctx: &'a egui::Context, tex_man: &'a mut super::AppTextureManager) -> Self {
        WindowContext {
            window_space,
            tex_man,
            egui: WindowEguiContext::new(ctx),
        }
    }
}

/*
            let tile = 22;
            let stride = tileset.stride as usize;
            let width = tileset.width as usize;
            let height = tileset.height as usize;
            let mut pixels = Vec::<u8>::with_capacity(3 * width * height);
            for &quad in &tileset.data[tile*stride*height..(tile+1)*stride*height] {
                let mut src = quad;
                for _ in 0..4 {
                    let pix = src & 0xff;
                    src >>= 8;
                    let r = (pix&0x30) >> 4;
                    let g = (pix&0x0c) >> 2;
                    let b = (pix&0x03) >> 0;
                    pixels.push((r << 6) as u8);
                    pixels.push((g << 6) as u8);
                    pixels.push((b << 6) as u8);
                }
            }
*/
