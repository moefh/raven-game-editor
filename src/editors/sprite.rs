use crate::app::{WindowContext, ImageCollection};
use crate::data_asset::{Sprite, DataAssetId};
use egui::{Vec2, Sense, Image, Rect, Pos2, emath};

pub struct SpriteEditor {
    pub asset: super::DataAssetEditor,
    force_reload_image: bool,
    selected_frame: u32,
}

const FULL_UV : Rect = Rect { min: Pos2::ZERO, max: Pos2::new(1.0, 1.0) };

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
            selected_frame: 0,
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, sprite: &mut Sprite) {
        let title = format!("{} - Sprite", sprite.asset.name);
        let window = super::create_editor_window(sprite.asset.id, &title, wc.window_space);
        window.open(&mut self.asset.open).show(wc.egui.ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Sprite name:");
                ui.text_edit_singleline(&mut sprite.asset.name);
            });
            ui.add_space(5.0);

            let (image, texture) = ImageCollection::sprite(wc.tex_man, wc.egui.ctx, sprite, self.force_reload_image);
            self.force_reload_image = false;

            ui.horizontal_top(|ui| {
                // item picker:
                let source = egui::scroll_area::ScrollSource { scroll_bar: true, drag: false, mouse_wheel: true };
                let frame_size = image.get_item_size();
                let frame_picker_size = image.get_full_size();
                let scroll = egui::ScrollArea::vertical().auto_shrink([true, true]).scroll_source(source).show(ui, |ui| {
                    ui.scope_builder(
                        egui::UiBuilder::new()
                            .id_salt(format!("sprite_{}_frames", sprite.asset.id))
                            .sense(Sense::drag()),
                        |ui| {
                            let min_size = Vec2::new(50.0, 50.0).max(frame_picker_size + Vec2::splat(15.0));
                            let (response, painter) = ui.allocate_painter(min_size, Sense::drag());
                            let space = response.rect;
                            let canvas_rect = Rect {
                                min: space.min,
                                max: space.min + frame_picker_size,
                            };

                            // draw items
                            Image::from_texture((texture.id(), frame_picker_size)).uv(FULL_UV).paint_at(ui, canvas_rect);

                            // draw selection rectangle
                            let stroke = egui::Stroke::new(2.0, egui::Color32::BLACK);
                            let mut sel_rect = Rect {
                                min: canvas_rect.min,
                                max: canvas_rect.min + frame_size,
                            };
                            sel_rect.min.y += (self.selected_frame as f32) * frame_size.y;
                            sel_rect.max.y += (self.selected_frame as f32) * frame_size.y;
                            painter.rect_stroke(sel_rect, egui::CornerRadius::ZERO, stroke, egui::StrokeKind::Inside);
                        }).response
                });
                if let Some(pointer_pos) = scroll.inner.interact_pointer_pos() {
                    let pos = pointer_pos - scroll.inner_rect.min + scroll.state.offset;
                    if pos.x >= 0.0 && pos.x <= scroll.inner_rect.width() {
                        self.selected_frame = u32::min((pos.y / frame_size.y).floor() as u32, image.num_items-1);
                    }
                };

                // image:
                ui.scope_builder(
                    egui::UiBuilder::new()
                        .id_salt(format!("sprite_{}_frames", sprite.asset.id)),
                        |ui| {
                            let min_size = Vec2::new(100.0, 100.0).min(1.1 * frame_size).max(ui.available_size());
                            let (resp, painter) = ui.allocate_painter(min_size, Sense::drag());

                            let image_zoom = get_image_zoom(frame_size, resp.rect.size());
                            let center = resp.rect.center();
                            let canvas_rect = Rect {
                                min: center - image_zoom * frame_size / 2.0,
                                max: center + image_zoom * frame_size / 2.0,
                            };
                            let frame_uv = image.get_item_uv(self.selected_frame);
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

                            if let Some(pointer_pos) = resp.interact_pointer_pos() {
                                let screen_to_frame = emath::RectTransform::from_to(
                                    canvas_rect,
                                    Rect { min: Pos2::ZERO, max: Pos2::ZERO + frame_size }
                                );
                                let canvas_pos = screen_to_frame * pointer_pos;
                                let x = canvas_pos.x as i32;
                                let y = canvas_pos.y as i32;
                                self.force_reload_image = image.set_pixel(&mut sprite.data, x, y, self.selected_frame, 0u8);
                            }
                        });

                // TODO: color picker
            });
        });
    }
}
