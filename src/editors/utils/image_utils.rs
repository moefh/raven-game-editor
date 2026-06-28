use crate::image::{
    ImagePixels,
    ImageSlicingMethod,
};

pub enum ImageClipboardData {
    Empty,
    Image(ImagePixels),
}

impl ImageClipboardData {
    pub fn is_none(&self) -> bool {
        matches!(self, ImageClipboardData::Empty)
    }

    pub fn take(&mut self) -> ImageClipboardData {
        std::mem::replace(self, ImageClipboardData::Empty)
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ImageSlicingMethodOption {
    BySize,
    ByNumber,
}

impl ImageSlicingMethodOption {
    pub fn from_slicing_method(method: &ImageSlicingMethod) -> Self {
        match method {
            ImageSlicingMethod::BySize{..} => ImageSlicingMethodOption::BySize,
            ImageSlicingMethod::ByNumber{..} => ImageSlicingMethodOption::ByNumber,
        }
    }
    pub fn text(&self) -> &str {
        match self {
            ImageSlicingMethodOption::BySize => "by size",
            ImageSlicingMethodOption::ByNumber => "by quantity",
        }
    }
}
