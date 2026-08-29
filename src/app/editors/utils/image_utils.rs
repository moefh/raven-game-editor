use crate::image::{
    ImagePixels,
    ImageSlicingMethod,
};

#[derive(Clone, Copy, PartialEq)]
pub enum AddImageLocation {
    BeforeSelected,
    AfterSelected,
    AtEnd,
}

impl AddImageLocation {
    pub fn text(self) -> &'static str {
        match self {
            Self::BeforeSelected => { "before selected" }
            Self::AfterSelected => { "after selected" }
            Self::AtEnd => { "at end" }
        }
    }
}

pub enum ImageClipboardData {
    Empty,
    Image(ImagePixels),
}

impl ImageClipboardData {
    pub fn is_some(&self) -> bool {
        matches!(self, ImageClipboardData::Image(_))
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
