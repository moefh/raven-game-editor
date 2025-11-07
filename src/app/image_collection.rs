use crate::app::{AppTextureManager, AppTextureName};
use crate::data_asset::{DataAssetId, ImageCollectionAsset};
use egui::{Rect, Pos2, Vec2};

pub struct ImageCollection {
    pub tex_name: AppTextureName,
    pub asset_id: DataAssetId,
    pub width: u32,
    pub height: u32,
    pub num_items: u32,
}

impl ImageCollection {
    pub fn from_asset(asset: &impl ImageCollectionAsset) -> Self {
        ImageCollection {
            tex_name: super::AppTextureName::new(asset.asset_id(), 0),
            asset_id: asset.asset_id(),
            width: asset.width(),
            height: asset.height(),
            num_items: asset.num_items(),
        }
    }

    pub fn load_asset<'a>(asset: &impl ImageCollectionAsset, tex_man: &'a mut super::AppTextureManager, ctx: &egui::Context, force_load: bool)
                          -> (Self, &'a egui::TextureHandle) {
        let image = Self::from_asset(asset);
        let texture = image.get_asset_texture(tex_man, ctx, asset, force_load);
        (image, texture)
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

    pub fn get_asset_texture<'a>(&self, man: &'a mut AppTextureManager, ctx: &egui::Context,
                                 asset: &impl ImageCollectionAsset, force_load: bool) -> &'a egui::TextureHandle {
        if self.asset_id != asset.asset_id() {
            println!("WARNING: get_asset_texture() for wrong asset id: {} vs {}", self.asset_id, asset.asset_id());
        }
        let width = self.width as usize;
        let height = (self.height * self.num_items) as usize;
        man.get_rgba_texture(ctx, self.tex_name, width, height, asset.data(), force_load)
    }

    pub fn set_pixel(&self, data: &mut [u8], x: i32, y: i32, item: u32, color: u8) -> bool {
        if x < 0 || x as u32 >= self.width { return false; }
        if y < 0 || y as u32 >= self.height { return false; }
        if item > self.num_items { return false; }
        let x = x as u32;
        let y = y as u32;
        let index = ((item * self.height + y) * self.width + x) as usize;
        if index > data.len() {
            println!("ERROR: set_pixel(): data is too small: {} vs {}", index, data.len());
            return false;
        }
        if data[index] == color {
            false
        } else {
            data[index] = color;
            true
        }
    }

    pub fn resize(&self, new_width: u32, new_height: u32, new_num_items: u32, data: &mut Vec<u8>, new_pixel: u8) {
        let new_data_len = (new_width as usize) * (new_height as usize) * (new_num_items as usize);

        if self.width == new_width && self.height == new_height {
            // only changing number of elements is faster
            data.resize(new_data_len, new_pixel);
            return;
        }

        let mut new_data = vec![new_pixel; new_data_len];
        for index in 0..data.len().min(new_num_items as usize) {
            for y in 0..self.height.min(new_height) {
                let len = self.width.min(new_width) as usize;
                let src_start = (((index as u32 * self.height) + y) * self.width) as usize;
                let src_end = src_start + len;
                let dst_start = (((index as u32 * new_height) + y) * new_width) as usize;
                let dst_end = dst_start + len;

                new_data.splice(dst_start..dst_end, data[src_start..src_end].iter().copied());
            }
        }
        data.clear();
        data.append(&mut new_data);
    }
}
