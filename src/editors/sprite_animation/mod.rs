mod properties;

use crate::IMAGES;
use crate::app::WindowContext;
use crate::image::ImageCollection;
use crate::data_asset::{
    SpriteAnimation, SpriteAnimationFrame, Sprite,
    DataAssetId, GenericAsset, AssetList, AssetIdList,
};

use properties::PropertiesDialog;
use super::widgets::{ColorPickerState, ImageEditorState, SpriteFrameListView};

enum EditorTabs {
    Sprite,
    Frames,
}

#[derive(Clone)]
struct FrameDragPayload {
    frame: usize,
}

impl FrameDragPayload {
    fn new(frame: usize) -> Self {
        FrameDragPayload {
            frame,
        }
    }
}

pub struct SpriteAnimationEditor {
    pub asset: super::DataAssetEditor,
    force_reload_image: bool,
    properties_dialog: Option<PropertiesDialog>,
    selected_tab: EditorTabs,
    selected_loop: usize,
    selected_loop_frame: usize,
    sprite_frames: Vec<SpriteAnimationFrame>,
    selected_sprite_frame: usize,
    color_picker: ColorPickerState,
    image_editor: ImageEditorState,
}

fn build_sprite_frames(frames: &mut Vec<SpriteAnimationFrame>, num_frames: u32) {
    frames.clear();
    for index in 0..num_frames as u8 {
        frames.push(SpriteAnimationFrame { head_index: Some(index), foot_index: None });
    }
}

