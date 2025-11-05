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

pub struct AppTextureManager {
    textures: HashMap<AppTextureName, egui::TextureHandle>,
}

impl AppTextureManager {
    pub fn new() -> Self {
        AppTextureManager {
            textures: HashMap::new(),
        }
    }

    pub fn get_texture(&mut self, ctx: &egui::Context, name: AppTextureName,
                       w: usize, h: usize, stride: usize, data: &[u32]) -> &egui::TextureHandle {
        self.textures.entry(name).or_insert_with(|| {
            let mut pixels = Vec::<u8>::with_capacity(3 * w * h);
            for y in 0..h {
                for (i, &quad) in data[y*stride..(y+1)*stride].iter().enumerate() {
                    let mut src = quad;
                    let n = if (i+1)*4 <= w { 4 } else { (i+1)*4 - w };
                    for _p in 0..n {
                        let pix = src & 0xff;
                        src >>= 8;
                        let r = (pix&0x03) >> 0;
                        let g = (pix&0x0c) >> 2;
                        let b = (pix&0x30) >> 4;
                        pixels.push(((r << 6) | (r << 4) | r) as u8);
                        pixels.push(((g << 6) | (g << 4) | g) as u8);
                        pixels.push(((b << 6) | (b << 4) | b) as u8);
                    }
                }
            }
            ctx.load_texture(
                format!("{:?}", name),
                egui::epaint::ColorImage::from_rgb([w, h], &pixels),
                egui::TextureOptions::NEAREST,
            )
        })
    }
}
