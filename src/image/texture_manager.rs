use std::collections::HashMap;

use super::TextureName;

const MAX_COLORS: usize = 256;

pub struct TextureManager {
    textures: HashMap<TextureName, egui::TextureHandle>,
    pixel_to_rgb_opaque: Vec<egui::Color32>,
    pixel_to_rgb_transp: Vec<egui::Color32>,
}

impl TextureManager {
    pub fn new() -> Self {
        let mut pixel_to_rgb_opaque = vec![egui::Color32::BLACK; MAX_COLORS];
        let mut pixel_to_rgb_transp = vec![egui::Color32::BLACK; MAX_COLORS];
        for color in 0..MAX_COLORS {
            let r = color as u8 & 0x07;
            let g = (color as u8 & 0x38) >> 3;
            let b = (color as u8 & 0xc0) >> 6;
            let cr = (r << 5) | (r << 2) | (r >> 2);
            let cg = (g << 5) | (g << 2) | (g >> 2);
            let cb = (b << 6) | (b << 4) | (b << 2) | b;
            let rgb = egui::Color32::from_rgb(cr, cg, cb);
            pixel_to_rgb_opaque[color] = rgb;
            pixel_to_rgb_transp[color] = if r==0 && g==7 && b == 0 { egui::Color32::TRANSPARENT } else { rgb };
        }

        TextureManager {
            textures: HashMap::new(),
            pixel_to_rgb_opaque,
            pixel_to_rgb_transp,
        }
    }

    pub fn clear(&mut self) {
        self.textures.clear();
    }

    pub fn get_rgba_texture(&mut self, ctx: &egui::Context, name: TextureName,
                            w: usize, h: usize, data: &[u8], force_load: bool) -> &egui::TextureHandle {
        let load_image = || {
            let pixels: Vec::<egui::Color32> = data.iter().map(|pix| {
                self.pixel_to_rgb_opaque[*pix as usize]
            }).collect();
            ctx.load_texture(
                format!("{}", name),
                egui::epaint::ColorImage::new([w, h], pixels),
                egui::TextureOptions::NEAREST,
            )
        };
        if force_load {
            self.textures.entry(name).insert_entry(load_image()).into_mut()
        } else {
            self.textures.entry(name).or_insert_with(load_image)
        }
    }

    pub fn get_rgba_texture_transparent(&mut self, ctx: &egui::Context, name: TextureName,
                                        w: usize, h: usize, data: &[u8], force_load: bool) -> &egui::TextureHandle {
        let load_image = || {
            let pixels: Vec::<egui::Color32> = data.iter().map(|pix| {
                self.pixel_to_rgb_transp[*pix as usize]
            }).collect();
            ctx.load_texture(
                format!("{}", name),
                egui::epaint::ColorImage::new([w, h], pixels),
                egui::TextureOptions::NEAREST,
            )
        };
        if force_load {
            self.textures.entry(name).insert_entry(load_image()).into_mut()
        } else {
            self.textures.entry(name).or_insert_with(load_image)
        }
    }
}
