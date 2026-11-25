use super::{
    default_texture_slot,
    ImagePixelsCollection,
    ImageCollection,
    DynamicImageId,
    TextureNameId,
    TextureSlot,
};

pub struct DynamicImage {
    pub texture_name_id: TextureNameId,
    pub pixels: ImagePixelsCollection,
}

impl DynamicImage {
    pub fn new(width: u32, height: u32, num_items: u32) -> Self {
        DynamicImage {
            texture_name_id: TextureNameId::Dynamic(DynamicImageId::gen_id()),
            pixels: ImagePixelsCollection::new(width, height, num_items),
        }
    }
}

impl ImageCollection for DynamicImage {
    fn texture_name_id(&self) -> TextureNameId { self.texture_name_id }
    fn texture_slot(&self, transparent: bool, float: bool) -> TextureSlot { default_texture_slot(transparent, float) }
    fn width(&self) -> u32 { self.pixels.width }
    fn height(&self) -> u32 { self.pixels.height }
    fn num_items(&self) -> u32 { self.pixels.num_items }
    fn set_width(&mut self, width: u32) { self.pixels.width = width; }
    fn set_height(&mut self, height: u32) { self.pixels.height = height; }
    fn set_num_items(&mut self, num_items: u32) { self.pixels.num_items = num_items; }
    fn data(&self) -> &Vec<u8> { &self.pixels.data }
    fn data_mut(&mut self) -> &mut Vec<u8> { &mut self.pixels.data }
}
