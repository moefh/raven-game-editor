use super::{
    default_texture_slot,
    ImageCollection,
    TextureNameId,
    TextureSlot,
};

pub struct ImageCollectionAltId<'a, Image> {
    pub texture_name_id: TextureNameId,
    pub image: &'a Image,
    dummy_data: Vec<u8>,
}

impl<'a, Image: ImageCollection> ImageCollectionAltId<'a, Image> {
    pub fn new(texture_name_id: TextureNameId, image: &'a Image) -> Self {
        ImageCollectionAltId {
            texture_name_id,
            image,
            dummy_data: Vec::new(),
        }
    }
}

impl<'a, Image: ImageCollection> ImageCollection for ImageCollectionAltId<'a, Image> {
    fn texture_name_id(&self) -> TextureNameId { self.texture_name_id }
    fn texture_slot(&self, transparent: bool, float: bool) -> TextureSlot { default_texture_slot(transparent, float) }
    fn width(&self) -> u32 { self.image.width() }
    fn height(&self) -> u32 { self.image.height() }
    fn num_items(&self) -> u32 { self.image.num_items() }
    fn set_width(&mut self, _width: u32) { }
    fn set_height(&mut self, _height: u32) { }
    fn set_num_items(&mut self, _num_items: u32) { }
    fn data(&self) -> &Vec<u8> { self.image.data() }
    fn data_mut(&mut self) -> &mut Vec<u8> { &mut self.dummy_data }
}
