use crate::image::{TextureSlot, ImageCollection, ImageFragment, ImagePixels, ImageRect};
use crate::data_asset::ImageCollectionAsset;
use crate::app::{WindowContext, KeyboardPressed};

use super::super::ClipboardData;
use egui::{Vec2, Sense, Image, Rect, Pos2, emath};

pub enum ImageSelection {
    None,
    Rect(Pos2, Pos2),
    Fragment(Pos2, ImageFragment),
}

impl ImageSelection {
    pub fn is_floating(&self) -> bool {
        matches!(self, ImageSelection::Fragment(..))
    }

    pub fn set_changed(&mut self) {
        if let ImageSelection::Fragment(_, frag) = self {
            frag.changed = true;
        }
    }

    pub fn take_fragment(&mut self) -> Option<(Pos2, ImageFragment)> {
        match self {
            ImageSelection::Fragment(..) => {
                let mut ret = ImageSelection::None;
                std::mem::swap(self, &mut ret);
                match ret {
                    ImageSelection::Fragment(pos, frag) => Some((pos, frag)),
                    _ => None,  // this shouldn't happen
                }
            }
            _ => None
        }
    }

    pub fn get_rect(&self) -> Option<Rect> {
        match self {
            ImageSelection::Rect(origin, end) => {
                Some(Rect {
                    min: Pos2::new(origin.x.min(end.x), origin.y.min(end.y)),
                    max: Pos2::new(origin.x.max(end.x), origin.y.max(end.y)),
                })
            }
            ImageSelection::Fragment(pos, frag) => {
                Some(Rect {
                    min: *pos,
                    max: *pos + Vec2::new(frag.width() as f32, frag.height() as f32),
                })
            }
            ImageSelection::None => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum ImageDrawingTool {
    Pencil,
    Fill,
    Select,
}

#[derive(Clone, Copy)]
pub struct ImageDisplay {
    pub bits: u8,
}

impl ImageDisplay {
    pub const GRID: u8        = 1 << 0;
    pub const TRANSPARENT: u8 = 1 << 1;

    pub fn new(bits: u8) -> Self {
        ImageDisplay {
            bits,
        }
    }

    pub fn toggle(&mut self, bits: u8) {
        self.bits ^= bits;
    }

    pub fn has_bits(&self, bits: u8) -> bool {
        (self.bits & bits) != 0
    }

    pub fn is_transparent(&self) -> bool {
        self.has_bits(Self::TRANSPARENT)
    }

    pub fn texture_slot(&self) -> TextureSlot {
        if self.is_transparent() {
            TextureSlot::Transparent
        } else {
            TextureSlot::Opaque
        }
    }

    pub fn float_texture_slot(&self) -> TextureSlot {
        if self.is_transparent() {
            TextureSlot::FloatTransparent
        } else {
            TextureSlot::FloatOpaque
        }
    }
}

pub struct ImageEditorWidget {
    pub display: ImageDisplay,
    pub selection: ImageSelection,
    tool: ImageDrawingTool,
    selected_image: u32,
    undo_target: Option<ImageFragment>,
    image_changed: bool,
    tool_changed: bool,
    drag_mouse_origin: Pos2,
    drag_frag_origin: Pos2,
}

impl ImageEditorWidget {
    pub fn new() -> Self {
        ImageEditorWidget {
            selected_image: 0,
            display: ImageDisplay::new(ImageDisplay::TRANSPARENT | ImageDisplay::GRID),
            tool: ImageDrawingTool::Pencil,
            selection: ImageSelection::None,
            drag_mouse_origin: Pos2::ZERO,
            drag_frag_origin: Pos2::ZERO,
            undo_target: None,
            image_changed: false,
            tool_changed: false,
        }
    }

    pub fn with_selected_image(mut self, selected_image: u32) -> Self {
        self.selected_image = selected_image;
        self
    }

    pub fn toggle_display(&mut self, bits: u8) {
        self.display.toggle(bits);
        if (bits & ImageDisplay::TRANSPARENT) != 0 {
            self.image_changed = true;
            self.selection.set_changed();
        }
    }

    pub fn set_undo_target(&mut self, asset: &impl ImageCollectionAsset) {
        let image = ImageCollection::from_asset(asset);
        self.undo_target = image.copy_fragment(asset.asset_id(), asset.data(), self.selected_image, ImageRect::from_image_item(&image));
    }

    fn lift_selection(&mut self, asset: &mut impl ImageCollectionAsset, bg_color: u8) {
        if self.selection.is_floating() { return; } // already lifted

        if let Some(sel_rect) = self.selection.get_rect() && sel_rect.is_positive() {
            self.set_undo_target(asset);
            let image = ImageCollection::from_asset(asset);
            let image_rect = ImageRect::from_rect(sel_rect, &image);
            let bg_color = if self.display.is_transparent() { ImageFragment::TRANSPARENT_COLOR } else { bg_color };
            if let Some(frag) = image.cut_fragment(asset.asset_id(), asset.data_mut(), self.selected_image, image_rect, bg_color) {
                self.selection = ImageSelection::Fragment(sel_rect.min, frag);
                self.image_changed = true;
            } else {
                self.selection = ImageSelection::None;
            }
        }
    }

    pub fn undo(&mut self, asset: &mut impl ImageCollectionAsset) {
        if let Some(frag) = self.undo_target.take() {
            let image = ImageCollection::from_asset(asset);
            image.paste_fragment(asset.data_mut(), self.selected_image, 0, 0, &frag, false);
            self.image_changed = true;
            self.selection = ImageSelection::None;
        }
    }

    pub fn delete_selection(&mut self, asset: &mut impl ImageCollectionAsset, fill_color: u8) {
        self.lift_selection(asset, fill_color);
        self.selection = ImageSelection::None;
    }

    pub fn drop_selection(&mut self, asset: &mut impl ImageCollectionAsset) {
        let image = ImageCollection::from_asset(asset);
        if let ImageSelection::Fragment(pos, frag) = &self.selection {
            let transparent = self.display.is_transparent();
            image.paste_fragment(asset.data_mut(), self.selected_image, pos.x as i32, pos.y as i32, frag, transparent);
            self.image_changed = true;
        }
        self.selection = ImageSelection::None;
    }

    pub fn vflip(&mut self, asset: &mut impl ImageCollectionAsset, bg_color: u8) {
        if matches!(self.selection, ImageSelection::Rect(..)) {
            self.lift_selection(asset, bg_color);
        }

        if let ImageSelection::Fragment(_, frag) = &mut self.selection {
            let image = ImageCollection::from_asset(frag);
            image.v_flip(frag.data_mut(), 0);
            frag.changed = true;
        } else {
            let image = ImageCollection::from_asset(asset);
            image.v_flip(asset.data_mut(), self.selected_image);
            self.image_changed = true;
        }
    }

    pub fn hflip(&mut self, asset: &mut impl ImageCollectionAsset, bg_color: u8) {
        if matches!(self.selection, ImageSelection::Rect(..)) {
            self.lift_selection(asset, bg_color);
        }

        if let ImageSelection::Fragment(_, frag) = &mut self.selection {
            let image = ImageCollection::from_asset(frag);
            image.h_flip(frag.data_mut(), 0);
            frag.changed = true;
        } else {
            let image = ImageCollection::from_asset(asset);
            image.h_flip(asset.data_mut(), self.selected_image);
            self.image_changed = true;
        }
    }

    pub fn get_tool(&self) -> ImageDrawingTool {
        self.tool
    }

    pub fn set_tool(&mut self, tool: ImageDrawingTool) {
        self.tool = tool;
        self.tool_changed = true;
    }

    pub fn get_selected_image(&self) -> u32 {
        self.selected_image
    }

    pub fn set_selected_image(&mut self, sel_image: u32, asset: &impl ImageCollectionAsset) {
        if self.selected_image != sel_image {
            self.selected_image = sel_image;
            self.set_undo_target(asset);
        }
    }

    fn get_selected_color_for_click(&self, resp: &egui::Response, colors: (u8, u8)) -> Option<u8> {
        if resp.dragged_by(egui::PointerButton::Primary) {
            Some(colors.0)
        } else if resp.dragged_by(egui::PointerButton::Secondary) {
            Some(colors.1)
        } else {
            None
        }
    }

    fn handle_selection_mouse(&mut self, image: &ImageCollection, mouse_pos: Pos2, asset: &mut impl ImageCollectionAsset,
                              resp: &egui::Response, colors: (u8, u8)) {
        if ! resp.dragged_by(egui::PointerButton::Primary) {
            self.drag_mouse_origin = mouse_pos;
            self.drag_frag_origin = mouse_pos;
            return;
        }

        let orig_mouse_pos = mouse_pos;
        let mouse_pos = Rect::from_min_size(Pos2::ZERO, image.get_item_size()).clamp(mouse_pos);
        if resp.drag_started() {
            self.drag_mouse_origin = mouse_pos;
            match self.selection {
                ImageSelection::Rect(..) => {
                    if let Some(sel_rect) = self.selection.get_rect() && sel_rect.contains(orig_mouse_pos) {
                        // lift fragment for the selected rectangle
                        self.lift_selection(asset, colors.1);

                        // prepare to move it
                        self.drag_frag_origin = sel_rect.min;
                    } else {
                        self.selection = ImageSelection::None;
                    }
                }
                ImageSelection::Fragment(..) => {
                    if let Some(sel_rect) = self.selection.get_rect() {
                        if sel_rect.contains(orig_mouse_pos) {
                            // prepare to move fragment
                            self.drag_frag_origin = sel_rect.min;
                        } else {
                            // drop fragment because the click was outside it
                            self.drop_selection(asset);
                        }
                    }
                }
                _ => {}
            }
        } else if ! resp.drag_stopped() {
            self.selection = if let Some((_, frag)) = self.selection.take_fragment() {
                let pos = (self.drag_frag_origin + (mouse_pos - self.drag_mouse_origin)).round();
                ImageSelection::Fragment(pos, frag)
            } else {
                ImageSelection::Rect(self.drag_mouse_origin.round(), mouse_pos.round())
            };
        }
    }

    fn handle_mouse(&mut self, image: &ImageCollection, mouse_pos: Pos2, asset: &mut impl ImageCollectionAsset,
                    resp: &egui::Response, colors: (u8, u8)) {
        let x = mouse_pos.x.floor() as i32;
        let y = mouse_pos.y.floor() as i32;

        match self.tool {
            ImageDrawingTool::Pencil => {
                if resp.drag_started() { self.set_undo_target(asset); }
                if let Some(color) = self.get_selected_color_for_click(resp, colors) &&
                    image.set_pixel(asset.data_mut(), x, y, self.selected_image, color) {
                        self.image_changed = true;
                    }
            }

            ImageDrawingTool::Fill => {
                if resp.drag_started() { self.set_undo_target(asset); }
                if let Some(color) = self.get_selected_color_for_click(resp, colors) &&
                    image.flood_fill(asset.data_mut(), x, y, self.selected_image, color) {
                        self.image_changed = true;
                    }
            }

            ImageDrawingTool::Select => {
                self.handle_selection_mouse(image, mouse_pos, asset, resp, colors);
            }
        }
    }

    pub fn copy(&mut self, wc: &mut WindowContext, asset: &mut impl ImageCollectionAsset) {
        match &self.selection {
            ImageSelection::Rect(origin, end) => {
                let sel_rect = Rect {
                    min: Pos2::new(origin.x.min(end.x), origin.y.min(end.y)),
                    max: Pos2::new(origin.x.max(end.x), origin.y.max(end.y)),
                };
                let image = ImageCollection::from_asset(asset);
                let image_rect = ImageRect::from_rect(sel_rect, &image);
                if let Some(frag) = image.copy_fragment(asset.asset_id(), asset.data_mut(), self.selected_image, image_rect) {
                    wc.clipboard = Some(ClipboardData::Image(frag.take_pixels()));
                }
            }
            ImageSelection::Fragment(_, frag) => {
                wc.clipboard = Some(ClipboardData::Image(frag.pixels.clone()));
            }
            _ => {}
        }
    }

    pub fn cut(&mut self, wc: &mut WindowContext, asset: &mut impl ImageCollectionAsset, fill_color: u8) {
        self.set_undo_target(asset);
        self.lift_selection(asset, fill_color);
        if let Some((_, frag)) = self.selection.take_fragment() {
            wc.clipboard = Some(ClipboardData::Image(frag.take_pixels()));
        }
    }

    pub fn paste(&mut self, wc: &mut WindowContext, asset: &mut impl ImageCollectionAsset) {
        if let Some(ClipboardData::Image(pixels)) = &wc.clipboard {
            self.tool = ImageDrawingTool::Select;
            self.set_undo_target(asset);
            self.drop_selection(asset);
            self.selection = ImageSelection::Fragment(Pos2::ZERO, ImageFragment::from_pixels(asset.asset_id(), pixels.clone()));
        }
    }

    pub fn paste_pixels(&mut self, asset: &mut impl ImageCollectionAsset, pixels: ImagePixels) {
        self.tool = ImageDrawingTool::Select;
        self.set_undo_target(asset);
        self.drop_selection(asset);
        self.selection = ImageSelection::Fragment(Pos2::ZERO, ImageFragment::from_pixels(asset.asset_id(), pixels));
    }

    pub fn handle_keyboard(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, asset: &mut impl ImageCollectionAsset, fill_color: u8) {
        let ctrl_z = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::Z);
        if ui.input_mut(|i| i.consume_shortcut(&ctrl_z)) {
            self.undo(asset);
            return;
        }

        let del = egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Delete);
        if ui.input_mut(|i| i.consume_shortcut(&del)) {
            self.delete_selection(asset, fill_color);
            return;
        }

        match wc.keyboard_pressed.take() {
            Some(KeyboardPressed::CtrlC) => { self.copy(wc, asset); }
            Some(KeyboardPressed::CtrlX) => { self.cut(wc, asset, fill_color); }
            Some(KeyboardPressed::CtrlV) => { self.paste(wc, asset); }
            None => {}
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, asset: &mut impl ImageCollectionAsset, colors: (u8, u8)) {
        let slot = self.display.texture_slot();
        let (image, texture) = ImageCollection::plus_loaded_texture(asset, wc.tex_man, ui.ctx(), slot, self.image_changed);
        if self.image_changed { self.image_changed = false; }

        let image_size = image.get_item_size();
        let min_size = Vec2::splat(100.0).min(image_size + Vec2::splat(10.0)).max(ui.available_size());
        let (resp, painter) = ui.allocate_painter(min_size, Sense::drag());

        let resp_size = resp.rect.size();
        let (zoomx, zoomy) = (resp_size.x / (image_size.x + 1.0), (resp_size.y / (image_size.y + 1.0)));
        let image_zoom = f32::max(f32::min(zoomx, zoomy).floor(), 1.0);
        let center = resp.rect.center();
        let canvas_rect = Rect {
            min: center - image_zoom * image_size / 2.0,
            max: center + image_zoom * image_size / 2.0,
        };
        let canvas_to_image = emath::RectTransform::from_to(
            canvas_rect,
            Rect { min: Pos2::ZERO, max: Pos2::ZERO + image_size }
        );

        // draw background
        painter.rect_filled(canvas_rect, egui::CornerRadius::ZERO, wc.settings.image_bg_color);

        // draw image
        let item_uv = image.get_item_uv(self.selected_image);
        Image::from_texture((texture.id(), image_size)).uv(item_uv).paint_at(ui, canvas_rect);

        // draw floating selection
        if let ImageSelection::Fragment(pos, frag) = &mut self.selection {
            let slot = self.display.float_texture_slot();
            let (frag_image, frag_texture) = ImageCollection::plus_loaded_texture(frag, wc.tex_man, ui.ctx(), slot, frag.changed);
            if frag.changed { frag.changed = false; }
            let uv = frag_image.get_item_uv(0);
            let frag_size = frag_image.get_item_size();
            let frag_canvas_rect = Rect {
                min: canvas_rect.min + image_zoom * pos.to_vec2(),
                max: canvas_rect.min + image_zoom * (*pos + frag_size).to_vec2(),
            };
            Image::from_texture((frag_texture.id(), frag_size)).uv(uv).paint_at(ui, frag_canvas_rect);
        }

        // draw border
        let stroke = egui::Stroke::new(1.0, egui::Color32::BLACK);
        painter.rect_stroke(canvas_rect, egui::CornerRadius::ZERO, stroke, egui::StrokeKind::Outside);

        // draw grid
        let canvas_size = canvas_rect.size();
        let display_grid =
            self.display.has_bits(ImageDisplay::GRID) &&
            (f32::min(canvas_size.x, canvas_size.y) / f32::max(image_size.x, image_size.y) > 2.0);
        if display_grid {
            let stroke = egui::Stroke::new(1.0, wc.settings.image_grid_color);
            for y in 0..=image.height {
                let py = canvas_rect.min.y + canvas_rect.height() * y as f32 / image.height as f32;
                painter.hline(canvas_rect.x_range(), py, stroke);
            }
            for x in 0..=image.width {
                let px = canvas_rect.min.x + canvas_rect.width() * x as f32 / image.width as f32;
                painter.vline(px, canvas_rect.y_range(), stroke);
            }
        }

        if self.tool_changed {
            self.tool_changed = false;
            self.drop_selection(asset);
        }

        // handle click
        if let Some(pointer_pos) = resp.interact_pointer_pos() {
            let image_pos = canvas_to_image * pointer_pos;
            self.handle_mouse(&image, image_pos, asset, &resp, colors);
        }

        // draw selection rectangle
        if let Some(sel_rect) = self.selection.get_rect() && (sel_rect.width() > 0.0 || sel_rect.height() > 0.0) {
            let image_to_canvas = canvas_to_image.inverse();
            let sel_rect = Rect {
                min: image_to_canvas * sel_rect.min,
                max: image_to_canvas * sel_rect.max,
            };
            if sel_rect.is_positive() || resp.dragged_by(egui::PointerButton::Primary) {
                super::paint_marching_ants(&painter, sel_rect, wc.settings);
                wc.request_marching_ants_repaint();
            }
        }
    }
}
