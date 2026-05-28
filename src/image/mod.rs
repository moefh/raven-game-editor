mod image_collection;
mod image_collection_io;
mod texture_manager;
mod static_image_store;
mod image_pixels;
pub mod colors;

use egui::{Rect, Pos2};

pub use texture_manager::TextureManager;
pub use image_collection::ImageCollection;
pub use image_collection_io::ImageCollectionIO;
pub use static_image_store::StaticImageStore;
pub use image_pixels::{ImagePixels, ImagePixelsCollection};

use crate::data_asset::DataAssetId;

pub enum ImageRotation {
    CW90,  // 90 degrees clockwise
    CCW90, // 90 degrees counter-clockwise
}

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

#[derive(Debug)]
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

    pub fn set_changed(&mut self) {
        self.changed = true;
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