impl SpriteAnimationEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        SpriteAnimationEditor {
            asset: super::DataAssetEditor::new(id, open),
            force_reload_image: false,
            properties_dialog: None,
            selected_tab: EditorTabs::Sprite,
            selected_loop: 0,
            selected_loop_frame: 0,
            sprite_frames: Vec::new(),
            selected_sprite_frame: 0,
            color_picker: ColorPickerState::new(0b000011, 0b001100),
            image_editor: ImageEditorState::new(),
        }
    }

    pub fn prepare_for_saving(&mut self, animation: &mut SpriteAnimation, sprites: &mut AssetList<Sprite>) {
        if let Some(sprite) = sprites.get_mut(&animation.sprite_id) {
            self.image_editor.drop_selection(sprite);
        }
    }

    fn select_loop(&mut self, selected_loop: usize) {
        self.selected_loop = selected_loop;
        self.selected_loop_frame = 0;
    }

    fn sprite_tab(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, animation: &mut SpriteAnimation,
                  _sprite_ids: &AssetIdList, sprites: &mut AssetList<Sprite>) {
        let sprite = match sprites.get_mut(&animation.sprite_id) {
            Some(s) => s,
            None => { return; }
        };

        let asset_id = animation.asset.id;

        // color picker:
        egui::SidePanel::right(format!("editor_panel_{}_right", asset_id)).resizable(false).show_inside(ui, |ui| {
            ui.add_space(5.0);
            super::widgets::color_picker(ui, wc, &mut self.color_picker);
        });

        // loop frames:
        egui::TopBottomPanel::bottom(format!("editor_panel_{}_loop_frames", asset_id)).show_inside(ui, |ui| {
            ui.add_space(8.0);
            if let Some(aloop) = animation.loops.get(self.selected_loop) {
                let slot = self.image_editor.display.texture_slot();
                let (image, texture) = ImageCollection::load_asset_texture(sprite, wc.tex_man, wc.egui.ctx, slot, self.force_reload_image);
                if self.force_reload_image { self.force_reload_image = false; }
                let view = SpriteFrameListView::new(texture, &image, &aloop.frame_indices,
                                                    animation.foot_overlap, self.selected_loop_frame);
                let scroll = view.show(ui);
                let num_frames = aloop.frame_indices.len();
                if num_frames != 0 &&
                    let Some(pointer_pos) = scroll.inner.interact_pointer_pos() &&
                    scroll.inner_rect.contains(pointer_pos) {
                        let pos = pointer_pos - scroll.inner_rect.min + scroll.state.offset;
                        let frame_size = image.get_item_size();
                        self.selected_loop_frame = usize::min((pos.x / frame_size.x).floor() as usize, num_frames - 1);
                    }
            }
        });

        // body:
        egui::CentralPanel::default().show_inside(ui, |ui| {
            if let Some(image_item) = animation.loops.get(self.selected_loop)
                .and_then(|aloop| aloop.frame_indices.get(self.selected_loop_frame))
                .and_then(|frame| frame.head_index)  {
                    self.image_editor.selected_image = image_item as u32;
                    let colors = (self.color_picker.left_color, self.color_picker.right_color);
                    super::widgets::image_editor(ui, wc, sprite, &mut self.image_editor, colors);
                }
        });
    }

    fn frames_tab(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, animation: &mut SpriteAnimation,
                  _sprite_ids: &AssetIdList, sprites: &mut AssetList<Sprite>) {
        let sprite = match sprites.get_mut(&animation.sprite_id) {
            Some(s) => s,
            None => { return; }
        };

        let asset_id = animation.asset.id;
        let slot = self.image_editor.display.texture_slot();
        let (image, texture) = ImageCollection::get_asset_texture(sprite, wc.tex_man, wc.egui.ctx, slot);

        egui::TopBottomPanel::top(format!("editor_panel_{}_loop_sel_frames", asset_id)).show_inside(ui, |ui| {
            ui.add_space(5.0);
            if let Some(aloop) = animation.loops.get_mut(self.selected_loop) {
                egui::Grid::new(format!("editor_panel_{}_prop_grid", animation.asset.id))
                    .num_columns(2)
                    .spacing([4.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut aloop.name);
                        ui.end_row();

                        ui.label("Length:");
                        ui.horizontal(|ui| {
                            let min_frames = if self.selected_loop == 0 { 1 } else { 0 };
                            if ui.button("\u{2796}").clicked() && aloop.frame_indices.len() > min_frames {
                                aloop.frame_indices.pop();
                            }
                            ui.label(format!("{}", aloop.frame_indices.len()));
                            if ui.button("\u{2795}").clicked() && aloop.frame_indices.len() < (u8::MAX-2) as usize {
                                aloop.frame_indices.push(SpriteAnimationFrame { head_index: None, foot_index: None });
                            }

                            ui.separator();

                            ui.label("Foot overlap:");
                            if ui.button("\u{2796}").clicked() && animation.foot_overlap > i8::MIN {
                                animation.foot_overlap -= 1;
                            }
                            ui.label(format!("{}", animation.foot_overlap));
                            if ui.button("\u{2795}").clicked() && animation.foot_overlap < i8::MAX {
                                animation.foot_overlap += 1;
                            }
                        });
                        ui.end_row();
                    });
                ui.add_space(5.0);
                let view = SpriteFrameListView::new(texture, &image, &aloop.frame_indices,
                                                    animation.foot_overlap, aloop.frame_indices.len() + 1);
                view.show(ui);
            }
        });

        egui::TopBottomPanel::bottom(format!("editor_panel_{}_loop_all_frames", asset_id)).show_inside(ui, |ui| {
            ui.add_space(5.0);
            ui.label("Sprite frames (drag to the lists above):");
            let view = SpriteFrameListView::new(texture, &image, &self.sprite_frames, 0, self.selected_sprite_frame);
            let scroll = view.show(ui);
            let num_frames = self.sprite_frames.len();
            if num_frames != 0 &&
                let Some(pointer_pos) = scroll.inner.interact_pointer_pos() &&
                scroll.inner_rect.contains(pointer_pos) &&
                scroll.inner.drag_started() {
                    let pos = pointer_pos - scroll.inner_rect.min + scroll.state.offset;
                    let frame_size = image.get_item_size();
                    self.selected_sprite_frame = usize::min((pos.x / frame_size.x).floor() as usize, num_frames - 1);
                    scroll.inner.dnd_set_drag_payload(FrameDragPayload::new(self.selected_sprite_frame));
                }
        });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            let drop_frame = egui::Frame::default().inner_margin(2.0);
            if let Some(aloop) = animation.loops.get_mut(self.selected_loop) {
                ui.take_available_space();
                ui.columns_const(|[head_ui, foot_ui]| {
                    head_ui.label("Head frames:");
                    head_ui.take_available_space();
                    egui::ScrollArea::both().id_salt("head").auto_shrink([false, false]).show(head_ui, |ui| {
                        for frame in &mut aloop.frame_indices {
                            let (_, dropped_payload) = ui.dnd_drop_zone::<FrameDragPayload, ()>(drop_frame, |ui| {
                                let name = match frame.head_index {
                                    Some(index) => &format!("{}", index),
                                    None => "(empty)",
                                };
                                let label = ui.add(egui::Label::new(name).selectable(false).sense(egui::Sense::click()));
                                egui::Popup::context_menu(&label).show(|ui| {
                                    if ui.button("Remove").clicked() {
                                        frame.head_index.take();
                                    }
                                });
                            });
                            if let Some(payload) = dropped_payload {
                                frame.head_index.replace(std::sync::Arc::unwrap_or_clone(payload).frame as u8);
                            }
                        }
                    });

                    foot_ui.label("Foot frames:");
                    egui::ScrollArea::both().id_salt("foot").auto_shrink([false, false]).show(foot_ui, |ui| {
                        for frame in &mut aloop.frame_indices {
                            let (_, dropped_payload) = ui.dnd_drop_zone::<FrameDragPayload, ()>(drop_frame, |ui| {
                                let name = match frame.foot_index {
                                    Some(index) => &format!("{}", index),
                                    None => "(empty)",
                                };
                                let label = ui.add(egui::Label::new(name).selectable(false).sense(egui::Sense::click()));
                                egui::Popup::context_menu(&label).show(|ui| {
                                    if ui.button("Remove").clicked() {
                                        frame.foot_index.take();
                                    }
                                });
                            });
                            if let Some(payload) = dropped_payload {
                                frame.foot_index.replace(std::sync::Arc::unwrap_or_clone(payload).frame as u8);
                            }
                        }
                    });
                });
            }
        });
    }

    pub fn show(&mut self, wc: &mut WindowContext, animation: &mut SpriteAnimation,
                sprite_ids: &AssetIdList, sprites: &mut AssetList<Sprite>) {
        if let Some(dlg) = &mut self.properties_dialog && dlg.open {
            dlg.show(wc, animation, sprite_ids, sprites);
        }

        if let Some(sprite) = sprites.get(&animation.sprite_id) {
            if sprite.num_frames as usize != self.sprite_frames.len() {
                build_sprite_frames(&mut self.sprite_frames, sprite.num_frames);
            }
        } else {
            return;  // animation has an invalid sprite id
        }

        let asset_id = animation.asset.id;
        let title = format!("{} - Animation", animation.asset.name);
        let window = super::create_editor_window(asset_id, &title, wc);
        let mut asset_open = self.asset.open;
        window.open(&mut asset_open).min_size([450.0, 400.0]).default_size([500.0, 400.0]).show(wc.egui.ctx, |ui| {
            // header:
            egui::TopBottomPanel::top(format!("editor_panel_{}_top", asset_id)).show_inside(ui, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("Animation", |ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::Image::new(IMAGES.properties).max_width(14.0).max_height(14.0));
                            if ui.button("Properties...").clicked() {
                                let dlg = self.properties_dialog.get_or_insert_with(|| {
                                    PropertiesDialog::new(animation.sprite_id)
                                });
                                dlg.set_open(animation);
                            }
                        });
                    });
                });
            });

            // footer:
            egui::TopBottomPanel::bottom(format!("editor_panel_{}_bottom", asset_id)).show_inside(ui, |ui| {
                ui.add_space(5.0);
                ui.label(format!("{} bytes", animation.data_size()));
            });

            // loops:
            egui::SidePanel::left(format!("editor_panel_{}_left", asset_id)).resizable(false).max_width(120.0).show_inside(ui, |ui| {
                ui.add_space(5.0);
                egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                    for (loop_index, aloop) in animation.loops.iter().enumerate() {
                        let response = ui.selectable_label(self.selected_loop == loop_index, &aloop.name);
                        if response.clicked() {
                            self.select_loop(loop_index);
                        }
                    }
                });
            });

            // tabs:
            egui::TopBottomPanel::top(format!("editor_panel_{}_tabs", asset_id)).show_inside(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    if ui.selectable_label(matches!(self.selected_tab, EditorTabs::Sprite), "Sprite").clicked() {
                        self.selected_tab = EditorTabs::Sprite;
                    }
                    if ui.selectable_label(matches!(self.selected_tab, EditorTabs::Frames), "Frames").clicked() {
                        self.selected_tab = EditorTabs::Frames;
                    }
                });
            });

            match self.selected_tab {
                EditorTabs::Sprite => self.sprite_tab(ui, wc, animation, sprite_ids, sprites),
                EditorTabs::Frames => self.frames_tab(ui, wc, animation, sprite_ids, sprites),
            };
        });
        self.asset.open = asset_open;
    }
}
