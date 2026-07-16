use super::ImageCollection;

pub enum ColorSetSource {
    SingleImage(u32),
    AllImagesPrioritizing(u32),
    AllImages,
}

pub struct ColorSet {
    pub name: String,
    pub colors: Vec<u8>,
}

impl ColorSet {
    pub const NUM_COLORS: usize = 28;

    pub fn new(name: String, colors: Vec<u8>) -> Self {
        ColorSet {
            name,
            colors,
        }
    }

    // count how many times each color appears in the image
    fn get_color_histogram(image_data: &[u8]) -> Vec<u32> {
        let mut histogram = vec![0u32; 256];
        for color in image_data.iter() {
            let color = *color as usize;
            histogram[color] = histogram[color].saturating_add(1);
        }
        histogram
    }

    pub fn from_image(name: String, image: &impl ImageCollection, source: &ColorSetSource) -> Self {
        let full_image_data = match source {
            ColorSetSource::AllImages |
            ColorSetSource::AllImagesPrioritizing(..) => { &image.data()[..] }
            ColorSetSource::SingleImage(..) => { &[] }
        };
        let single_image_data = match source {
            ColorSetSource::AllImages => { &[] }
            ColorSetSource::SingleImage(index) |
            ColorSetSource::AllImagesPrioritizing(index) => {
                let index = *index as usize;
                let image_size = (image.width() * image.height()) as usize;
                &image.data()[index * image_size .. (index + 1) * image_size]
            }
        };

        // count how many times each color appears in the image
        let single_image_histogram = Self::get_color_histogram(single_image_data);
        let full_image_histogram = Self::get_color_histogram(full_image_data);

        // sort the most used colors first, eliminating unused colors
        let mut single_image_colors = single_image_histogram.into_iter().enumerate().filter(|&(_, n)| n != 0).collect::<Vec<(usize, u32)>>();
        single_image_colors.sort_by_key(|&(_, n)| -(n as i64));
        let mut full_image_colors = full_image_histogram.into_iter().enumerate().filter(|&(_, n)| n != 0).collect::<Vec<(usize, u32)>>();
        full_image_colors.sort_by_key(|&(_, n)| -(n as i64));

        // pick however many colors we need
        let mut colors = single_image_colors
            .into_iter()
            .filter(|&(_, n)| n != 0)
            .take(Self::NUM_COLORS)
            .map(|(c, _)| (c & 0xff) as u8)
            .collect::<Vec<u8>>();
        if colors.len() < Self::NUM_COLORS {
            for (color, _) in full_image_colors {
                let color = (color & 0xff) as u8;
                if colors.len() >= Self::NUM_COLORS { break; }  // filled colorset
                if colors.contains(&color) { continue; }        // already got that color
                colors.push(color);
            }
        }
        colors.sort();
        ColorSet::new(name, colors)
    }
}

pub struct ColorSetCollection {
    stock: Vec<ColorSet>,
    custom: Vec<ColorSet>,
}

impl ColorSetCollection {
    const STOCK_SET_PRIMARY: &[u8; ColorSet::NUM_COLORS] = &[
        0b00_000_001, 0b00_001_000, 0b01_000_000, 0b00_001_001,
        0b00_000_010, 0b00_010_000, 0b10_000_000, 0b01_010_010,
        0b00_000_011, 0b00_011_000, 0b11_000_000, 0b01_011_011,
        0b00_000_100, 0b00_100_000, 0b00_111_111, 0b10_100_100,
        0b00_000_101, 0b00_101_000, 0b11_000_111, 0b10_101_101,
        0b00_000_110, 0b00_110_000, 0b11_111_000, 0b11_110_110,
        0b00_000_111, 0b00_111_000, 0b00_000_000, 0b11_111_111,
    ];

