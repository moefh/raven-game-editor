use std::collections::VecDeque;
use crate::misc::{TextureManager, TextureName, TextureSlot};
use crate::data_asset::{DataAssetId, ImageCollectionAsset};
use egui::{Rect, Pos2, Vec2};

pub struct ImageCollection {
    pub asset_id: DataAssetId,
    pub width: u32,
    pub height: u32,
    pub num_items: u32,
}

impl ImageCollection {
    pub fn from_asset(asset: &impl ImageCollectionAsset) -> Self {
        ImageCollection {
            asset_id: asset.asset_id(),
            width: asset.width(),
            height: asset.height(),
            num_items: asset.num_items(),
        }
    }

    pub fn load_asset<'a>(asset: &impl ImageCollectionAsset, tex_man: &'a mut TextureManager, ctx: &egui::Context, force_load: bool)
                          -> (Self, &'a egui::TextureHandle) {
        let image = Self::from_asset(asset);
        let texture = image.get_asset_texture(tex_man, ctx, asset, TextureSlot::Opaque, force_load);
        (image, texture)
    }

    pub fn load_asset_on_slot<'a>(asset: &impl ImageCollectionAsset, tex_man: &'a mut TextureManager, ctx: &egui::Context,
                                  slot: TextureSlot, force_load: bool) -> (Self, &'a egui::TextureHandle) {
        let image = Self::from_asset(asset);
        let texture = image.get_asset_texture(tex_man, ctx, asset, slot, force_load);
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

    pub fn get_asset_texture<'a>(&self, man: &'a mut TextureManager, ctx: &egui::Context,
                                 asset: &impl ImageCollectionAsset, slot: TextureSlot, force_load: bool) -> &'a egui::TextureHandle {
        if self.asset_id != asset.asset_id() {
            println!("WARNING: get_asset_texture() for wrong asset id: {} vs {}", self.asset_id, asset.asset_id());
        }
        let width = self.width as usize;
        let height = (self.height * self.num_items) as usize;
        let name = TextureName::new(asset.asset_id(), slot);
        match slot {
            TextureSlot::Opaque => man.get_rgba_texture(ctx, name, width, height, asset.data(), force_load),
            TextureSlot::Transparent => man.get_rgba_texture_transparent(ctx, name, width, height, asset.data(), force_load),
        }
    }

    pub fn get_pixel(&self, data: &[u8], x: i32, y: i32, item: u32) -> u8 {
        if x < 0 || x as u32 >= self.width { return 0; }
        if y < 0 || y as u32 >= self.height { return 0; }
        if item > self.num_items { return 0; }
        data[((item * self.height + y as u32) * self.width + x as u32) as usize]
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

    fn flood_fill_scan(&self, data: &mut [u8], item: u32, work: &mut VecDeque<(i32,i32)>, fill_over: u8,
                       range: std::ops::RangeInclusive<i32>, y: i32) {
        let mut span_added = false;
        for x in range {
            if fill_over != self.get_pixel(data, x, y, item) {
                span_added = false;
            } else if ! span_added {
                work.push_back((x, y));
                span_added = true;
            }
        }
    }

    pub fn flood_fill(&self, data: &mut [u8], x: i32, y: i32, item: u32, color: u8) -> bool {
        if x < 0 || x as u32 >= self.width { return false; }
        if y < 0 || y as u32 >= self.height { return false; }
        if item > self.num_items { return false; }
        let fill_over = self.get_pixel(data, x, y, item);
        if fill_over == color { return false; }

        let width = self.width as i32;
        let height = self.height as i32;

        let mut work = VecDeque::new();
        work.push_back((x, y));
        while let Some((x, y)) = work.pop_front() {
            // left
            let mut lx = x;
            while lx > 0 && fill_over == self.get_pixel(data, lx-1, y, item) {
                lx -= 1;
                self.set_pixel(data, lx, y, item, color);
            }

            // right
            let mut rx = x;
            while rx < width && fill_over == self.get_pixel(data, rx, y, item) {
                self.set_pixel(data, rx, y, item, color);
                rx += 1;
            }

            if y < height - 1 { self.flood_fill_scan(data, item, &mut work, fill_over, lx..=rx-1, y+1); }
            if y > 0 { self.flood_fill_scan(data, item, &mut work, fill_over, lx..=rx-1, y-1); }
        }
        true
    }

    pub fn h_flip(&self, data: &mut [u8], item: u32) {
        if item > self.num_items { return; }

        let item = item as usize;
        let width = self.width as usize;
        let height = self.height as usize;

        let mut line = vec![0; width];
        for y in 0..height {
            let left = (item * height + y) * width;
            line[..].clone_from_slice(&data[left..left+width]);
            for x in 0..width {
                data[left+x] = line[width-1-x];
            }
        }
    }

    pub fn v_flip(&self, data: &mut [u8], item: u32) {
        if item > self.num_items { return; }

        let item = item as usize;
        let width = self.width as usize;
        let height = self.height as usize;

        let mut top_line = vec![0; width];
        let mut bot_line = vec![0; width];
        for y in 0..height/2 {
            let top = (item * height + y) * width;
            let bot = (item * height + height - 1 - y) * width;
            top_line[..].clone_from_slice(&data[top..top+width]);
            bot_line[..].clone_from_slice(&data[bot..bot+width]);
            data[top..top+width].clone_from_slice(&bot_line);
            data[bot..bot+width].clone_from_slice(&top_line);
        }
    }
}
