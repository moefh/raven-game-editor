use std::collections::VecDeque;
use egui::{Rect, Pos2, Vec2};

use super::{TextureManager, TextureName, TextureNameId, TextureSlot, ImageRect, ImageFragment, ImagePixels};
use crate::data_asset::{DataAssetId, Tileset, Sprite, Font, PropFont};

pub trait ImageCollection {
    fn texture_name_id(&self) -> TextureNameId;
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn num_items(&self) -> u32;
    fn data(&self) -> &Vec<u8>;
    fn data_mut(&mut self) -> &mut Vec<u8>;

    fn load_texture<'a>(&self, tex_man: &'a mut TextureManager, ctx: &egui::Context, slot: TextureSlot, force_load: bool)
                        -> &'a egui::TextureHandle {
        self.get_or_load_texture(tex_man, ctx, slot, force_load)
    }

    fn texture<'a>(&self, tex_man: &'a mut TextureManager, ctx: &egui::Context, slot: TextureSlot) -> &'a egui::TextureHandle {
        self.get_or_load_texture(tex_man, ctx, slot, false)
    }

    fn get_item_size(&self) -> Vec2 {
        Vec2::new(self.width() as f32, self.height() as f32)
    }

    fn get_full_size(&self) -> Vec2 {
        Vec2::new(self.width() as f32, (self.height() * self.num_items()) as f32)
    }

    fn get_item_uv(&self, item: u32) -> Rect {
        let item = if item > self.num_items() { self.num_items() - 1 } else { item };
        Rect {
            min: Pos2::new(0.0, item as f32 / self.num_items() as f32),
            max: Pos2::new(1.0, (item+1) as f32 / self.num_items() as f32),
        }
    }

    fn get_or_load_texture<'a>(&self, man: &'a mut TextureManager, ctx: &egui::Context, slot: TextureSlot, force_load: bool)
                               -> &'a egui::TextureHandle {
        let width = self.width() as usize;
        let height = (self.height() * self.num_items()) as usize;
        let tex_name = TextureName::new(self.texture_name_id(), slot);
        match slot {
            TextureSlot::Opaque | TextureSlot::FloatOpaque => {
                man.get_rgba_texture(ctx, tex_name, width, height, self.data(), force_load)
            }
            TextureSlot::Transparent | TextureSlot::FloatTransparent => {
                man.get_rgba_texture_transparent(ctx, tex_name, width, height, self.data(), force_load)
            }
        }
    }

    fn get_pixel(&self, x: i32, y: i32, item: u32) -> u8 {
        if x < 0 || x as u32 >= self.width() { return 0; }
        if y < 0 || y as u32 >= self.height() { return 0; }
        if item > self.num_items() { return 0; }
        self.data()[((item * self.height() + y as u32) * self.width() + x as u32) as usize]
    }

    fn set_pixel(&mut self, x: i32, y: i32, item: u32, color: u8) -> bool {
        if x < 0 || x as u32 >= self.width() { return false; }
        if y < 0 || y as u32 >= self.height() { return false; }
        if item > self.num_items() { return false; }
        let x = x as u32;
        let y = y as u32;
        let index = ((item * self.height() + y) * self.width() + x) as usize;
        let data = self.data_mut();
        if index > data.len() { return false; }
        if data[index] == color {
            false
        } else {
            data[index] = color;
            true
        }
    }

    fn resize(&mut self, new_width: u32, new_height: u32, new_num_items: u32, new_pixel: u8) {
        let new_data_len = (new_width as usize) * (new_height as usize) * (new_num_items as usize);

        if self.width() == new_width && self.height() == new_height {
            // only changing number of elements is faster
            self.data_mut().resize(new_data_len, new_pixel);
            return;
        }

        let height = self.height();
        let width = self.width();
        let data = self.data_mut();
        let mut new_data = vec![new_pixel; new_data_len];
        for index in 0..data.len().min(new_num_items as usize) {
            for y in 0..height.min(new_height) {
                let len = width.min(new_width) as usize;
                let src_start = (((index as u32 * height) + y) * width) as usize;
                let src_end = src_start + len;
                let dst_start = (((index as u32 * new_height) + y) * new_width) as usize;
                let dst_end = dst_start + len;

                new_data.splice(dst_start..dst_end, data[src_start..src_end].iter().copied());
            }
        }
        data.clear();
        data.append(&mut new_data);
    }

    fn flood_fill_scan(&mut self, item: u32, work: &mut VecDeque<(i32,i32)>, fill_over: u8,
                       range: std::ops::RangeInclusive<i32>, y: i32) {
        let mut span_added = false;
        for x in range {
            if fill_over != self.get_pixel(x, y, item) {
                span_added = false;
            } else if ! span_added {
                work.push_back((x, y));
                span_added = true;
            }
        }
    }

    fn flood_fill(&mut self, x: i32, y: i32, item: u32, color: u8) -> bool {
        if x < 0 || x as u32 >= self.width() { return false; }
        if y < 0 || y as u32 >= self.height() { return false; }
        if item > self.num_items() { return false; }
        let fill_over = self.get_pixel(x, y, item);
        if fill_over == color { return false; }

        let width = self.width() as i32;
        let height = self.height() as i32;

        let mut work = VecDeque::new();
        work.push_back((x, y));
        while let Some((x, y)) = work.pop_front() {
            // left
            let mut lx = x;
            while lx > 0 && fill_over == self.get_pixel(lx-1, y, item) {
                lx -= 1;
                self.set_pixel(lx, y, item, color);
            }

            // right
            let mut rx = x;
            while rx < width && fill_over == self.get_pixel(rx, y, item) {
                self.set_pixel(rx, y, item, color);
                rx += 1;
            }

            if y < height - 1 { self.flood_fill_scan(item, &mut work, fill_over, lx..=rx-1, y+1); }
            if y > 0 { self.flood_fill_scan(item, &mut work, fill_over, lx..=rx-1, y-1); }
        }
        true
    }

    fn h_flip(&mut self, item: u32) {
        if item > self.num_items() { return; }

        let item = item as usize;
        let width = self.width() as usize;
        let height = self.height() as usize;

        let data = self.data_mut();
        let mut line = vec![0; width];
        for y in 0..height {
            let left = (item * height + y) * width;
            line[..].clone_from_slice(&data[left..left+width]);
            for x in 0..width {
                data[left+x] = line[width-1-x];
            }
        }
    }

    fn v_flip(&mut self, item: u32) {
        if item > self.num_items() { return; }

        let item = item as usize;
        let width = self.width() as usize;
        let height = self.height() as usize;

        let data = self.data_mut();
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

    fn copy_fragment(&self, id: DataAssetId, item: u32, rect: ImageRect) -> Option<ImageFragment> {
        if rect.width == 0 || rect.height == 0 || rect.x + rect.width > self.width() || rect.y + rect.height > self.height() {
            return None;
        }

        let data = self.data();
        let mut copy = vec![0; (rect.width * rect.height) as usize];
        let width = rect.width as usize;
        for y in 0..rect.height {
            let src = ((item * self.height() + y + rect.y) * self.width() + rect.x) as usize;
            let dest = (y * rect.width) as usize;
            copy[dest..dest + width].clone_from_slice(&data[src..src + width]);
        }
        Some(ImageFragment::new(id, width as u32, rect.height, copy))
    }

    fn cut_fragment(&mut self, id: DataAssetId, item: u32, rect: ImageRect, color: u8) -> Option<ImageFragment> {
        let frag = self.copy_fragment(id, item, rect)?;
        let width = frag.width() as usize;
        let self_width = self.width();
        let self_height = self.height();
        let data = self.data_mut();
        for y in rect.y..rect.y+frag.height() {
            let index = ((item * self_height + y) * self_width + rect.x) as usize;
            data[index..index+width].fill(color);
        }
        Some(frag)
    }

    fn paste_fragment(&mut self, item: u32, x: i32, y: i32, frag: &ImageFragment, transparent: bool) {
        if (x > 0 &&   x  as u32 >= self.width()) || (y > 0 &&   y  as u32 >= self.height()) { return; }
        if (x < 0 && (-x) as u32 >= self.width()) || (y < 0 && (-y) as u32 >= self.height()) { return; }

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
        if width > self.width() - x { width = self.width() - x; }
        if height > self.height() - y { height = self.height() - y; }

        let self_width = self.width();
        let self_height = self.height();
        let data = self.data_mut();
        for iy in 0..height {
            let src = ((iy + src_y) * frag.width() + src_x) as usize;
            let dest = ((item * self_height + y + iy) * self_width + x) as usize;
            let frag_data = frag.data();
            if transparent {
                for ix in 0..width as usize {
                    let pixel = frag_data[src+ix];
                    if pixel != ImagePixels::TRANSPARENT_COLOR {
                        data[dest+ix] = pixel;
                    }
                }
            } else {
                data[dest .. dest + width as usize].clone_from_slice(&frag_data[src .. src + width as usize]);
            }
        }

    }

    fn save_image_png(&self, path: impl AsRef<std::path::Path>, num_items_x: u32) -> Result<(), Box<dyn std::error::Error>> {
        fn conv_pixel(pixel: u8) -> [u8; 4] {
            let r = (pixel     ) & 0b11;
            let g = (pixel >> 2) & 0b11;
            let b = (pixel >> 4) & 0b11;
            [
                (r << 6) | (r << 4) | (r << 2) | r,
                (g << 6) | (g << 4) | (g << 2) | g,
                (b << 6) | (b << 4) | (b << 2) | b,
                if r == 0 && g == 0b11 && b == 0 { 0 } else { 255 },
            ]
        }
        self.save_png(path, num_items_x, conv_pixel)
    }

    fn save_font_png(&self, path: impl AsRef<std::path::Path>, num_items_x: u32) -> Result<(), Box<dyn std::error::Error>> {
        fn conv_pixel(pixel: u8) -> [u8; 4] {
            if pixel == Font::BG_COLOR {
                [0, 0xff, 0, 0xff]
            } else {
                [0, 0, 0, 0xff]
            }
        }
        self.save_png(path, num_items_x, conv_pixel)
    }

    fn save_png<F: Fn(u8) -> [u8; 4]>(&self, path: impl AsRef<std::path::Path>, num_items_x: u32, conv_pixel: F)
                                      -> Result<(), Box<dyn std::error::Error>> {
        if num_items_x > self.num_items() {
            Err(std::io::Error::other(format!("invalid horizontal size: {}", num_items_x)))?;
        }
        let num_items_y = self.num_items().div_ceil(num_items_x);
        let dst_w = num_items_x * self.width();
        let dst_h = num_items_y * self.height();

        let data = self.data();
        let mut dst = vec![0u8; (4 * dst_w * dst_h) as usize];
        for y_item in 0..num_items_y {
            let dst_item_off_y = dst_w * y_item * self.height();
            for x_item in 0..num_items_x {
                if y_item * num_items_x + x_item >= self.num_items() { break; }
                let src_item_off = (y_item * num_items_x + x_item) * self.width() * self.height();
                for y in 0..self.height() {
                    let dst_off_y = dst_item_off_y + x_item * self.width() + dst_w * y;
                    for x in 0..self.width() {
                        let dst_off = (4 * (dst_off_y + x)) as usize;
                        let src_off = (src_item_off + y * self.width() + x) as usize;
                        let [r, g, b, a] = conv_pixel(data[src_off]);
                        dst[dst_off  ] = r;
                        dst[dst_off+1] = g;
                        dst[dst_off+2] = b;
                        dst[dst_off+3] = a;
                    }
                }
            }
        }
        ::image::save_buffer_with_format(path, &dst, dst_w, dst_h, ::image::ExtendedColorType::Rgba8, ::image::ImageFormat::Png)?;
        Ok(())
    }
}

