use std::collections::HashMap;

use super::TextureName;

const MAX_COLORS: usize = 256;

pub struct TextureManager {
    textures: HashMap<TextureName, egui::TextureHandle>,
    bits_per_pixel: u8,
    color_to_rgb_opaque: Vec<egui::Color32>,
    color_to_rgb_transp: Vec<egui::Color32>,
}

impl TextureManager {
    fn gen_color6_to_rgb_maps(opaque: &mut Vec<egui::Color32>, transp: &mut Vec<egui::Color32>) {
        for color in 0..MAX_COLORS {
            let r = (color as u8 >> 1) & 0x3;
            let g = (color as u8 >> 4) & 0x3;
            let b = (color as u8 >> 6) & 0x3;
            let cr = (r << 6) | (r << 4) | (r << 2) | r;
            let cg = (g << 6) | (g << 4) | (g << 2) | g;
            let cb = (b << 6) | (b << 4) | (b << 2) | b;
            let rgb = egui::Color32::from_rgb(cr, cg, cb);
            opaque[color] = rgb;
            transp[color] = if r==0 && g==3 && b == 0 { egui::Color32::TRANSPARENT } else { rgb };
        }
    }

    fn gen_color8_to_rgb_maps(opaque: &mut Vec<egui::Color32>, transp: &mut Vec<egui::Color32>) {
        for color in 0..MAX_COLORS {
            let r = (color as u8) & 0x7;
            let g = (color as u8 >> 3) & 0x7;
            let b = (color as u8 >> 6) & 0x3;
            let cr = (r << 5) | (r << 2) | (r >> 2) | r;
            let cg = (g << 5) | (g << 2) | (g >> 2) | g;
            let cb = (b << 6) | (b << 4) | (b << 2) | b;
            let rgb = egui::Color32::from_rgb(cr, cg, cb);
            opaque[color] = rgb;
            transp[color] = if r==0 && g==7 && b == 0 { egui::Color32::TRANSPARENT } else { rgb };
        }
    }

    pub fn new(bits_per_pixel: u8) -> Self {
        let mut tex_man = TextureManager {
            textures: HashMap::new(),
            color_to_rgb_opaque: vec![egui::Color32::BLACK; MAX_COLORS],
            color_to_rgb_transp: vec![egui::Color32::BLACK; MAX_COLORS],
            bits_per_pixel: 0,
        };
        tex_man.set_bits_per_pixel(bits_per_pixel);
        tex_man
    }

    pub fn set_bits_per_pixel(&mut self, bits_per_pixel: u8) {
        if self.bits_per_pixel != bits_per_pixel {
            self.bits_per_pixel = bits_per_pixel;
            match self.bits_per_pixel {
                8 => Self::gen_color8_to_rgb_maps(&mut self.color_to_rgb_opaque, &mut self.color_to_rgb_transp),
                _ => Self::gen_color6_to_rgb_maps(&mut self.color_to_rgb_opaque, &mut self.color_to_rgb_transp),
            };
            self.clear();
        }
    }

    pub fn clear(&mut self) {
        self.textures.clear();
    }

    pub fn get_rgba_texture(&mut self, ctx: &egui::Context, name: TextureName,
                            w: usize, h: usize, data: &[u8], force_load: bool) -> &egui::TextureHandle {
        let load_image = || {
            let pixels: Vec::<egui::Color32> = data.iter().map(|pix| {
                self.color_to_rgb_opaque[*pix as usize]
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
                self.color_to_rgb_transp[*pix as usize]
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
