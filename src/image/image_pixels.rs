use super::colors;

use crate::data_asset::PropFont;

#[derive(Clone, Debug)]
pub struct ImagePixels {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl ImagePixels {
    pub fn new(width: u32, height: u32, data: Vec<u8>) -> Self {
        ImagePixels {
            width,
            height,
            data,
        }
    }

    pub fn force_palette(&mut self, palette: &[u8], color_to_palette_index_map: &[u8]) -> bool {
        let mut changed = false;
        for pixel in self.data.iter_mut() {
            let pal_index = color_to_palette_index_map[*pixel as usize] as usize;
            let new_pixel = palette[pal_index % palette.len()];
            if *pixel != new_pixel {
                *pixel = new_pixel;
                changed = true;
            }
        }
        changed
    }

    pub fn pixel_to_rgba(pixel: u8) -> [u8; 4] {
        let r = (pixel     ) & 0b111;
        let g = (pixel >> 3) & 0b111;
        let b = (pixel >> 6) & 0b11;
        [
            (r << 5) | (r << 2) | (r >> 1),
            (g << 5) | (g << 2) | (g >> 1),
            (b << 6) | (b << 4) | (b << 2) | b,
            if r == 0 && g == 0b111 && b == 0 { 0 } else { 255 },
        ]
    }

    pub fn rgba_to_pixel(data: &[u8]) -> u8 {
        if data[3] >= 0x80 {
            (data[2] & 0b11_000_000) |
            ((data[1] >> 2) & 0b00_111_000) |
            ((data[0] >> 5) & 0b00_000_111)
        } else {
            colors::TRANSPARENT
        }
    }

    pub fn rgb_to_pixel(data: &[u8]) -> u8 {
        (data[2] & 0b11_000_000) |
        ((data[1] >> 2) & 0b00_111_000) |
        ((data[0] >> 5) & 0b00_000_111)
    }

    pub fn rgb_image_to_pixels(image: ::image::RgbImage) -> Vec<u8> {
        let mut data = Vec::with_capacity((image.width() * image.height()) as usize);
        for pixel in image.as_raw().chunks_exact(3) {
            data.push(Self::rgb_to_pixel(pixel));
        }
        data
    }

    pub fn rgba_image_to_pixels(image: ::image::RgbaImage) -> Vec<u8> {
        let mut data = Vec::with_capacity((image.width() * image.height()) as usize);
        for pixel in image.as_raw().chunks_exact(4) {
            data.push(Self::rgba_to_pixel(pixel));
        }
        data
    }

    pub fn load_png(path: impl AsRef<std::path::Path>) -> Result<ImagePixels, Box<dyn std::error::Error>> {
        let img = ::image::ImageReader::open(path)?.decode()?.to_rgba8();
        Ok(ImagePixels {
            width: img.width(),
            height: img.height(),
            data: Self::rgba_image_to_pixels(img),
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

#[derive(Clone)]
pub struct ImagePixelsCollection {
    pub width: u32,
    pub height: u32,
    pub num_items: u32,
    pub data: Vec<u8>,
}

impl ImagePixelsCollection {
    pub fn new(width: u32, height: u32, num_items: u32) -> Self {
        ImagePixelsCollection {
            width,
            height,
            num_items,
            data: vec![0u8; (width*height*num_items) as usize],
        }
    }
}
