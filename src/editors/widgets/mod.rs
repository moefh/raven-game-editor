mod color_picker;

use crate::app::ImageCollection;
use crate::data_asset::DataAssetId;
use egui::{Vec2, Sense, Image, Rect, Pos2, emath};

pub const FULL_UV : Rect = Rect { min: Pos2::ZERO, max: Pos2::new(1.0, 1.0) };

pub use color_picker::{*};

pub fn image_item_picker(ui: &mut egui::Ui, asset_id: DataAssetId, texture: &egui::TextureHandle,
                         image: &ImageCollection, selected_image: u32, zoom: f32) -> egui::scroll_area::ScrollAreaOutput<egui::Response> {
    let source = egui::scroll_area::ScrollSource { scroll_bar: true, drag: false, mouse_wheel: true };
    egui::ScrollArea::vertical().auto_shrink([true, true]).scroll_source(source).show(ui, |ui| {
        ui.scope_builder(
            egui::UiBuilder::new()
                .id_salt(format!("image_item_picker_{}", asset_id))
                .sense(Sense::drag()),
            |ui| {
                let image_size = zoom * image.get_item_size();
                let image_picker_size = zoom * image.get_full_size();
                let min_size = Vec2::splat(50.0).max(image_picker_size + Vec2::new(15.0, 5.0)).min(Vec2::new(200.0, f32::INFINITY));
                let (response, painter) = ui.allocate_painter(min_size, Sense::drag());
                let space = response.rect;
                let canvas_rect = Rect {
                    min: space.min,
                    max: space.min + image_picker_size,
                };

                // draw background
                painter.rect_filled(canvas_rect, egui::CornerRadius::ZERO, egui::Color32::from_rgb(0xe0u8, 0xffu8, 0xffu8));

                // draw items
                Image::from_texture((texture.id(), image_picker_size)).uv(FULL_UV).paint_at(ui, canvas_rect);

                // draw selection rectangle
                let stroke = egui::Stroke::new(3.0, egui::Color32::BLACK);
                let mut sel_rect = Rect {
                    min: canvas_rect.min,
                    max: canvas_rect.min + image_size,
                };
                sel_rect.min.y += (selected_image as f32) * image_size.y;
                sel_rect.max.y += (selected_image as f32) * image_size.y;
                painter.rect_stroke(sel_rect, egui::CornerRadius::ZERO, stroke, egui::StrokeKind::Inside);

                let in_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
                let sel_in_rect = sel_rect.expand2(Vec2::splat(-2.0));
                painter.rect_stroke(sel_in_rect, egui::CornerRadius::ZERO, in_stroke, egui::StrokeKind::Inside);
            }).response
    })
}

pub fn image_editor(ui: &mut egui::Ui, texture: &egui::TextureHandle, image: &ImageCollection, selected_image: u32)
                    -> (egui::Response, emath::RectTransform) {
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

    // draw background
    painter.rect_filled(resp.rect, egui::CornerRadius::ZERO, egui::Color32::from_rgb(0xe0u8, 0xffu8, 0xffu8));

    // draw image
    let item_uv = image.get_item_uv(selected_image);
    Image::from_texture((texture.id(), image_size)).uv(item_uv).paint_at(ui, canvas_rect);

    // draw border
    let stroke = egui::Stroke::new(1.0, egui::Color32::BLACK);
    painter.rect_stroke(canvas_rect, egui::CornerRadius::ZERO, stroke, egui::StrokeKind::Outside);

    // draw grid
    let canvas_size = canvas_rect.size();
    let display_grid = f32::min(canvas_size.x, canvas_size.y) / f32::max(image_size.x, image_size.y) > 3.0;
    if display_grid {
        let stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(112, 112, 112));
        for y in 0..=image.height {
            let py = canvas_rect.min.y + canvas_rect.height() * y as f32 / image.height as f32;
            painter.hline(canvas_rect.x_range(), py, stroke);
        }
        for x in 0..=image.width {
            let px = canvas_rect.min.x + canvas_rect.width() * x as f32 / image.width as f32;
            painter.vline(px, canvas_rect.y_range(), stroke);
        }
    }
    let canvas_to_image = emath::RectTransform::from_to(
        canvas_rect,
        Rect { min: Pos2::ZERO, max: Pos2::ZERO + image_size }
    );
    (resp, canvas_to_image)
}
