use super::super::widgets::ImageEditorZoom;

pub const IMAGE_ZOOM_OPTIONS: &[ImageZoomOption] = &[
    ImageZoomOption::FitToWindow("Fit"),
    ImageZoomOption::Value(1.0, "1x"),
    ImageZoomOption::Value(2.0, "2x"),
    ImageZoomOption::Value(5.0, "5x"),
    ImageZoomOption::Value(10.0, "10x"),
    ImageZoomOption::Value(20.0, "20x"),
    ImageZoomOption::Custom("Custom"),
];

#[derive(PartialEq)]
pub enum ImageZoomOption<'a> {
    FitToWindow(&'a str),
    Value(f32, &'a str),
    Custom(&'a str),
}

impl<'a> ImageZoomOption<'a> {
    pub fn name(&self) -> &'a str {
        match self {
            ImageZoomOption::FitToWindow(name) => { name }
            ImageZoomOption::Custom(name) => { name }
            ImageZoomOption::Value(_, name) => { name }
        }
    }

    pub fn is_custom(&self) -> bool {
        matches!(&self, ImageZoomOption::Custom(_))
    }

    pub fn image_editor_zoom(&self, cur_editor_zoom: ImageEditorZoom) -> ImageEditorZoom {
        match self {
            ImageZoomOption::FitToWindow(_) => { ImageEditorZoom::FitToWindow }
            ImageZoomOption::Custom(_) => { cur_editor_zoom }
            ImageZoomOption::Value(zoom, _) => { ImageEditorZoom::Custom(*zoom) }
        }
    }

    pub fn from_image_editor_zoom(image_zoom: ImageEditorZoom) -> &'a Self {
        match image_zoom {
            ImageEditorZoom::FitToWindow => { &IMAGE_ZOOM_OPTIONS[0] }
            ImageEditorZoom::Custom(1.0) => { &IMAGE_ZOOM_OPTIONS[1] }
            ImageEditorZoom::Custom(2.0) => { &IMAGE_ZOOM_OPTIONS[2] }
            ImageEditorZoom::Custom(5.0) => { &IMAGE_ZOOM_OPTIONS[3] }
            ImageEditorZoom::Custom(10.0) => { &IMAGE_ZOOM_OPTIONS[4] }
            ImageEditorZoom::Custom(20.0) => { &IMAGE_ZOOM_OPTIONS[5] }
            ImageEditorZoom::Custom(_) => { &IMAGE_ZOOM_OPTIONS[6] }
        }
    }
}
