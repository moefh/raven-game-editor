use crate::app::{WindowContext, AppTextureName, ImageCollection};
use crate::data_asset::{Sprite, DataAssetId};
use egui::{Vec2, Sense, Image, Rect, Pos2, emath};

pub struct SpriteEditor {
    pub asset: super::DataAssetEditor,
    tex: AppTextureName,
}

impl SpriteEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        SpriteEditor {
            asset: super::DataAssetEditor {
                id,
                open,
            },
            tex: AppTextureName::new(id, 0),
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, sprite: &mut Sprite) {
        let title = format!("{} - Sprite", sprite.asset.name);
        let window = super::create_editor_window(sprite.asset.id, &title, wc.window_space);
        window.open(&mut self.asset.open).show(wc.egui.ctx, |ui| {
            let image = ImageCollection::from_sprite(sprite, self.tex);
            let texture = image.get_sprite_texture(wc.tex_man, wc.egui.ctx, sprite);
            
            ui.text_edit_singleline(&mut sprite.asset.name);
            egui::ScrollArea::neither().auto_shrink([false, false]).show(ui, |ui| {
                let zoom = 5.0;
                let (resp, painter) = ui.allocate_painter(Vec2::new(zoom*image.width as f32, zoom*image.height as f32), Sense::drag());
                let rect = resp.rect;
                let tile_size = image.get_item_size();
                let tile_uv = image.get_item_uv(8);
                Image::from_texture((texture.id(), tile_size)).uv(tile_uv).paint_at(ui, rect);

                let stroke = egui::Stroke::new(1.0, egui::Color32::BLUE);
                for y in 0..=sprite.height {
                    let py = rect.min.y + rect.height() * y as f32 / sprite.height as f32;
                    painter.hline(rect.x_range(), py, stroke);
                }
                for x in 0..=sprite.width {
                    let px = rect.min.x + rect.width() * x as f32 / sprite.width as f32;
                    painter.vline(px, rect.y_range(), stroke);
                }

                let screen_to_tile = emath::RectTransform::from_to(
                    rect,
                    Rect { min: Pos2::ZERO, max: Pos2::ZERO + tile_size }
                );
                if let Some(pointer_pos) = resp.interact_pointer_pos() {
                    let canvas_pos = screen_to_tile * pointer_pos;
                    let x = canvas_pos.x as i32;
                    let y = canvas_pos.y as i32;
                    
                    println!("{},{}", x, y);
                }
            });
        });
    }
}
