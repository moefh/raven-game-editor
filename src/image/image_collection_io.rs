use super::{colors, ImagePixels, ImagePixelsCollection};
use crate::data_asset::{Tileset, Sprite, PalSprite, Font, PropFont};

pub enum ImageSlicingMethod {
    BySize { width: u32, height: u32 },
    ByNumber { nx: u32, ny: u32 },
}

impl ImageSlicingMethod {
    pub fn by_size(width: u32, height: u32) -> Self {
        ImageSlicingMethod::BySize { width, height }
    }

    pub fn by_number(nx: u32, ny: u32) -> Self {
        ImageSlicingMethod::ByNumber { nx, ny }
    }
}

pub trait ImageCollectionIO {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn num_items(&self) -> u32;
    fn set_width(&mut self, width: u32);
    fn set_height(&mut self, height: u32);
    fn set_num_items(&mut self, num_items: u32);
    fn data(&self) -> &Vec<u8>;
    fn data_mut(&mut self) -> &mut Vec<u8>;

    fn load_image_png(&mut self, path: impl AsRef<std::path::Path>, slicing: &ImageSlicingMethod, border: u32, space_between: u32)
                      -> Result<(), Box<dyn std::error::Error>> {
        let src = ::image::ImageReader::open(path)?.decode()?.to_rgba8();
        let src_data = src.as_raw();

        let (nx, ny, width, height) = match *slicing {
            ImageSlicingMethod::BySize { width, height } => {
                let nx = (src.width() - 2*border + space_between) / (width + space_between);
                let ny = (src.height() - 2*border + space_between) / (height + space_between);
                (nx, ny, width, height)
            }
            ImageSlicingMethod::ByNumber { nx, ny } => {
                let width = if nx <= 1 {
                    src.width()  - 2*border
                } else {
                    (src.width()  - 2*border - (nx - 1) * space_between) / nx
                };
                let height = if ny <= 1 {
                    src.height() - 2*border
                } else {
                    (src.height() - 2*border - (ny - 1) * space_between) / ny
                };
                (nx, ny, width, height)
            }
        };
        let nx = nx.max(1);
        let ny = ny.max(1);
        let width = width.max(1);
        let height = height.max(1);

        let dst_data = self.data_mut();
        if dst_data.len() != (nx * ny * width * height) as usize {
            dst_data.resize((nx * ny * width * height) as usize, colors::TRANSPARENT);
        }
        dst_data.fill(colors::TRANSPARENT);
        for iy in 0..ny {
            for ix in 0..nx {
                let dst_off = ((iy * nx) + ix) * width * height;
                for y in 0..height {
                    let src_y = border + iy * (height + space_between) + y;
                    if src_y >= src.height() { continue; }
                    for x in 0..width {
                        let src_x = border + ix * (width + space_between) + x;
                        if src_x >= src.width() { continue; }
                        let src_off = (src_y * src.width() + src_x) as usize * 4;
                        dst_data[(dst_off + y*width + x) as usize] = ImagePixels::rgba_to_pixel(&src_data[src_off..src_off+4]);
                    }
                }
            }
        }
        self.set_width(width);
        self.set_height(height);
        self.set_num_items(nx * ny);
        Ok(())
    }

    fn save_image_png(&self, path: impl AsRef<std::path::Path>, num_items_x: u32) -> Result<(), Box<dyn std::error::Error>> {
        self.save_png(path, num_items_x, ImagePixels::pixel_to_rgba)
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

impl ImageCollectionIO for Sprite {
    fn width(&self) -> u32 { self.width }
    fn height(&self) -> u32 { self.height }
    fn num_items(&self) -> u32 { self.num_frames }
    fn set_width(&mut self, width: u32) { self.width = width; }
    fn set_height(&mut self, height: u32) { self.height = height; }
    fn set_num_items(&mut self, num_items: u32) { self.num_frames = num_items; }
    fn data(&self) -> &Vec<u8> { &self.data }
    fn data_mut(&mut self) -> &mut Vec<u8> { &mut self.data }
}

impl ImageCollectionIO for PalSprite {
    fn width(&self) -> u32 { self.width }
    fn height(&self) -> u32 { self.height }
    fn num_items(&self) -> u32 { self.num_frames }
    fn set_width(&mut self, width: u32) { self.width = width; }
    fn set_height(&mut self, height: u32) { self.height = height; }
    fn set_num_items(&mut self, num_items: u32) { self.num_frames = num_items; }
    fn data(&self) -> &Vec<u8> { &self.data }
    fn data_mut(&mut self) -> &mut Vec<u8> { &mut self.data }
}

impl ImageCollectionIO for Tileset {
    fn width(&self) -> u32 { self.width }
    fn height(&self) -> u32 { self.height }
    fn num_items(&self) -> u32 { self.num_tiles }
    fn set_width(&mut self, width: u32) { self.width = width; }
    fn set_height(&mut self, height: u32) { self.height = height; }
    fn set_num_items(&mut self, num_items: u32) { self.num_tiles = num_items; }
    fn data(&self) -> &Vec<u8> { &self.data }
    fn data_mut(&mut self) -> &mut Vec<u8> { &mut self.data }
}

impl ImageCollectionIO for Font {
    fn width(&self) -> u32 { self.width }
    fn height(&self) -> u32 { self.height }
    fn num_items(&self) -> u32 { Font::NUM_CHARS }
    fn set_width(&mut self, width: u32) { self.width = width; }
    fn set_height(&mut self, height: u32) { self.height = height; }
    fn set_num_items(&mut self, _num_items: u32) { }
    fn data(&self) -> &Vec<u8> { &self.data }
    fn data_mut(&mut self) -> &mut Vec<u8> { &mut self.data }
}

impl ImageCollectionIO for PropFont {
    fn width(&self) -> u32 { self.max_width }
    fn height(&self) -> u32 { self.height }
    fn num_items(&self) -> u32 { PropFont::NUM_CHARS }
    fn set_width(&mut self, width: u32) { self.max_width = width; }
    fn set_height(&mut self, height: u32) { self.height = height; }
    fn set_num_items(&mut self, _num_items: u32) { }
    fn data(&self) -> &Vec<u8> { &self.data }
    fn data_mut(&mut self) -> &mut Vec<u8> { &mut self.data }
}

impl ImageCollectionIO for ImagePixels {
    fn width(&self) -> u32 { self.width }
    fn height(&self) -> u32 { self.height }
    fn num_items(&self) -> u32 { 1 }
    fn set_width(&mut self, width: u32) { self.width = width; }
    fn set_height(&mut self, height: u32) { self.height = height; }
    fn set_num_items(&mut self, _num_items: u32) { }
    fn data(&self) -> &Vec<u8> { &self.data }
    fn data_mut(&mut self) -> &mut Vec<u8> { &mut self.data }
}

impl ImageCollectionIO for ImagePixelsCollection {
    fn width(&self) -> u32 { self.width }
    fn height(&self) -> u32 { self.height }
    fn num_items(&self) -> u32 { self.num_items }
    fn set_width(&mut self, width: u32) { self.width = width; }
    fn set_height(&mut self, height: u32) { self.height = height; }
    fn set_num_items(&mut self, num_items: u32) { self.num_items = num_items; }
    fn data(&self) -> &Vec<u8> { &self.data }
    fn data_mut(&mut self) -> &mut Vec<u8> { &mut self.data }
}
