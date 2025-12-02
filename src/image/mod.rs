mod image_collection;
mod texture_manager;
mod static_image_store;

use egui::{Rect, Pos2};

pub use texture_manager::TextureManager;
pub use image_collection::ImageCollection;
pub use static_image_store::StaticImageStore;

use crate::data_asset::{DataAssetId, PropFont};

#[derive(Copy, Clone)]
pub struct ImageRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl ImageRect {
    pub fn from_rect(rect: Rect, image: &impl ImageCollection) -> Self {
        let rect = rect.intersect(Rect::from_min_max(Pos2::ZERO, Pos2::new(image.width() as f32, image.height() as f32)));
        ImageRect {
            x: rect.min.x as u32,
            y: rect.min.y as u32,
            width: rect.width().max(0.0) as u32,
            height: rect.height().max(0.0) as u32,
        }
    }

    pub fn from_image_item(image: &impl ImageCollection) -> Self {
        ImageRect {
            x: 0,
            y: 0,
            width: image.width(),
            height: image.height(),
        }
    }
}

#[derive(Clone)]
pub struct ImagePixels {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl ImagePixels {
    pub const TRANSPARENT_COLOR: u8 = 0b001100;

    pub fn new(width: u32, height: u32, data: Vec<u8>) -> Self {
        ImagePixels {
            width,
            height,
            data,
        }
    }

    pub fn load_png(path: impl AsRef<std::path::Path>) -> Result<ImagePixels, Box<dyn std::error::Error>> {
        let img = ::image::ImageReader::open(path)?.decode()?.to_rgba8();
        let width = img.width();
        let height = img.height();
        let mut data = Vec::with_capacity((width * height) as usize);
        for pixel in img.pixels() {
            if pixel[3] >= 0x80 {
                data.push((pixel[2] >> 2) & 0b110000 |
                          (pixel[1] >> 4) & 0b001100 |
                          (pixel[0] >> 6) & 0b000011);
            } else {
                data.push(ImagePixels::TRANSPARENT_COLOR);
            }
        }
        Ok(ImagePixels {
            width,
            height,
            data,
        })
    }

    pub fn save_prop_font_png(path: impl AsRef<std::path::Path>, pfont: &PropFont) -> Result<(), Box<dyn std::error::Error>> {
        let dst_char_width = pfont.char_widths.iter().max().ok_or(std::io::Error::other("invalid prop font char width")).copied()? as u32;

        let num_items_x = 16;
        let num_items_y = PropFont::NUM_CHARS.div_ceil(num_items_x);
        let dst_w = num_items_x * dst_char_width;
        let dst_h = num_items_y * pfont.height;

        let mut dst = vec![0u8; (4 * dst_w * dst_h) as usize];
        for y_item in 0..num_items_y {
            let dst_item_off_y = dst_w * y_item * pfont.height;
            for x_item in 0..num_items_x {
                if y_item * num_items_x + x_item >= PropFont::NUM_CHARS { break; }
                let src_item_off = (y_item * num_items_x + x_item) * pfont.max_width * pfont.height;
                for y in 0..pfont.height {
                    let dst_off_y = dst_item_off_y + x_item * dst_char_width + dst_w * y;
                    for x in 0..dst_char_width {
                        let dst_off = (4 * (dst_off_y + x)) as usize;
                        let src_off = (src_item_off + y * pfont.max_width + x) as usize;
                        dst[dst_off  ] = 0;
                        dst[dst_off+1] = if pfont.data[src_off] == PropFont::BG_COLOR { 0xff } else { 0 };
                        dst[dst_off+2] = 0;
                        dst[dst_off+3] = 0xff;
                    }
                }
            }
        }
        ::image::save_buffer_with_format(path, &dst, dst_w, dst_h, ::image::ExtendedColorType::Rgba8, ::image::ImageFormat::Png)?;
        Ok(())
    }
}

pub struct ImageFragment {
    pub id: DataAssetId,
    pub pixels: ImagePixels,
    pub changed: bool,
}

impl ImageFragment {
    pub fn new(id: DataAssetId, width: u32, height: u32, data: Vec<u8>) -> Self {
        ImageFragment {
            id,
            pixels: ImagePixels::new(width, height, data),
            changed: true,
        }
    }

    pub fn from_pixels(id: DataAssetId, pixels: ImagePixels) -> Self {
        ImageFragment {
            id,
            pixels,
            changed: true,
        }
    }

    pub fn take_pixels(self) -> ImagePixels {
        self.pixels
    }
}

impl ImageCollection for ImageFragment {
    fn texture_name_id(&self) -> TextureNameId { TextureNameId::Asset(self.id) }
    fn width(&self) -> u32 { self.pixels.width }
    fn height(&self) -> u32 { self.pixels.height }
    fn num_items(&self) -> u32 { 1 }
    fn data(&self) -> &Vec<u8> { &self.pixels.data }
    fn data_mut(&mut self) -> &mut Vec<u8> { &mut self.pixels.data }
}

pub struct StaticImageData {
    pub id: StaticImageId,
    pub pixels: ImagePixels,
    pub num_items: u32,
}

impl StaticImageData {
    pub fn new(id: StaticImageId, width: u32, height: u32, num_items: u32, data: Vec<u8>) -> Self {
        StaticImageData {
            id,
            num_items,
            pixels: ImagePixels {
                width,
                height,
                data,
            },
        }
    }
}

impl ImageCollection for StaticImageData {
    fn texture_name_id(&self) -> TextureNameId { TextureNameId::Static(self.id) }
    fn width(&self) -> u32 { self.pixels.width }
    fn height(&self) -> u32 { self.pixels.height }
    fn num_items(&self) -> u32 { self.num_items }
    fn data(&self) -> &Vec<u8> { &self.pixels.data }
    fn data_mut(&mut self) -> &mut Vec<u8> { &mut self.pixels.data }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TextureSlot {
    Opaque,
    Transparent,
    FloatOpaque,
    FloatTransparent,
}

impl std::fmt::Display for TextureSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TextureSlot::Opaque => write!(f, "op"),
            TextureSlot::Transparent => write!(f, "tr"),
            TextureSlot::FloatOpaque => write!(f, "fl_op"),
            TextureSlot::FloatTransparent => write!(f, "fl_tr"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct StaticImageId(u32);

impl std::fmt::Display for StaticImageId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TextureNameId {
    Asset(DataAssetId),
    Static(StaticImageId),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TextureName {
    pub id: TextureNameId,
    pub slot: TextureSlot,
}

impl TextureName {
    pub fn new(id: TextureNameId, slot: TextureSlot) -> Self {
        TextureName {
            id,
            slot,
        }
    }
}

impl std::fmt::Display for TextureName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.id {
            TextureNameId::Asset(asset_id) => write!(f, "raven://asset/{}/{}", asset_id, self.slot),
            TextureNameId::Static(static_id) => write!(f, "raven://static/{}/{}", static_id, self.slot),
        }
    }
}
