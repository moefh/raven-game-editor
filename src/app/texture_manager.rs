use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct AppTextureName {
    pub asset_id: super::DataAssetId,
    pub index: u32,
}

impl AppTextureName {
    pub fn new(asset_id: super::DataAssetId, index: u32) -> Self {
        AppTextureName {
            asset_id,
            index,
        }
    }
}

impl std::fmt::Display for AppTextureName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "raven://asset/{}/{}", self.asset_id, self.index)
    }
}

pub struct AppTextureManager {
    textures: HashMap<AppTextureName, egui::TextureHandle>,
}

impl AppTextureManager {
    pub fn new() -> Self {
        AppTextureManager {
            textures: HashMap::new(),
        }
    }

    /*
    pub fn get_texture(&mut self, ctx: &egui::Context, name: AppTextureName,
                       w: usize, h: usize, stride: usize, data: &[u32], force_load: bool) -> &egui::TextureHandle {
        let load_image = || {
            let mut pixels = Vec::<u8>::with_capacity(3 * w * h);
            for y in 0..h {
                for (i, &quad) in data[y*stride..(y+1)*stride].iter().enumerate() {
                    let mut src = quad;
                    let n = if (i+1)*4 <= w { 4 } else { (i+1)*4 - w };
                    for _p in 0..n {
                        let pix = src & 0xff;
                        src >>= 8;
                        let r = pix&0x03;
                        let g = (pix&0x0c) >> 2;
                        let b = (pix&0x30) >> 4;
                        pixels.push(((r << 6) | (r << 4) | (r << 2) | r) as u8);
                        pixels.push(((g << 6) | (g << 4) | (g << 2) | g) as u8);
                        pixels.push(((b << 6) | (b << 4) | (b << 2) | b) as u8);
                   }
                }
            }
            ctx.load_texture(
                format!("{}", name),
                egui::epaint::ColorImage::from_rgb([w, h], &pixels),
                egui::TextureOptions::NEAREST,
            )
        };
        if force_load {
            //self.textures.insert(name, load_image());
            //self.textures.get(&name).unwrap()
            self.textures.entry(name).insert_entry(load_image()).into_mut()
        } else {
            self.textures.entry(name).or_insert_with(load_image)
        }
    }
    */

    pub fn get_rgba_texture(&mut self, ctx: &egui::Context, name: AppTextureName,
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
            //self.textures.insert(name, load_image());
            //self.textures.get(&name).unwrap()
            self.textures.entry(name).insert_entry(load_image()).into_mut()
        } else {
            self.textures.entry(name).or_insert_with(load_image)
        }
    }
}
