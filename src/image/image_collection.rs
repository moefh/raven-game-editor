use std::collections::VecDeque;
use egui::{Rect, Pos2, Vec2};

use super::{TextureManager, TextureName, TextureSlot, ImageRect, ImageFragment, StaticImageData};
use crate::data_asset::{DataAssetId, ImageCollectionAsset};

pub struct ImageCollection {
    pub width: u32,
    pub height: u32,
    pub num_items: u32,
}

impl ImageCollection {
    pub fn from_asset(asset: &impl ImageCollectionAsset) -> Self {
        ImageCollection {
            width: asset.width(),
            height: asset.height(),
            num_items: asset.num_items(),
        }
    }

    pub fn from_static_image(image: &StaticImageData) -> Self {
        ImageCollection {
            width: image.width,
            height: image.height,
            num_items: image.num_items,
        }
    }

    pub fn plus_loaded_texture<'a>(asset: &impl ImageCollectionAsset, tex_man: &'a mut TextureManager, ctx: &egui::Context,
                                   slot: TextureSlot, force_load: bool) -> (Self, &'a egui::TextureHandle) {
        let image = Self::from_asset(asset);
        let tex_name = TextureName::from_asset_id(asset.asset_id(), slot);
        let texture = image.get_or_load_texture(tex_man, ctx, tex_name, asset.data(), force_load);
        (image, texture)
    }

    pub fn plus_texture<'a>(asset: &impl ImageCollectionAsset, tex_man: &'a mut TextureManager, ctx: &egui::Context,
                            slot: TextureSlot) -> (Self, &'a egui::TextureHandle) {
        let image = Self::from_asset(asset);
        let tex_name = TextureName::from_asset_id(asset.asset_id(), slot);
        let texture = image.get_or_load_texture(tex_man, ctx, tex_name, asset.data(), false);
        (image, texture)
    }

    pub fn plus_static_texture<'a>(static_image: &StaticImageData, tex_man: &'a mut TextureManager, ctx: &egui::Context,
                                   slot: TextureSlot) -> (Self, &'a egui::TextureHandle) {
        let image = Self::from_static_image(static_image);
        let tex_name = TextureName::from_static_image_id(static_image.id, slot);
        let texture = image.get_or_load_texture(tex_man, ctx, tex_name, &static_image.data, false);
        (image, texture)
    }

    pub fn texture<'a>(&self, man: &'a mut TextureManager, ctx: &egui::Context,
                       asset: &impl ImageCollectionAsset, slot: TextureSlot) -> &'a egui::TextureHandle {
        let tex_name = TextureName::from_asset_id(asset.asset_id(), slot);
        self.get_or_load_texture(man, ctx, tex_name, asset.data(), false)
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

    fn get_or_load_texture<'a>(&self, man: &'a mut TextureManager, ctx: &egui::Context, tex_name: TextureName,
                               data: &[u8], force_load: bool) -> &'a egui::TextureHandle {
        let width = self.width as usize;
        let height = (self.height * self.num_items) as usize;
        match tex_name.slot {
            TextureSlot::Opaque | TextureSlot::FloatOpaque => {
                man.get_rgba_texture(ctx, tex_name, width, height, data, force_load)
            }
            TextureSlot::Transparent | TextureSlot::FloatTransparent => {
                man.get_rgba_texture_transparent(ctx, tex_name, width, height, data, force_load)
            }
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

    pub fn copy_fragment(&self, id: DataAssetId, data: &[u8], item: u32, rect: ImageRect) -> Option<ImageFragment> {
        if rect.width == 0 || rect.height == 0 || rect.x + rect.width > self.width || rect.y + rect.height > self.height {
            return None;
        }

        let mut copy = vec![0; (rect.width * rect.height) as usize];
        let width = rect.width as usize;
        for y in 0..rect.height {
            let src = ((item * self.height + y + rect.y) * self.width + rect.x) as usize;
            let dest = (y * rect.width) as usize;
            copy[dest..dest + width].clone_from_slice(&data[src..src + width]);
        }
        Some(ImageFragment::new(id, width as u32, rect.height, copy))
    }

    pub fn cut_fragment(&self, id: DataAssetId, data: &mut [u8], item: u32, rect: ImageRect, color: u8) -> Option<ImageFragment> {
        let frag = self.copy_fragment(id, data, item, rect)?;
        let width = frag.width() as usize;
        for y in rect.y..rect.y+frag.height() {
            let index = ((item * self.height + y) * self.width + rect.x) as usize;
            data[index..index+width].fill(color);
        }
        Some(frag)
    }

    pub fn paste_fragment(&self, data: &mut [u8], item: u32, x: i32, y: i32, frag: &ImageFragment, transparent: bool) {
        if (x > 0 &&   x  as u32 >= self.width) || (y > 0 &&   y  as u32 >= self.height) { return; }
        if (x < 0 && (-x) as u32 >= self.width) || (y < 0 && (-y) as u32 >= self.height) { return; }

        let mut src_x = 0;
        let mut src_y = 0;
        let mut width = frag.width();
        let mut height = frag.height();
        let mut x = x;
        let mut y = y;
        if x < 0 { src_x = (-x) as u32; width -= src_x; x = 0; }
        if y < 0 { src_y = (-y) as u32; height -= src_y; y = 0; }
        let x = x as u32;
        let y = y as u32;
        if width > self.width - x { width = self.width - x; }
        if height > self.height - y { height = self.height - y; }

        for iy in 0..height {
            let src = ((iy + src_y) * frag.width() + src_x) as usize;
            let dest = ((item * self.height + y + iy) * self.width + x) as usize;
            let frag_data = frag.data();
            if transparent {
                for ix in 0..width as usize {
                    let pixel = frag_data[src+ix];
                    if pixel != ImageFragment::TRANSPARENT_COLOR {
                        data[dest+ix] = pixel;
                    }
                }
            } else {
                data[dest .. dest + width as usize].clone_from_slice(&frag_data[src .. src + width as usize]);
            }
        }

    }
}
