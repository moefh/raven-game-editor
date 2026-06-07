pub struct ColorSet {
    pub name: String,
    pub colors: Vec<u8>,
}

impl ColorSet {
    pub fn new(name: String, colors: Vec<u8>) -> Self {
        ColorSet {
            name,
            colors,
        }
    }
}

pub struct ColorSetCollection {
    stock: Vec<ColorSet>,
    custom: Vec<ColorSet>,
}

impl ColorSetCollection {
    const STOCK_SET_PRIMARY: &[u8] = &[
        0b00_000_001, 0b00_001_000, 0b01_000_000, 0b00_001_001,
        0b00_000_010, 0b00_010_000, 0b10_000_000, 0b01_010_010,
        0b00_000_011, 0b00_011_000, 0b11_000_000, 0b01_011_011,
        0b00_000_100, 0b00_100_000, 0b00_111_111, 0b10_100_100,
        0b00_000_101, 0b00_101_000, 0b11_000_111, 0b10_101_101,
        0b00_000_110, 0b00_110_000, 0b11_111_000, 0b11_110_110,
        0b00_000_111, 0b00_111_000, 0b00_000_000, 0b11_111_111,
    ];

    const STOCK_SET_SECONDARY: &[u8] = &[
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

    pub fn add_custom_colorset(&mut self, set: ColorSet) {
        self.custom.push_mut(set).colors.resize(Self::STOCK_SET_PRIMARY.len(), 0);
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
