use super::{
    colors,
    ImagePixels,
    ImagePixelsCollection,
};
use crate::data_asset::{
    Tileset,
    Sprite,
    PalSprite,
    Font,
    PropFont,
};

pub struct ImageLoadOptions {
    pub slicing_method: ImageSlicingMethod,
    pub space_between: u32,
    pub border: u32,
    pub zoom_x: u32,
    pub zoom_y: u32,
}

#[derive(Clone, Copy)]
pub enum ImageSlicingMethod {
    BySize { width: u32, height: u32 },
    ByNumber { nx: u32, ny: u32 },
}

impl ImageSlicingMethod {
    pub const fn by_size(width: u32, height: u32) -> Self {
        ImageSlicingMethod::BySize { width, height }
    }

    pub const fn by_number(nx: u32, ny: u32) -> Self {
        ImageSlicingMethod::ByNumber { nx, ny }
    }
}

struct SrcImageReader {
    image: ::image::RgbaImage,
    image_width: u32,
    image_height: u32,
    zoom_x: u32,
    zoom_y: u32,
    pixel_freq: [usize; 256],
}

impl SrcImageReader {
    fn new(zoom_x: u32, zoom_y: u32, image: ::image::RgbaImage) -> Self {
        let image_width = image.width();
        let image_height = image.height();
        SrcImageReader {
            image,
            image_width,
            image_height,
            zoom_x: zoom_x.max(1),
            zoom_y: zoom_y.max(1),
            pixel_freq: [0; 256],
        }
    }

    fn width(&self) -> u32 {
        self.image_width.div_ceil(self.zoom_x)
    }

    fn height(&self) -> u32 {
        self.image_height.div_ceil(self.zoom_y)
    }

    fn get_pixel(&mut self, x: u32, y: u32) -> u8 {
        if self.zoom_x == 1 && self.zoom_y == 1 {
            self.get_raw_pixel(x, y)
        } else {
            self.pixel_freq[..].fill(0);
            for iy in y*self.zoom_y .. y*self.zoom_y+self.zoom_y {
                for ix in x*self.zoom_x .. x*self.zoom_x+self.zoom_x {
                    if ix < self.image_width && iy < self.image_height {
                        let pixel = self.get_raw_pixel(ix, iy);
                        self.pixel_freq[pixel as usize] += 1;
                    }
                }
            }
            self.pixel_freq.iter().enumerate().max_by_key(|&(_, freq)| freq).map(|(pixel, _)| pixel).unwrap_or(0) as u8
        }
    }

    fn get_raw_pixel(&self, x: u32, y: u32) -> u8 {
        let offset = ((self.image.width() * y + x) * 4) as usize;
        let data = self.image.as_raw();
        ImagePixels::rgba_to_pixel(&data[offset..offset+4])
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

    fn load_image_png(&mut self, data: &[u8], options: &ImageLoadOptions) -> Result<(), std::io::Error> {
        let image = ::image::ImageReader::new(std::io::Cursor::new(data))
            .with_guessed_format().map_err(|e| std::io::Error::other(e.to_string()))?
            .decode().map_err(|e| std::io::Error::other(e.to_string()))?
            .to_rgba8();
        let mut src = SrcImageReader::new(options.zoom_x, options.zoom_y, image);

        let (nx, ny, width, height) = match options.slicing_method {
            ImageSlicingMethod::BySize { width, height } => {
                let nx = (src.width() - 2*options.border + options.space_between) / (width + options.space_between);
                let ny = (src.height() - 2*options.border + options.space_between) / (height + options.space_between);
                (nx, ny, width, height)
            }
            ImageSlicingMethod::ByNumber { nx, ny } => {
                let width = if nx <= 1 {
                    src.width()  - 2*options.border
                } else {
                    (src.width()  - 2*options.border - (nx - 1) * options.space_between) / nx
                };
                let height = if ny <= 1 {
                    src.height() - 2*options.border
                } else {
                    (src.height() - 2*options.border - (ny - 1) * options.space_between) / ny
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
                    let src_y = options.border + iy * (height + options.space_between) + y;
                    if src_y >= src.height() { continue; }
                    for x in 0..width {
                        let src_x = options.border + ix * (width + options.space_between) + x;
                        if src_x >= src.width() { continue; }
                        //let src_off = (src_y * src.width() + src_x) as usize * 4;
                        //dst_data[(dst_off + y*width + x) as usize] = ImagePixels::rgba_to_pixel(&src_data[src_off..src_off+4]);
                        dst_data[(dst_off + y*width + x) as usize] = src.get_pixel(src_x, src_y);
                    }
                }
            }
        }
        self.set_width(width);
        self.set_height(height);
        self.set_num_items(nx * ny);
        Ok(())
    }

    fn save_image_png(&self, num_items_x: u32) -> Result<Vec<u8>, std::io::Error> {
        self.save_png(num_items_x, ImagePixels::pixel_to_rgba)
    }

    fn save_font_png(&self, num_items_x: u32) -> Result<Vec<u8>, std::io::Error> {
        fn conv_pixel(pixel: u8) -> [u8; 4] {
            if pixel == Font::BG_COLOR {
                [0, 0xff, 0, 0xff]
            } else {
                [0, 0, 0, 0xff]
            }
        }
        self.save_png(num_items_x, conv_pixel)
    }

    fn save_png<F: Fn(u8) -> [u8; 4]>(
        &self,
        num_items_x: u32, conv_pixel: F
    ) -> Result<Vec<u8>, std::io::Error> {
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

        let mut out = std::io::Cursor::new(Vec::new());
        ::image::write_buffer_with_format(&mut out, &dst, dst_w, dst_h, ::image::ExtendedColorType::Rgba8, ::image::ImageFormat::Png)
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        Ok(out.into_inner())
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
