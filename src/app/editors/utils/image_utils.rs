use crate::image::{
    ImagePixels,
    ImageSlicingMethod,
};
use crate::data_asset;

#[derive(Clone, Copy, PartialEq)]
pub enum AddImageLocation {
    Start,
    BeforeSelected,
    AfterSelected,
    End,
}

impl AddImageLocation {
    pub fn text(self) -> &'static str {
        match self {
            Self::Start => { "the start" }
            Self::BeforeSelected => { "before selected" }
            Self::AfterSelected => { "after selected" }
            Self::End => { "the end" }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum SelectImageLocation {
    Start,
    Selected,
}

impl SelectImageLocation {
    pub fn text(self) -> &'static str {
        match self {
            Self::Start => { "the start" }
            Self::Selected => { "selected" }
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

pub struct CollisionRect {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

impl CollisionRect {
    pub fn from_data(rect: data_asset::Rect) -> Self {
        CollisionRect {
            x1: rect.x,
            y1: rect.y,
            x2: rect.x + rect.w,
            y2: rect.y + rect.h,
        }
    }

    pub fn apply_to_data(&self, rect: &mut data_asset::Rect) {
        rect.x = self.x1.clamp(-256, i16::MAX as i32);
        rect.y = self.y1.clamp(-256, i16::MAX as i32);
        rect.w = (self.x2 - self.x1).clamp(0, u16::MAX as i32);
        rect.h = (self.y2 - self.y1).clamp(0, u16::MAX as i32);
    }

    pub fn set_left_border(&mut self, val: i32) {
        self.x1 = val.clamp(-256, self.x2);
    }

    pub fn set_right_border(&mut self, val: i32) {
        self.x2 = val.clamp(self.x1, u16::MAX as i32);
    }

    pub fn set_top_border(&mut self, val: i32) {
        self.y1 = val.clamp(-256, self.y2);
    }

    pub fn set_bottom_border(&mut self, val: i32) {
        self.y2 = val.clamp(self.y1, u16::MAX as i32);
    }
}
