mod image_collection;
mod texture_manager;
mod static_image_store;

use egui::{Rect, Pos2};
use crate::data_asset::ImageCollectionAsset;

pub use texture_manager::TextureManager;
pub use image_collection::ImageCollection;
pub use static_image_store::StaticImageStore;

use crate::data_asset::DataAssetId;

#[derive(Copy, Clone)]
pub struct ImageRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl ImageRect {
    pub fn from_rect(rect: Rect, image: &ImageCollection) -> Self {
        let rect = rect.intersect(Rect::from_min_max(Pos2::ZERO, Pos2::new(image.width as f32, image.height as f32)));
        ImageRect {
            x: rect.min.x as u32,
            y: rect.min.y as u32,
            width: rect.width().max(0.0) as u32,
            height: rect.height().max(0.0) as u32,
        }
    }
}

pub struct ImageFragment {
    pub id: DataAssetId,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub changed: bool,
}

impl ImageFragment {
    pub fn new(id: DataAssetId, width: u32, height: u32, data: Vec<u8>) -> Self {
        ImageFragment {
            id,
            width,
            height,
            data,
            changed: true,
        }
    }
}

impl ImageCollectionAsset for ImageFragment {
    fn asset_id(&self) -> DataAssetId { self.id }
    fn width(&self) -> u32 { self.width }
    fn height(&self) -> u32 { self.height }
    fn num_items(&self) -> u32 { 1 }
    fn data(&self) -> &[u8] { &self.data }
    fn data_mut(&mut self) -> &mut [u8] { &mut self.data }
}

pub struct StaticImageData {
    pub id: StaticImageId,
    pub width: u32,
    pub height: u32,
    pub num_items: u32,
    pub data: Vec<u8>,
}

impl StaticImageData {
    pub fn new(id: StaticImageId, width: u32, height: u32, num_items: u32, data: Vec<u8>) -> Self {
        StaticImageData {
            id,
            width,
            height,
            num_items,
            data,
        }
    }
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
    pub fn from_asset_id(asset_id: DataAssetId, slot: TextureSlot) -> Self {
        TextureName {
            id: TextureNameId::Asset(asset_id),
            slot,
        }
    }

    pub fn from_static_image_id(static_image_id: StaticImageId, slot: TextureSlot) -> Self {
        TextureName {
            id: TextureNameId::Static(static_image_id),
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
