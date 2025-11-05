use crate::app::{AppTextureManager, AppTextureName};
use crate::data_asset::{DataAssetId, Tileset, Sprite};
use egui::{Rect, Pos2, Vec2};

pub struct ImageCollection {
    pub tex: AppTextureName,
    pub asset_id: DataAssetId,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub num_items: u32,
}

#[allow(dead_code)]
impl ImageCollection {
    pub fn from_tileset(tileset: &Tileset, tex: AppTextureName) -> Self {
        ImageCollection {
            tex,
            asset_id: tileset.asset.id,
            width: tileset.width,
            height: tileset.height,
            stride: tileset.stride,
            num_items: tileset.num_tiles,
        }
    }

    pub fn from_sprite(sprite: &Sprite, tex: AppTextureName) -> Self {
        ImageCollection {
            tex,
            asset_id: sprite.asset.id,
            width: sprite.width,
            height: sprite.height,
            stride: sprite.stride,
            num_items: sprite.num_frames,
        }
    }

    pub fn get_num_items(&self) -> u32 {
        self.num_items as u32
    }
    
    pub fn get_item_size(&self) -> Vec2 {
        Vec2::new(self.width as f32, self.height as f32)
    }

    pub fn get_item_uv(&self, item: u32) -> Rect {
        let item = if item > self.num_items { self.num_items - 1 } else { item };
        Rect {
            min: Pos2::new(0.0, item as f32 / self.num_items as f32),
            max: Pos2::new(1.0, (item+1) as f32 / self.num_items as f32),
        }
    }

    pub fn get_tileset_texture<'a>(&self, man: &'a mut AppTextureManager, ctx: &egui::Context,
                               tileset: &Tileset) -> &'a egui::TextureHandle {
        if self.asset_id != tileset.asset.id {
            print!("WARNING: get_tileset_texture() for wrong tileset id: {} vs {}\n", self.asset_id, tileset.asset.id);
        }
        let width = self.width as usize;
        let height = (self.height * self.num_items) as usize;
        let stride = self.stride as usize;
        man.get_texture(ctx, self.tex, width, height, stride, &tileset.data)
    }

    pub fn get_sprite_texture<'a>(&self, man: &'a mut AppTextureManager, ctx: &egui::Context,
                              sprite: &Sprite) -> &'a egui::TextureHandle {
        if self.asset_id != sprite.asset.id {
            print!("WARNING: get_sprite_texture() for wrong sprite: {} vs {}\n", self.asset_id, sprite.asset.id);
        }
        let width = self.width as usize;
        let height = (self.height * self.num_items) as usize;
        let stride = self.stride as usize;
        man.get_texture(ctx, self.tex, width, height, stride, &sprite.data)
    }
}
