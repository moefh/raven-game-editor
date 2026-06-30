use crate::image::{
    colors,
    ImageCollection,
    ImageFragment,
    ImagePixels,
    ImageRect,
    ImageRotation,
};
use crate::app::{
    WindowContext,
    KeyboardPressed,
};
use crate::data_asset;

use super::super::{
    AssetIdHolder,
    ImageClipboardData,
};
use egui::{Vec2, Sense, Image, Rect, Pos2, emath};

pub enum ImageEditorAction {
    None,
    Undo,
    Cut,
    Copy,
    Paste,
    Select,
}

#[derive(Debug)]
pub enum ImageSelection {
    None,
    Rect(Pos2, Pos2),
    Fragment(Pos2, ImageFragment),
}

impl ImageSelection {
    pub fn is_floating(&self) -> bool {
        matches!(self, ImageSelection::Fragment(..))
    }

    pub fn is_empty(&self) -> bool {
        match self {
            ImageSelection::None => true,
            ImageSelection::Rect(origin, end) => {
                let width = end.x - origin.x;
                let height = end.y - origin.y;
                width.abs() == 0.0 || height.abs() == 0.0
            }
            ImageSelection::Fragment(_, _) => false,
        }
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
    Collision,
}

#[derive(Clone, Copy)]
pub struct ImageDisplay {
    pub bits: u8,
}

impl ImageDisplay {
    pub const GRID: u8        = 1 << 0;
    pub const TRANSPARENT: u8 = 1 << 1;
    pub const COLLISION: u8   = 1 << 2;

    pub fn grid_only() -> Self { ImageDisplay::new(Self::GRID) }

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
}

pub struct ImageEditorWidget<ImageAsset> {
    pub display: ImageDisplay,
    pub selection: ImageSelection,
    pub pick_left_color: Option<u8>,
    pub pick_right_color: Option<u8>,
    pub collision_rect: Option<data_asset::Rect>,
    pub selection_enabled: bool,
    pub hover_pos: Vec2,
    zoom: f32,
    scroll: Vec2,
    tool: ImageDrawingTool,
    selected_image: u32,
    undo_target: Option<ImageFragment>,
    image_changed: bool,
    drop_selection_next_show: bool,
    drag_mouse_origin: Pos2,
    drag_frag_origin: Pos2,
    _marker: std::marker::PhantomData<ImageAsset>,
}

impl<ImageAsset> ImageEditorWidget<ImageAsset> where ImageAsset: ImageCollection + AssetIdHolder {
    pub fn new() -> Self {
        ImageEditorWidget {
            _marker: std::marker::PhantomData::<ImageAsset>,
            zoom: 0.0,
            scroll: Vec2::ZERO,
            selected_image: 0,
            display: ImageDisplay::new(ImageDisplay::TRANSPARENT | ImageDisplay::GRID),
            collision_rect: None,
            tool: ImageDrawingTool::Pencil,
            selection: ImageSelection::None,
            pick_left_color: None,
            pick_right_color: None,
            drag_mouse_origin: Pos2::ZERO,
            drag_frag_origin: Pos2::ZERO,
            undo_target: None,
            selection_enabled: true,
            image_changed: false,
            drop_selection_next_show: false,
            hover_pos: Vec2::ZERO,
        }
    }

    pub fn with_selected_image(mut self, selected_image: u32) -> Self {
        self.selected_image = selected_image;
        self
    }

    pub fn with_image_display(mut self, display: ImageDisplay) -> Self {
        self.display = display;
        self
    }

    pub fn with_selection_enabled(mut self, selection_enabled: bool) -> Self {
        self.selection_enabled = selection_enabled;
        self
    }

    pub fn toggle_display(&mut self, bits: u8) {
        self.display.toggle(bits);
        if (bits & ImageDisplay::TRANSPARENT) != 0 {
            self.image_changed = true;
            self.selection.set_changed();
        }
    }

    pub fn set_image_changed(&mut self) {
        self.image_changed = true;
    }

    pub fn has_image_changed(&self) -> bool {
        self.image_changed
    }

    pub fn set_undo_target(&mut self, image: &ImageAsset) {
        self.undo_target = image.copy_fragment(image.get_asset_id(), self.selected_image, ImageRect::from_image_item(image));
    }

