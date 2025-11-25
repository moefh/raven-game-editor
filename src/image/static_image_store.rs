use std::collections::HashMap;

use super::{StaticImageId, StaticImageData};

pub struct StaticImageStore {
    next_id: u32,
    images: HashMap<StaticImageId, StaticImageData>,
}

impl StaticImageStore {
    pub fn new() -> Self {
        StaticImageStore {
            next_id: 0,
            images: HashMap::new(),
        }
    }

    fn image_to_pixels(image: ::image::RgbImage) -> Vec<u8> {
        let mut data = Vec::new();

        for pixel in image.pixels() {
            data.push((pixel[0] >> 2) & 0b110000 |
                      (pixel[1] >> 4) & 0b001100 |
                      (pixel[2] >> 6) & 0b000011);
        }

        data
    }

    fn gen_id(&mut self) -> StaticImageId {
        let id = self.next_id;
        self.next_id += 1;
        StaticImageId(id)
    }

    pub fn load_image(&mut self, name: &str, width: u32, height: u32, data: &[u8]) -> StaticImageId {
        let image = match ::image::load_from_memory(data) {
            Ok(image) => image,
            Err(e) => panic!("ERROR: failed to load {}: {}", name, e),
        };

        let id = self.gen_id();
        let num_items = image.height() / height;
        let data = Self::image_to_pixels(image.to_rgb8());
        let image = StaticImageData::new(id, width, height, num_items, data);
        self.images.insert(id, image);
        id
    }

    pub fn get(&self, id: StaticImageId) -> Option<&StaticImageData> {
        self.images.get(&id)
    }
}
