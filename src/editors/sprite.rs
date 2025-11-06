use crate::app::{WindowContext, ImageCollection};
use crate::data_asset::{Sprite, DataAssetId};
use egui::{Vec2, Sense, Image, Rect, Pos2, emath};

pub struct SpriteEditor {
    pub asset: super::DataAssetEditor,
    force_reload_image: bool,
}

fn get_image_zoom(image_size: Vec2, canvas_size: Vec2) -> f32 {
    let (zoomx, zoomy) = (canvas_size.x / (image_size.x + 1.0), (canvas_size.y / (image_size.y + 1.0)));
    f32::max(f32::min(zoomx, zoomy).floor(), 1.0)
}

impl SpriteEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        SpriteEditor {
            asset: super::DataAssetEditor {
                id,
                open,
            },
            force_reload_image: false,
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, sprite: &mut Sprite) {
        let frame = 0; // the sprite frame we're seeing
        let title = format!("{} - Sprite", sprite.asset.name);
        let window = super::create_editor_window(sprite.asset.id, &title, wc.window_space);
        window.open(&mut self.asset.open).show(wc.egui.ctx, |ui| {
            ui.text_edit_singleline(&mut sprite.asset.name);

            let (image, texture) = ImageCollection::sprite(wc.tex_man, wc.egui.ctx, sprite, self.force_reload_image);
            self.force_reload_image = false;
            egui::ScrollArea::neither().auto_shrink([false, false]).show(ui, |ui| {
                let frame_size = image.get_item_size();
                let min_size = Vec2::new(100.0, 100.0).min(1.1 * frame_size).max(ui.available_size());
                let (resp, painter) = ui.allocate_painter(min_size, Sense::drag());

                let image_zoom = get_image_zoom(frame_size, resp.rect.size());
                let center = resp.rect.center();
                let canvas_rect = Rect {
                    min: center - image_zoom * frame_size / 2.0,
                    max: center + image_zoom * frame_size / 2.0,
                };
                let frame_uv = image.get_item_uv(frame);
                Image::from_texture((texture.id(), frame_size)).uv(frame_uv).paint_at(ui, canvas_rect);

                let canvas_size = canvas_rect.size();
                let display_grid = f32::min(canvas_size.x, canvas_size.y) / f32::max(frame_size.x, frame_size.y) > 3.0;
                if display_grid {
                    let stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(112, 112, 112));
                    for y in 0..=sprite.height {
                        let py = canvas_rect.min.y + canvas_rect.height() * y as f32 / sprite.height as f32;
                        painter.hline(canvas_rect.x_range(), py, stroke);
                    }
                    for x in 0..=sprite.width {
                        let px = canvas_rect.min.x + canvas_rect.width() * x as f32 / sprite.width as f32;
                        painter.vline(px, canvas_rect.y_range(), stroke);
                    }
                }

                let screen_to_frame = emath::RectTransform::from_to(
                    canvas_rect,
                    Rect { min: Pos2::ZERO, max: Pos2::ZERO + frame_size }
                );
                if let Some(pointer_pos) = resp.interact_pointer_pos() {
                    let canvas_pos = screen_to_frame * pointer_pos;
                    let x = canvas_pos.x as i32;
                    let y = canvas_pos.y as i32;
                    self.force_reload_image = image.set_pixel(&mut sprite.data, x, y, frame, 0u8);
                }
            });
        });
    }
}