    pub fn force_palette(&mut self, palette: &[u8], color_to_palette_index_map: &[u8]) {
        if let ImageSelection::Fragment(_, frag) = &mut self.selection &&
            frag.pixels.force_palette(palette, color_to_palette_index_map) {
                frag.set_changed();
            }
    }

    fn lift_selection(&mut self, image: &mut ImageAsset, bg_color: u8) {
        if self.selection.is_floating() { return; } // already lifted

        if let Some(sel_rect) = self.selection.get_rect() && sel_rect.is_positive() {
            self.set_undo_target(image);
            let image_rect = ImageRect::from_rect(sel_rect, image);
            let bg_color = if self.display.is_transparent() { colors::TRANSPARENT } else { bg_color };
            if let Some(frag) = image.cut_fragment(image.get_asset_id(), self.selected_image, image_rect, bg_color) {
                self.selection = ImageSelection::Fragment(sel_rect.min, frag);
                self.image_changed = true;
            } else {
                self.selection = ImageSelection::None;
            }
        }
    }

    pub fn can_undo(&self) -> bool {
        self.undo_target.is_some()
    }

    pub fn undo(&mut self, image: &mut ImageAsset) {
        if let Some(frag) = self.undo_target.take() {
            image.paste_fragment(self.selected_image, 0, 0, &frag, false);
            self.image_changed = true;
            self.selection = ImageSelection::None;
        }
    }

    pub fn delete_selection(&mut self, image: &mut ImageAsset, fill_color: u8) {
        self.lift_selection(image, fill_color);
        self.selection = ImageSelection::None;
    }

    pub fn drop_selection(&mut self, image: &mut ImageAsset) {
        if let ImageSelection::Fragment(pos, frag) = &self.selection {
            let transparent = self.display.is_transparent();
            image.paste_fragment(self.selected_image, pos.x as i32, pos.y as i32, frag, transparent);
            self.image_changed = true;
        }
        self.selection = ImageSelection::None;
    }

    pub fn vflip(&mut self, image: &mut ImageAsset, bg_color: u8) {
        if matches!(self.selection, ImageSelection::Rect(..)) {
            self.lift_selection(image, bg_color);
        }

        if let ImageSelection::Fragment(_, frag) = &mut self.selection {
            frag.v_flip(0);
            frag.changed = true;
        } else {
            image.v_flip(self.selected_image);
            self.image_changed = true;
        }
    }

    pub fn hflip(&mut self, image: &mut ImageAsset, bg_color: u8) {
        if matches!(self.selection, ImageSelection::Rect(..)) {
            self.lift_selection(image, bg_color);
        }

        if let ImageSelection::Fragment(_, frag) = &mut self.selection {
            frag.h_flip(0);
            frag.changed = true;
        } else {
            image.h_flip(self.selected_image);
            self.image_changed = true;
        }
    }

    pub fn rotate(&mut self, image: &mut ImageAsset, rotation: ImageRotation, bg_color: u8) {
        if self.selection.is_empty() {
            if image.width() != image.height() {   // empty selection with non-square image: float to rotate
                self.tool = ImageDrawingTool::Select;
                self.selection = ImageSelection::Rect(
                    Pos2::ZERO, Pos2::new(image.width() as f32, image.height() as f32)
                );
                self.lift_selection(image, bg_color);
            }
        } else if matches!(self.selection, ImageSelection::Rect(..)) {
            self.lift_selection(image, bg_color);  // float selected rectangle to rotate
        }

        match &self.selection {
            ImageSelection::Fragment(pos, frag) => {
                if let Some(rot_frag) = frag.rotate(image.get_asset_id(), 0, rotation) {
                    let rot_pos = Pos2::new(
                        pos.x + ((frag.pixels.width as f32 - frag.pixels.height as f32) / 2.0).round_ties_even(),
                        pos.y + ((frag.pixels.height as f32 - frag.pixels.width as f32) / 2.0).round_ties_even(),
                    );
                    self.selection = ImageSelection::Fragment(rot_pos, rot_frag);
                }
            }

            _ => if let Some(rot_frag) = image.rotate(image.get_asset_id(), self.selected_image, rotation) {
                image.paste_fragment(self.selected_image, 0, 0, &rot_frag, false);
                self.image_changed = true;
            }
        }
    }

    pub fn get_tool(&self) -> ImageDrawingTool {
        self.tool
    }

