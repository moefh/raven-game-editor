use std::collections::HashMap;
use crate::data_asset::DataAssetId;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TextureName {
    pub asset_id: DataAssetId,
    pub index: u32,
}

impl TextureName {
    pub fn new(asset_id: DataAssetId, index: u32) -> Self {
        TextureName {
            asset_id,
            index,
        }
    }
}

impl std::fmt::Display for TextureName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "raven://asset/{}/{}", self.asset_id, self.index)
    }
}

pub struct TextureManager {
    textures: HashMap<TextureName, egui::TextureHandle>,
}

impl TextureManager {
    pub fn new() -> Self {
        TextureManager {
            textures: HashMap::new(),
        }
    }

    pub fn get_rgba_texture(&mut self, ctx: &egui::Context, name: TextureName,
                            w: usize, h: usize, data: &[u8], force_load: bool) -> &egui::TextureHandle {
        let load_image = || {
            let pixels: Vec::<egui::Color32> = data.iter().map(|pix| {
                let r = pix&0x03;
                let g = (pix&0x0c) >> 2;
                let b = (pix&0x30) >> 4;
                if r==0 && g==3 && b == 0 {
                    egui::Color32::TRANSPARENT
                } else {
                    let cr = (r << 6) | (r << 4) | (r << 2) | r;
                    let cg = (g << 6) | (g << 4) | (g << 2) | g;
                    let cb = (b << 6) | (b << 4) | (b << 2) | b;
                    egui::Color32::from_rgb(cr, cg, cb)
                }
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
