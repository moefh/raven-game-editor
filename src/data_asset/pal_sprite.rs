#[derive(Copy, Clone, PartialEq, Eq, std::hash::Hash)]
pub enum PalSpriteDepth {
    Bpp1,
    Bpp2,
    Bpp4,
}

impl PalSpriteDepth {
    pub fn from_bits_per_pixel(bpp: u32) -> Self {
        match bpp {
            4 => PalSpriteDepth::Bpp4,
            2 => PalSpriteDepth::Bpp2,
            _ => PalSpriteDepth::Bpp1,
        }
    }

    pub fn bits_per_pixel(&self) -> u32 {
        match self {
            PalSpriteDepth::Bpp1 => 1,
            PalSpriteDepth::Bpp2 => 2,
            PalSpriteDepth::Bpp4 => 4,
        }
    }

    pub fn num_colors(&self) -> u32 {
        1 << self.bits_per_pixel()
    }
}

#[derive(std::hash::Hash)]
pub struct PalSprite {
    pub asset: super::DataAsset,
    pub width: u32,
    pub height: u32,
    pub num_frames: u32,
    pub depth: PalSpriteDepth,
    pub palette: Vec<u8>,
    pub color_to_palette_index_map: Vec<u8>,
    pub data: Vec<u8>,
}

pub struct CreationData {
    pub width: u32,
    pub height: u32,
    pub num_frames: u32,
    pub bits_per_pixel: u32,
    pub palette: Vec<u8>,
    pub pixels: Vec<u8>,
}

impl PalSprite {
    pub const EMPTY_COLOR: u8 = 0;

    pub fn new(id: super::DataAssetId, name: String) -> Self {
        let width = 32;
        let height = 32;
        let num_frames = 1;
        let depth = PalSpriteDepth::from_bits_per_pixel(1);
        let palette = Self::gen_palette(depth);
        let color_to_palette_index_map = Self::gen_color_to_palette_index_map(&palette);
        PalSprite {
            asset: super::DataAsset::new(super::DataAssetType::PalSprite, id, name),
            width,
            height,
            num_frames,
            depth,
            palette,
            color_to_palette_index_map,
            data: vec![Self::EMPTY_COLOR; (width*height*num_frames) as usize],
        }
    }

    pub fn color_to_palette_index(&self, color: u8) -> u8 {
        self.color_to_palette_index_map[color as usize]
    }

    pub fn recalculate_color_to_palette_index_map(&mut self) {
        Self::calculate_color_to_palette_index_map(&mut self.color_to_palette_index_map, &self.palette);
    }

    pub fn force_palette(&mut self) -> bool {
        let mut changed = false;
        for pixel in self.data.iter_mut() {
            let pal_index = self.color_to_palette_index_map[*pixel as usize] as usize;
            let new_pixel = self.palette[pal_index % self.palette.len()];
            if *pixel != new_pixel {
                *pixel = new_pixel;
                changed = true;
            }
        }
        changed
    }

    pub fn from_data(id: super::DataAssetId, name: String, data: CreationData) -> Self {
        let color_to_palette_index_map = Self::gen_color_to_palette_index_map(&data.palette);
        PalSprite {
            asset: super::DataAsset::new(super::DataAssetType::PalSprite, id, name),
            width: data.width,
            height: data.height,
            num_frames: data.num_frames,
            depth: PalSpriteDepth::from_bits_per_pixel(data.bits_per_pixel),
            palette: data.palette,
            color_to_palette_index_map,
            data: data.pixels,
        }
    }

    fn gen_palette(depth: PalSpriteDepth) -> Vec<u8> {
        let mut colors = vec![0u8; 16];
        match depth {
            PalSpriteDepth::Bpp1 => { colors[1] = 0xff; }
            PalSpriteDepth::Bpp2 => {
                colors[1] = 0b01_001_001;
                colors[2] = 0b10_110_110;
                colors[3] = 0b11_111_111;
            }
            PalSpriteDepth::Bpp4 => {
                for r in 0..2u8 {
                    for g in 0..2u8 {
                        for b in 0..2u8 {
                            let index = ((b<<4) | (g<<2) | r) as usize;
                            colors[index] = (b<<6) | (g<<3) | r;
                            colors[2*index] = (b<<6) | (g<<3) | r | 0b00_100_100;
                        }
                    }
                }
            }
        }
        colors
    }

    pub fn calculate_color_to_palette_index_map(map: &mut [u8], palette: &[u8]) {
        let color_distance = |color1, color2| {
            let r1 = color1 & 0x7u8;
            let g1 = (color1 >> 3) & 0x7u8;
            let b1 = (color1 >> 6) & 0x3u8;

            let r2 = color2 & 0x7u8;
            let g2 = (color2 >> 3) & 0x7u8;
            let b2 = (color2 >> 6) & 0x3u8;

            let dr = r1 as f32 - r2 as f32;
            let dg = g1 as f32 - g2 as f32;
            let db = (b1 as f32 - b2 as f32) * 2.0f32;
            (dr*dr + dg*dg + db*db).sqrt()
        };

        for (color, map_to_index) in map.iter_mut().enumerate() {
            let best_match = palette.iter().enumerate().min_by(|(_, c1), (_, c2)| {
                let d1 = color_distance(*c1, (color & 0xff) as u8);
                let d2 = color_distance(*c2, (color & 0xff) as u8);
                d1.total_cmp(&d2)
            });
            if let Some((best_pal_index, _)) = best_match {
                *map_to_index = (best_pal_index & 0xff) as u8;
            } else {
                *map_to_index = 0;
            }
        }
    }

    pub fn gen_color_to_palette_index_map(palette: &[u8]) -> Vec<u8> {
        let mut map = vec![0u8; 256];
        Self::calculate_color_to_palette_index_map(&mut map, palette);
        map
    }
}

impl super::DuplicableAsset<PalSprite> for PalSprite {
    fn duplicate(&self, dup_id: super::DataAssetId, dup_name: String) -> Self {
        PalSprite {
            asset: self.asset.duplicate(dup_id, dup_name),
            width: self.width,
            height: self.height,
            num_frames: self.num_frames,
            depth: self.depth,
            palette: self.palette.clone(),
            color_to_palette_index_map: self.color_to_palette_index_map.clone(),
            data: self.data.clone(),
        }
    }
}

impl super::GenericAsset for PalSprite {
    fn asset(&self) -> &super::DataAsset { &self.asset }

    fn data_size(&self) -> usize {
        // header: w(2) + h(2) + num_frames(2) + depth(2) + palette(16) + data<ptr>(4)
        let header = 2usize * 4usize + 16usize + 4usize;

        // image: ceil((depth * width) / 8) * height * num_frames
        let image = ((self.depth.bits_per_pixel() * self.width).div_ceil(8) * self.height * self.num_frames) as usize;

        header + image
    }
}