    pub fn set_tool(&mut self, tool: ImageDrawingTool) {
        self.set_tool_without_dropping_selection(tool);
        self.drop_selection_next_show = true;
    }

    pub fn set_tool_without_dropping_selection(&mut self, tool: ImageDrawingTool) {
        self.tool = tool;
    }

    pub fn get_selected_image(&self) -> u32 {
        self.selected_image
    }

    pub fn set_selected_image(&mut self, sel_image: u32, image: &ImageAsset) -> bool {
        if self.selected_image != sel_image {
            self.selected_image = sel_image;
            self.set_undo_target(image);
            true
        } else {
            false
        }
    }

    pub fn set_collision_rect(&mut self, rect: Option<data_asset::Rect>) {
        self.collision_rect = rect;
    }

    pub fn get_collision_rect(&mut self) -> Option<data_asset::Rect> {
        self.collision_rect
    }

    fn get_selected_color_for_click(resp: &egui::Response, colors: (u8, u8)) -> Option<u8> {
        if resp.dragged_by(egui::PointerButton::Primary) {
            Some(colors.0)
        } else if resp.dragged_by(egui::PointerButton::Secondary) {
            Some(colors.1)
        } else {
            None
        }
    }

    fn handle_selection_mouse(&mut self, mouse_pos: Pos2, image: &mut ImageAsset,
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
                        self.lift_selection(image, colors.1);

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
                            self.drop_selection(image);
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

    fn handle_collision_mouse(&mut self, mouse_pos: Pos2, image: &mut ImageAsset, resp: &egui::Response) {
        let make_rect = || { data_asset::Rect::new(0, 0, image.width() as i32, image.height() as i32) };

        let mouse_pos = Rect::from_min_size(Pos2::ZERO, image.get_item_size()).clamp(mouse_pos);
        let mouse_pos = (mouse_pos.x.floor() as i32, mouse_pos.y.floor() as i32);
        let rect = self.collision_rect.get_or_insert_with(make_rect);
        if resp.dragged_by(egui::PointerButton::Primary) {
            // set top-left
            let dx = mouse_pos.0 - rect.x;
            let dy = mouse_pos.1 - rect.y;
            rect.x += dx;
            rect.y += dy;
            rect.w -= dx;
            rect.h -= dy;
            if rect.w < 0 { rect.x += rect.w; rect.w = 0; }
            if rect.h < 0 { rect.y += rect.h; rect.h = 0; }
        }
        if resp.dragged_by(egui::PointerButton::Secondary) {
            // set bottom-right
            rect.w = (mouse_pos.0 - rect.x).max(0);
            rect.h = (mouse_pos.1 - rect.y).max(0);
        }
    }

    fn pick_color(&mut self, x: i32, y: i32, image: &mut ImageAsset, resp: &egui::Response) -> bool {
        let mouse_in_image = x >= 0 && y >= 0 && (x as u32) < image.width() && (y as u32) < image.height();
        if ! mouse_in_image { return false; }

        let color = image.get_pixel(x, y, self.selected_image);
        if resp.dragged_by(egui::PointerButton::Primary) {
            self.pick_left_color = Some(color);
            return true;
        }
        if resp.dragged_by(egui::PointerButton::Secondary) {
            self.pick_right_color = Some(color);
            return true
        }
        false
    }

    fn handle_mouse(&mut self, mouse_pos: Pos2, image: &mut ImageAsset, resp: &egui::Response, colors: (u8, u8)) {
        let x = mouse_pos.x.floor() as i32;
        let y = mouse_pos.y.floor() as i32;

        let ctrl_held = resp.ctx.input(|i| i.modifiers.ctrl);

        match self.tool {
            ImageDrawingTool::Pencil => {
                if ctrl_held {
                    self.pick_color(x, y, image, resp);
                } else {
                    if resp.drag_started() { self.set_undo_target(image); }
                    if let Some(color) = Self::get_selected_color_for_click(resp, colors) &&
                        image.set_pixel(x, y, self.selected_image, color) {
                            self.image_changed = true;
                        }
                }
            }

            ImageDrawingTool::Fill => {
                if ctrl_held {
                    self.pick_color(x, y, image, resp);
                } else {
                    if resp.drag_started() { self.set_undo_target(image); }
                    if let Some(color) = Self::get_selected_color_for_click(resp, colors) &&
                        image.flood_fill(x, y, self.selected_image, color) {
                            self.image_changed = true;
                        }
                }
            }

            ImageDrawingTool::Select => {
                self.handle_selection_mouse(mouse_pos, image, resp, colors);
            }

            ImageDrawingTool::Collision => {
                self.handle_collision_mouse(mouse_pos, image, resp);
            }
        }
    }

    pub fn copy(&mut self, wc: &mut WindowContext, image: &ImageAsset) {
        match &self.selection {
            ImageSelection::Rect(origin, end) => {
                let sel_rect = Rect {
                    min: Pos2::new(origin.x.min(end.x), origin.y.min(end.y)),
                    max: Pos2::new(origin.x.max(end.x), origin.y.max(end.y)),
                };
                let image_rect = ImageRect::from_rect(sel_rect, image);
                if let Some(frag) = image.copy_fragment(image.get_asset_id(), self.selected_image, image_rect) {
                    wc.image_clipboard = ImageClipboardData::Image(frag.take_pixels());
                }
            }
            ImageSelection::Fragment(_, frag) => {
                wc.image_clipboard = ImageClipboardData::Image(frag.pixels.clone());
            }
            _ => {}
        }
    }

    pub fn cut(&mut self, wc: &mut WindowContext, image: &mut ImageAsset, fill_color: u8) {
        self.set_undo_target(image);
        self.lift_selection(image, fill_color);
        if let Some((_, frag)) = self.selection.take_fragment() {
            wc.image_clipboard = ImageClipboardData::Image(frag.take_pixels());
        }
    }

    pub fn paste(&mut self, wc: &mut WindowContext, image: &mut ImageAsset) {
        if let ImageClipboardData::Image(pixels) = &wc.image_clipboard {
            self.tool = ImageDrawingTool::Select;
            self.set_undo_target(image);
            self.drop_selection(image);
            self.selection = ImageSelection::Fragment(Pos2::ZERO, ImageFragment::from_pixels(image.get_asset_id(), pixels.clone()));
        }
    }

    pub fn paste_pixels(&mut self, image: &mut ImageAsset, pixels: ImagePixels) {
        self.tool = ImageDrawingTool::Select;
        self.set_undo_target(image);
        self.drop_selection(image);
        self.selection = ImageSelection::Fragment(Pos2::ZERO, ImageFragment::from_pixels(image.get_asset_id(), pixels));
    }

    pub fn handle_keyboard(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, image: &mut ImageAsset, fill_color: u8) -> ImageEditorAction {
        let ctrl_z = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::Z);
        if ui.input_mut(|i| i.consume_shortcut(&ctrl_z)) {
            self.undo(image);
            return ImageEditorAction::Undo;
        }

        if self.selection_enabled {
            let ctrl_a = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::A);
            if ui.input_mut(|i| i.consume_shortcut(&ctrl_a)) {
                if self.tool != ImageDrawingTool::Select {
                    self.set_tool_without_dropping_selection(ImageDrawingTool::Select);
                }
                let width = image.width() as f32;
                let height = image.height() as f32;
                match self.selection {
                    ImageSelection::None => {
                        self.selection = ImageSelection::Rect(Pos2::ZERO, Pos2::new(width, height));
                    }
                    ImageSelection::Rect(origin, end) => {
                        self.selection = if origin.x == 0.0 && origin.y == 0.0 && end.x == width && end.y == height {
                            ImageSelection::None
                        } else {
                            ImageSelection::Rect(Pos2::ZERO, Pos2::new(width, height))
                        };
                    }
                    _ => {}
                }
                return ImageEditorAction::Select;
            }
        }