    const STOCK_SET_SECONDARY: &[u8; ColorSet::NUM_COLORS] = &[
        0b01_001_000, 0b00_001_001, 0b01_000_001, 0b00_000_000,
        0b01_010_000, 0b00_010_010, 0b01_000_010, 0b00_000_000,
        0b01_011_000, 0b00_011_011, 0b01_000_011, 0b00_000_000,
        0b10_100_000, 0b00_100_100, 0b01_000_100, 0b00_000_000,
        0b10_101_000, 0b00_101_101, 0b10_000_101, 0b00_000_000,
        0b10_110_000, 0b00_110_110, 0b10_000_110, 0b00_000_000,
        0b11_111_000, 0b00_111_111, 0b11_000_111, 0b11_111_111,
    ];

    pub fn new() -> Self {
        ColorSetCollection {
            stock: Self::build_stock_colorsets(),
            custom: Vec::new(),
        }
    }

    pub fn get_custom_colorsets(&self) -> impl Iterator<Item = &ColorSet> {
        self.custom.iter()
    }

    pub fn get_colorset_names(&self) -> impl Iterator<Item = &str> {
        self.get_stock_colorset_names().chain(self.get_custom_colorset_names())
    }

    pub fn get_stock_colorset_names(&self) -> impl Iterator<Item = &str> {
        self.stock.iter().map(|set| set.name.as_str())
    }

    pub fn get_custom_colorset_names(&self) -> impl Iterator<Item = &str> {
        self.custom.iter().map(|set| set.name.as_str())
    }

    fn get_colorset(&self, index: usize) -> Option<&ColorSet> {
        let first_custom_colorset = self.stock.len();
        if index < first_custom_colorset {
            self.stock.get(index)
        } else if index - first_custom_colorset < self.custom.len() {
            self.custom.get(index - first_custom_colorset)
        } else {
            None
        }
    }

    pub fn get_colorset_name(&self, index: usize) -> Option<&str> {
        self.get_colorset(index).map(|set| set.name.as_str())
    }

    pub fn get_colorset_colors(&self, index: usize) -> Option<&[u8]> {
        self.get_colorset(index).map(|set| &set.colors[..])
    }

    pub fn is_colorset_custom(&self, index: usize) -> bool {
        let first_custom_colorset = self.stock.len();
        index >= first_custom_colorset && index - first_custom_colorset < self.custom.len()
    }

    pub fn get_num_custom_colorsets(&self) -> usize {
        self.custom.len()
    }

    pub fn get_custom_colorset_range(&self) -> core::range::Range<usize> {
        core::range::Range::from(self.stock.len()..self.stock.len()+self.custom.len())
    }

    pub fn get_custom_colorset(&self, index: usize) -> Option<&ColorSet> {
        let first_custom_colorset = self.stock.len();
        if index >= first_custom_colorset {
            self.custom.get(index - first_custom_colorset)
        } else {
            None
        }
    }

    pub fn get_custom_colorset_mut(&mut self, index: usize) -> Option<&mut ColorSet> {
        let first_custom_colorset = self.stock.len();
        if index >= first_custom_colorset {
            self.custom.get_mut(index - first_custom_colorset)
        } else {
            None
        }
    }

    pub fn add_custom_colorset(&mut self, set: ColorSet) -> usize {
        let new_colorset_index = self.custom.len();
        self.custom.push_mut(set).colors.resize(ColorSet::NUM_COLORS, 0);
        self.stock.len() + new_colorset_index
    }

    pub fn remove_custom_colorset(&mut self, index: usize) -> Option<ColorSet> {
        let first_custom_colorset = self.stock.len();
        if index >= first_custom_colorset && index - first_custom_colorset < self.custom.len() {
            Some(self.custom.remove(index - first_custom_colorset))
        } else {
            None
        }
    }

    pub fn clear_custom_colorsets(&mut self) {
        self.custom.clear();
    }

    fn build_stock_colorsets() -> Vec<ColorSet> {
        vec![
            ColorSet {
                name: String::from("Primary"),
                colors: Self::STOCK_SET_PRIMARY.to_vec(),
            },
            ColorSet {
                name: String::from("Secondary"),
                colors: Self::STOCK_SET_SECONDARY.to_vec(),
            }
        ]
    }
}
