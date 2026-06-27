pub struct ImageConverter {
    map_6bit_to_8bit: Vec<u8>,
    vga_bits_per_pixel: u8,
}

impl ImageConverter {
    pub fn new(vga_bits_per_pixel: u8) -> Self {
        ImageConverter {
            map_6bit_to_8bit: Self::gen_6bit_to_8bit_map(),
            vga_bits_per_pixel,
        }
    }

    fn gen_6bit_to_8bit_map() -> Vec<u8> {
        let mut map_6bit_to_8bit = vec![0u8; 64];
        for (pix_6bit, pix_8bit) in map_6bit_to_8bit.iter_mut().enumerate() {
            let r6 = pix_6bit & 0x03;
            let g6 = (pix_6bit & 0x0c) >> 2;
            let b6 = (pix_6bit & 0x30) >> 4;
            let r8 = (r6 << 1) | (r6 >> 1);
            let g8 = (g6 << 1) | (g6 >> 1);
            let b8 = b6;
            *pix_8bit = (r8 | (g8 << 3) | (b8 << 6)) as u8;
        }
        map_6bit_to_8bit
    }

    fn image_6bit_u32_to_pixels(data: &[u32], width: u32, height: u32, num_items: u32, pixel_mapping: &[u8]) -> Vec<u8> {
        const COLOR_BITS: u32 = 0b0011_1111;

        let stride = width.div_ceil(4) as usize;
        let mut pixels = Vec::<u8>::with_capacity((width * height * num_items) as usize);
        for y in 0 .. (height * num_items) as usize {
            for x in 0..stride {
                let quad = data[y*stride + x];
                if x < stride-1 || width.is_multiple_of(4) {
                    pixels.push(pixel_mapping[(quad         & COLOR_BITS) as usize]);
                    pixels.push(pixel_mapping[((quad >>  8) & COLOR_BITS) as usize]);
                    pixels.push(pixel_mapping[((quad >> 16) & COLOR_BITS) as usize]);
                    pixels.push(pixel_mapping[((quad >> 24) & COLOR_BITS) as usize]);
                } else {
                    match width % 4 {
                        1 => {
                            pixels.push(pixel_mapping[(quad         & COLOR_BITS) as usize]);
                        },
                        2 => {
                            pixels.push(pixel_mapping[(quad         & COLOR_BITS) as usize]);
                            pixels.push(pixel_mapping[((quad >>  8) & COLOR_BITS) as usize]);
                        },
                        3 => {
                            pixels.push(pixel_mapping[(quad         & COLOR_BITS) as usize]);
                            pixels.push(pixel_mapping[((quad >>  8) & COLOR_BITS) as usize]);
                            pixels.push(pixel_mapping[((quad >> 16) & COLOR_BITS) as usize]);
                        },
                        _ => {},
                    }
                }
            }
        }
        pixels
    }

    fn image_8bit_u32_to_pixels(data: &[u32], width: u32, height: u32, num_items: u32) -> Vec<u8> {
        let stride = width.div_ceil(4) as usize;
        let mut pixels = Vec::<u8>::with_capacity((width * height * num_items) as usize);
        for y in 0 .. (height * num_items) as usize {
            for x in 0..stride {
                let quad = data[y*stride + x];
                if x < stride-1 || width.is_multiple_of(4) {
                    pixels.push((quad      ) as u8);
                    pixels.push((quad >>  8) as u8);
                    pixels.push((quad >> 16) as u8);
                    pixels.push((quad >> 24) as u8);
                } else {
                    match width % 4 {
                        1 => {
                            pixels.push((quad      ) as u8);
                        },
                        2 => {
                            pixels.push((quad      ) as u8);
                            pixels.push((quad >>  8) as u8);
                        },
                        3 => {
                            pixels.push((quad      ) as u8);
                            pixels.push((quad >>  8) as u8);
                            pixels.push((quad >> 16) as u8);
                        },
                        _ => {},
                    }
                }
            }
        }
        pixels
    }

    pub fn pal_image_to_pixels(data: &[u8], palette: &[u8], width: u32, height: u32, num_items: u32, bits_per_pixel: u32) -> Vec<u8> {
        let width = width as usize;
        let height = height as usize;
        let num_items = num_items as usize;
        let bits_per_pixel = bits_per_pixel as usize;
        let pixels_per_byte = 8 / bits_per_pixel;
        let stride = (width * bits_per_pixel).div_ceil(8);
        let palette_index_mask = (1u8 << bits_per_pixel) - 1;

        let mut pixels = vec![0u8; width * height * num_items];
        for y in 0 .. height * num_items {
            let mut block = 0u8;
            for x in 0 .. width {
                if x % pixels_per_byte == 0 {
                    block = data[y * stride + x/pixels_per_byte];
                }
                pixels[y * width + x] = palette[(block & palette_index_mask) as usize];
                block >>= bits_per_pixel;
            }
        }

        pixels
    }

    pub fn get_image_pixels(&self, data: &[u32], width: u32, height: u32, num_items: u32) -> Vec<u8> {
        if self.vga_bits_per_pixel == 8 {
            Self::image_8bit_u32_to_pixels(data, width, height, num_items)
        } else {
            Self::image_6bit_u32_to_pixels(data, width, height, num_items, &self.map_6bit_to_8bit)
        }
    }
}
