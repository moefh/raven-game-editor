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

impl ImageCollection {
    pub fn tileset<'a>(tex_man: &'a mut super::AppTextureManager, ctx: &egui::Context, tileset: &Tileset, force_load: bool)
                       -> (ImageCollection, &'a egui::TextureHandle) {
        let image = Self::from_tileset(tileset, super::AppTextureName::new(tileset.asset.id, 0));
        let texture = image.get_tileset_texture(tex_man, ctx, tileset, force_load);
        (image, texture)
    }

    pub fn sprite<'a>(tex_man: &'a mut super::AppTextureManager, ctx: &egui::Context, sprite: &Sprite, force_load: bool)
                      -> (ImageCollection, &'a egui::TextureHandle) {
        let image = Self::from_sprite(sprite, super::AppTextureName::new(sprite.asset.id, 0));
        let texture = image.get_sprite_texture(tex_man, ctx, sprite, force_load);
        (image, texture)
    }

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

    pub fn get_item_size(&self) -> Vec2 {
        Vec2::new(self.width as f32, self.height as f32)
    }

    pub fn get_full_size(&self) -> Vec2 {
        Vec2::new(self.width as f32, (self.height * self.num_items) as f32)
    }

    pub fn get_item_uv(&self, item: u32) -> Rect {
        let item = if item > self.num_items { self.num_items - 1 } else { item };
        Rect {
            min: Pos2::new(0.0, item as f32 / self.num_items as f32),
            max: Pos2::new(1.0, (item+1) as f32 / self.num_items as f32),
        }
    }

    pub fn get_tileset_texture<'a>(&self, man: &'a mut AppTextureManager, ctx: &egui::Context,
                                   tileset: &Tileset, force_load: bool) -> &'a egui::TextureHandle {
        if self.asset_id != tileset.asset.id {
            println!("WARNING: get_tileset_texture() for wrong tileset id: {} vs {}", self.asset_id, tileset.asset.id);
        }
        let width = self.width as usize;
        let height = (self.height * self.num_items) as usize;
        man.get_rgba_texture(ctx, self.tex, width, height, &tileset.data, force_load)
    }

    pub fn get_sprite_texture<'a>(&self, man: &'a mut AppTextureManager, ctx: &egui::Context,
                                  sprite: &Sprite, force_load: bool) -> &'a egui::TextureHandle {
        if self.asset_id != sprite.asset.id {
            println!("WARNING: get_sprite_texture() for wrong sprite: {} vs {}", self.asset_id, sprite.asset.id);
        }
        let width = self.width as usize;
        let height = (self.height * self.num_items) as usize;
        man.get_rgba_texture(ctx, self.tex, width, height, &sprite.data, force_load)
    }

    pub fn set_pixel(&self, data: &mut [u32], x: i32, y: i32, item: u32, color: u8) -> bool {
        if x < 0 || x as u32 >= self.width { return false; }
        if y < 0 || y as u32 >= self.height { return false; }
        if item > self.num_items { return false; }
        let x = x as u32;
        let y = y as u32;
        let color = color as u32;
        let index = ((item * self.height + y) * self.stride + x / 4) as usize;
        let quad = data[index];
        let new_quad = match x % 4 {
            0 => (quad & 0xffffff00) | color,
            1 => (quad & 0xffff00ff) | (color << 8),
            2 => (quad & 0xff00ffff) | (color << 16),
            3 => (quad & 0x00ffffff) | (color << 24),
            _ => quad,
        };
        data[index] = new_quad;
        new_quad != quad
    }
}