impl ImageCollection for Sprite {
    fn texture_name_id(&self) -> TextureNameId { TextureNameId::Asset(self.asset.id) }
    fn width(&self) -> u32 { self.width }
    fn height(&self) -> u32 { self.height }
    fn num_items(&self) -> u32 { self.num_frames }
    fn data(&self) -> &Vec<u8> { &self.data }
    fn data_mut(&mut self) -> &mut Vec<u8> { &mut self.data }
}

impl ImageCollection for Tileset {
    fn texture_name_id(&self) -> TextureNameId { TextureNameId::Asset(self.asset.id) }
    fn width(&self) -> u32 { self.width }
    fn height(&self) -> u32 { self.height }
    fn num_items(&self) -> u32 { self.num_tiles }
    fn data(&self) -> &Vec<u8> { &self.data }
    fn data_mut(&mut self) -> &mut Vec<u8> { &mut self.data }
}

impl ImageCollection for Font {
    fn texture_name_id(&self) -> TextureNameId { TextureNameId::Asset(self.asset.id) }
    fn width(&self) -> u32 { self.width }
    fn height(&self) -> u32 { self.height }
    fn num_items(&self) -> u32 { Font::NUM_CHARS }
    fn data(&self) -> &Vec<u8> { &self.data }
    fn data_mut(&mut self) -> &mut Vec<u8> { &mut self.data }
}

impl ImageCollection for PropFont {
    fn texture_name_id(&self) -> TextureNameId { TextureNameId::Asset(self.asset.id) }
    fn width(&self) -> u32 { self.max_width }
    fn height(&self) -> u32 { self.height }
    fn num_items(&self) -> u32 { PropFont::NUM_CHARS }
    fn data(&self) -> &Vec<u8> { &self.data }
    fn data_mut(&mut self) -> &mut Vec<u8> { &mut self.data }
}