        let del = egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Delete);
        if ui.input_mut(|i| i.consume_shortcut(&del)) {
            self.delete_selection(image, fill_color);
            return ImageEditorAction::None;
        }

        match wc.keyboard_pressed.take() {
            Some(KeyboardPressed::CtrlV) if self.selection_enabled => {
                self.paste(wc, image);
                return ImageEditorAction::Paste;
            }
            Some(KeyboardPressed::CtrlC) if self.selection_enabled => {
                self.copy(wc, image);
                return ImageEditorAction::Copy;
            }
            Some(KeyboardPressed::CtrlX) if self.selection_enabled => {
                self.cut(wc, image, fill_color);
                return ImageEditorAction::Cut;
            }
            _ => {}
        }

        ImageEditorAction::None
    }

    pub fn update_texture(wc: &mut WindowContext, image: &impl ImageCollection) {
        image.load_texture(wc.tex_man, wc.egui.ctx, image.texture_slot(false, false), true);
        image.load_texture(wc.tex_man, wc.egui.ctx, image.texture_slot(true, false), true);
    }

    fn draw_background(&self, painter: &egui::Painter, wc: &WindowContext, bg_rect: Rect, image: &ImageAsset) {
        painter.rect_filled(bg_rect, egui::CornerRadius::ZERO, wc.settings.image_bg_color);
        if self.zoom < 3.0 { return; }

        let stroke = egui::Stroke::new(1.0, wc.settings.image_grid_color);
        let width = image.width();
        let height = image.height();
        let (x_min, y_min) = if width > height {
            for x in 0..width-height {
                let p1 = Vec2::new(x as f32, 0.0);
                let p2 = Vec2::new((x + height) as f32, height as f32);
                painter.line_segment([bg_rect.min + self.zoom * p1, bg_rect.min + self.zoom * p2], stroke);
            }
            (width-height, 1)
        } else if height > width {
            for y in 0..height-width {
                let p1 = Vec2::new(0.0, y as f32);
                let p2 = Vec2::new(width as f32, (y + width) as f32);
                painter.line_segment([bg_rect.min + self.zoom * p1, bg_rect.min + self.zoom * p2], stroke);
            }
            (1, height-width)
        } else {
            (1, 0)
        };
        for x in x_min..width {
            let p1 = Vec2::new(x as f32, 0.0);
            let p2 = Vec2::new(width as f32, (width - x) as f32);
            painter.line_segment([bg_rect.min + self.zoom * p1, bg_rect.min + self.zoom * p2], stroke);
        }
        for y in y_min..height {
            let p1 = Vec2::new(0.0, y as f32);
            let p2 = Vec2::new((height - y) as f32, height as f32);
            painter.line_segment([bg_rect.min + self.zoom * p1, bg_rect.min + self.zoom * p2], stroke);
        }
    }

    pub fn set_zoom(&mut self, zoom: f32, canvas_size: Vec2, zoom_center: Vec2, image: &ImageAsset) {
        let zoom = zoom.max(1.0);
        let zoomed_image_size = image.get_item_size() * zoom;
        let zoom_delta = zoom / self.zoom;
        self.zoom = zoom;

        self.scroll = zoom_center - (zoom_center - self.scroll) * zoom_delta;
        self.clip_scroll(canvas_size, zoomed_image_size);
    }

    pub fn clip_scroll(&mut self, canvas_size: Vec2, zoomed_image_size: Vec2) {
        self.scroll = self.scroll.max(canvas_size - zoomed_image_size).min(Vec2::ZERO);
    }

    pub fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, image: &mut ImageAsset, colors: (u8, u8)) {
        if self.image_changed {
            Self::update_texture(wc, image);
            self.image_changed = false;
        }
        let min_size = Vec2::splat(100.0).max(ui.available_size());
        let (resp, mut painter) = ui.allocate_painter(min_size, Sense::drag());
        if ui.is_sizing_pass() { return; }
        let canvas_rect = resp.rect;
        let image_size = image.get_item_size();
        if self.zoom == 0.0 {
            let canvas_size = canvas_rect.size();
            let (zoomx, zoomy) = (canvas_size.x / image_size.x, canvas_size.y / image_size.y);
            self.zoom = f32::max(f32::min(zoomx, zoomy), 1.0);
        }
        let zoomed_image_size = image_size * self.zoom;
        let image_canvas_rect = if zoomed_image_size.x >= canvas_rect.width() && zoomed_image_size.y >= canvas_rect.height() {
            canvas_rect
        } else {
            let size = zoomed_image_size.min(canvas_rect.size());
            Rect::from_min_size(canvas_rect.min + 0.5 * (canvas_rect.size() - size).max(Vec2::ZERO), size)
        };
        let grid_stroke = egui::Stroke::new(1.0, wc.settings.image_grid_color);
        painter.rect_stroke(image_canvas_rect, egui::CornerRadius::ZERO, grid_stroke, egui::StrokeKind::Inside);
        let image_area_rect = image_canvas_rect.expand2(Vec2::splat(-1.0));
        ui.shrink_clip_rect(image_area_rect);
        painter.shrink_clip_rect(image_area_rect);

        let canvas_to_image = emath::RectTransform::from_to(
            Rect::from_min_size(image_area_rect.min + self.scroll, zoomed_image_size),
            Rect::from_min_size(Pos2::ZERO, image_size),
        );
        let paint_image_rect = Rect::from_min_size(image_area_rect.min + self.scroll, zoomed_image_size);

        self.clip_scroll(canvas_rect.size(), zoomed_image_size); // in case we've been resized

        // draw background
        self.draw_background(&painter, wc, paint_image_rect, image);

        // draw image
        let slot = image.texture_slot(self.display.is_transparent(), false);
        let texture = image.texture(wc.tex_man, ui.ctx(), slot);
        let item_uv = image.get_item_uv(self.selected_image);
        Image::from_texture((texture.id(), image_size)).uv(item_uv).paint_at(ui, paint_image_rect);

        // draw floating selection
        if let ImageSelection::Fragment(pos, frag) = &mut self.selection {
            let slot = image.texture_slot(self.display.is_transparent(), true);
            let frag_texture = frag.load_texture(wc.tex_man, ui.ctx(), slot, frag.changed);
            if frag.changed { frag.changed = false; }
            let uv = frag.get_item_uv(0);
            let frag_size = frag.get_item_size();
            let frag_canvas_rect = Rect {
                min: image_area_rect.min + self.zoom * pos.to_vec2(),
                max: image_area_rect.min + self.zoom * (*pos + frag_size).to_vec2(),
            };
            Image::from_texture((frag_texture.id(), frag_size)).uv(uv).paint_at(ui, frag_canvas_rect);
        }

        // draw grid and border
        let display_grid = self.display.has_bits(ImageDisplay::GRID) && self.zoom >= 3.0;
        if display_grid {
            let stroke = egui::Stroke::new(1.0, wc.settings.image_grid_color);
            for y in 0..image.height()+1 {
                let py = image_area_rect.min.y + y as f32 * self.zoom + self.scroll.y % self.zoom;
                painter.hline(image_area_rect.x_range(), py, stroke);
            }
            for x in 0..image.width()+1 {
                let px = image_area_rect.min.x + x as f32 * self.zoom + self.scroll.x % self.zoom;
                painter.vline(px, image_area_rect.y_range(), stroke);
            }
        }

        if self.drop_selection_next_show {
            self.drop_selection_next_show = false;
            self.drop_selection(image);
        }

        // ====================================================
        // == handle input

        let keys_pressed = resp.ctx.input(|i| i.modifiers);

        // set move cursor if ALT is pressed
        if resp.contains_pointer() && resp.hovered() && keys_pressed.alt {
            if resp.dragged() {
                resp.ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
            } else {
                resp.ctx.set_cursor_icon(egui::CursorIcon::Grab);
            }
        }

        // check zoom
        if resp.contains_pointer() && let Some(hover_pos) = ui.input(|i| i.pointer.hover_pos()) {
            let zoom_delta = ui.input(|i| i.zoom_delta());
            if zoom_delta != 1.0 {
                self.set_zoom(self.zoom * zoom_delta, canvas_rect.size(), hover_pos - image_area_rect.min, image);
            }
            self.hover_pos = ((hover_pos - image_area_rect.min - self.scroll) / self.zoom).floor().max(Vec2::ZERO);
        }

        // check pan
        if resp.dragged_by(egui::PointerButton::Middle) || keys_pressed.alt {
            self.scroll += resp.drag_delta();
            self.clip_scroll(canvas_rect.size(), zoomed_image_size);
        }

        // check click
        if let Some(pointer_pos) = resp.interact_pointer_pos() && ! keys_pressed.alt {
            let image_pos = canvas_to_image * pointer_pos;
            self.handle_mouse(image_pos, image, &resp, colors);
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

        // draw collision
        if self.display.has_bits(ImageDisplay::COLLISION) && let Some(col_rect) = self.collision_rect {
            let image_to_canvas = canvas_to_image.inverse();
            let col_rect = Rect {
                min: image_to_canvas * Pos2::new(col_rect.x as f32, col_rect.y as f32),
                max: image_to_canvas * Pos2::new((col_rect.x+col_rect.w) as f32, (col_rect.y+col_rect.h) as f32),
            };
            super::paint_ants(&painter, col_rect, wc.settings, 0);
        }
    }
}
