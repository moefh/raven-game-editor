mod properties;

use crate::misc::IMAGES;
use crate::app::{
    menu_item,
    menu_item_no_image,
    WindowContext,
};
use crate::image::{
    colors,
    ImageCollection,
};
use crate::data_asset::{
    SpriteAnimation,
    SpriteAnimationFrame,
    Sprite,
    DataAssetId,
    GenericAsset,
    AssetList,
    AssetIdList,
};

use properties::PropertiesDialog;
use super::{
    IMAGE_ZOOM_OPTIONS,
    ImageZoomOption,
    AssetEditorBase,
};
use super::widgets::{
    ColorPickerWidget,
    ImageEditorWidget,
    SpriteFrameListView,
    ImageDisplay,
    ImageDrawingTool,
};

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
    pub base: AssetEditorBase,
    editor: Editor,
    dialogs: Dialogs,
}

impl SpriteAnimationEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        SpriteAnimationEditor {
            base: AssetEditorBase::new(id, open),
            editor: Editor::new(id),
            dialogs: Dialogs::new(),
        }
    }

    pub fn prepare_for_saving(&mut self, animation: &mut SpriteAnimation, sprites: &mut AssetList<Sprite>) {
        if let Some(sprite) = sprites.get_mut(&animation.sprite_id) {
            self.editor.image_editor.drop_selection(sprite);
        }
    }

    fn show_footer(ui: &mut egui::Ui, wc: &WindowContext, animation: &SpriteAnimation, base: &AssetEditorBase) {
        let margin = egui::Margin { left: 5, right: 5, top: 4, bottom: 0 };
        let bottom_frame = egui::Frame::NONE.inner_margin(margin).fill(base.footer_bg_color(wc, animation.asset.id));
        egui::Panel::bottom(format!("editor_panel_{}_bottom", animation.asset.id)).frame(bottom_frame).show(ui, |ui| {
            let dirty = if base.is_dirty() { " (modified)" } else { "" };
            let num_loops = animation.loops.iter().fold(0, |n, aloop| {
                n + if aloop.frame_indices.is_empty() { 0 } else { 1 }
            });
            ui.label(format!("{} bytes [{} loops]{}", animation.data_size(), num_loops, dirty));
        });
    }

    pub fn show(&mut self, wc: &mut WindowContext, animation: &mut SpriteAnimation, sprite_ids: &AssetIdList, sprites: &mut AssetList<Sprite>) {
        self.dialogs.show(wc, animation, sprite_ids, sprites, &mut self.editor);

        self.base.show_window(wc, animation, [500.0, 400.0], [500.0, 400.0], |ui, wc, animation, base| {
            Self::show_footer(ui, wc, animation, base);
            self.editor.show(ui, wc, &mut self.dialogs, animation, sprite_ids, sprites);
        });
    }
}

struct Dialogs {
    properties_dialog: Option<PropertiesDialog>,
}

impl Dialogs {
    fn new() -> Self {
        Dialogs {
            properties_dialog: None,
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, animation: &mut SpriteAnimation,
                sprite_ids: &AssetIdList, sprites: &mut AssetList<Sprite>, editor: &mut Editor) {
        if let Some(dlg) = &mut self.properties_dialog && dlg.open {
            dlg.show(wc, animation, sprite_ids, sprites);
        }

        if let Some(sprite) = sprites.get(&animation.sprite_id) && sprite.num_frames as usize != editor.sprite_frames.len() {
            Editor::build_sprite_frames(&mut editor.sprite_frames, sprite.num_frames);
        }
    }
}

struct Editor {
    asset_id: DataAssetId,
    selected_tab: EditorTabs,
    selected_loop: usize,
    selected_loop_frame: usize,
    sprite_frames: Vec<SpriteAnimationFrame>,
    selected_sprite_frame: usize,
    color_picker: ColorPickerWidget,
    image_editor: ImageEditorWidget<Sprite>,
}

impl Editor {
    pub fn new(asset_id: DataAssetId) -> Self {
        Editor {
            asset_id,
            selected_tab: EditorTabs::Sprite,
            selected_loop: 0,
            selected_loop_frame: 0,
            sprite_frames: Vec::new(),
            selected_sprite_frame: 0,
            color_picker: ColorPickerWidget::new(format!("editor_{}_color_picker", asset_id), colors::RED, colors::GREEN, false),
            image_editor: ImageEditorWidget::<Sprite>::new().with_image_display(
                ImageDisplay::new(ImageDisplay::TRANSPARENT | ImageDisplay::GRID | ImageDisplay::COLLISION)),
        }
    }

    fn build_sprite_frames(frames: &mut Vec<SpriteAnimationFrame>, num_frames: u32) {
        frames.clear();
        for index in 0..num_frames as u8 {
            frames.push(SpriteAnimationFrame { head_index: Some(index), foot_index: None });
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

        // toolbar:
        egui::Panel::top(format!("editor_panel_{}_toolbar", asset_id)).resizable(false).show(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.add_space(2.0);
                let spacing = ui.spacing().item_spacing;
                ui.spacing_mut().item_spacing = egui::Vec2::new(1.0, 0.0);

                ui.label("Tool:");
                ui.add_space(1.0);
                if ui.add(egui::Button::image(IMAGES.pen)
                          .selected(self.image_editor.get_tool() == ImageDrawingTool::Pencil)
                          .frame_when_inactive(self.image_editor.get_tool() == ImageDrawingTool::Pencil)).on_hover_text("Pencil").clicked() {
                    self.image_editor.set_tool(ImageDrawingTool::Pencil);
                }
                if ui.add(egui::Button::image(IMAGES.select)
                          .selected(self.image_editor.get_tool() == ImageDrawingTool::Collision)
                          .frame_when_inactive(self.image_editor.get_tool() ==
                                               ImageDrawingTool::Collision)).on_hover_text("Collision").clicked() {
                    self.image_editor.set_tool(ImageDrawingTool::Collision);
                }

                ui.spacing_mut().item_spacing = spacing;
                ui.with_layout(egui::Layout::default().with_cross_align(egui::Align::RIGHT), |ui| {
                    ui.horizontal(|ui| {
                        let spacing = ui.spacing().item_spacing;
                        ui.spacing_mut().item_spacing = egui::Vec2::new(1.0, 0.0);
                        if ui.add(egui::Button::image(IMAGES.grid)
                                  .selected(self.image_editor.display.has_bits(ImageDisplay::GRID))
                                  .frame_when_inactive(self.image_editor.display.has_bits(ImageDisplay::GRID)))
                            .on_hover_text("Grid").clicked() {
                                self.image_editor.toggle_display(ImageDisplay::GRID);
                            }
                        if ui.add(egui::Button::image(IMAGES.transparency)
                                  .selected(self.image_editor.display.is_transparent())
                                  .frame_when_inactive(self.image_editor.display.is_transparent()))
                            .on_hover_text("Transparency").clicked() {
                                self.image_editor.toggle_display(ImageDisplay::TRANSPARENT);
                            }
                        if ui.add(egui::Button::image(IMAGES.select)
                                  .selected(self.image_editor.display.has_bits(ImageDisplay::COLLISION))
                                  .frame_when_inactive(self.image_editor.display.has_bits(ImageDisplay::COLLISION)))
                            .on_hover_text("Collision").clicked() {
                                self.image_editor.toggle_display(ImageDisplay::COLLISION);
                            }
                        ui.add_space(1.0);
                        ui.label("Display:");

                        ui.add_space(5.0);
                        ui.separator();
                        ui.add_space(5.0);

                        let mut cur_zoom_option = ImageZoomOption::from_image_editor_zoom(self.image_editor.zoom);
                        egui::ComboBox::from_id_salt(format!("pal_sprite_editor_{}_zoom_combo", self.asset_id))
                            .selected_text(cur_zoom_option.name())
                            .width(60.0)
                            .show_ui(ui, |ui| {
                                for option in IMAGE_ZOOM_OPTIONS {
                                    if option.is_custom() && ! cur_zoom_option.is_custom() { continue; }
                                    ui.selectable_value(&mut cur_zoom_option, option, option.name());
                                }
                            });
                        self.image_editor.zoom = cur_zoom_option.image_editor_zoom(self.image_editor.zoom);
                        ui.add_space(1.0);
                        ui.label("Zoom:");

                        ui.spacing_mut().item_spacing = spacing;
                    });
                });
            });
            ui.add_space(0.0);  // don't remove this, it's necessary
        });

        // Collision
        egui::Panel::top(format!("editor_panel_{}_collision_bar", asset_id)).resizable(false).show(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.add_space(2.0);
                ui.spacing_mut().item_spacing = egui::Vec2::new(0.0, 0.0);

                ui.label("Collision: (");
                let max_x = (animation.clip_rect.x + animation.clip_rect.w).max(0);
                let max_y = (animation.clip_rect.y + animation.clip_rect.h).max(0);
                let mut x = animation.clip_rect.x;
                let mut y = animation.clip_rect.y;
                ui.add(egui::DragValue::new(&mut x).speed(1.0).range(0..=max_x));
                ui.label(",");
                ui.add(egui::DragValue::new(&mut y).speed(1.0).range(0..=max_y));
                ui.label(")");

                // fix size
                let dx = x - animation.clip_rect.x;
                let dy = y - animation.clip_rect.y;
                animation.clip_rect.x += dx;
                animation.clip_rect.y += dy;
                animation.clip_rect.w -= dx;
                animation.clip_rect.h -= dy;
                if animation.clip_rect.w < 0 { animation.clip_rect.x += animation.clip_rect.w; animation.clip_rect.w = 0; }
                if animation.clip_rect.h < 0 { animation.clip_rect.y += animation.clip_rect.h; animation.clip_rect.h = 0; }

                ui.add_space(10.0);

                let max_w = sprite.width.saturating_sub(animation.clip_rect.x.max(0) as u32);
                let max_h = sprite.height.saturating_sub(animation.clip_rect.y.max(0) as u32);
                ui.add(egui::DragValue::new(&mut animation.clip_rect.w).speed(1.0).range(0..=max_w));
                ui.label("x");
                ui.add(egui::DragValue::new(&mut animation.clip_rect.h).speed(1.0).range(0..=max_h));

                if let Some(image_item) = animation.loops.get(self.selected_loop)
                    .and_then(|aloop| aloop.frame_indices.get(self.selected_loop_frame))
                    .and_then(|frame| frame.head_index) {
                        ui.with_layout(egui::Layout::default().with_cross_align(egui::Align::RIGHT), |ui| {
                            ui.horizontal(|ui| {
                                ui.label(format!("Frame {}", image_item));
                                ui.add_space(5.0);
                                ui.separator();
                            });
                            ui.add_space(0.0);  // don't remove this, it's necessary
                        });
                    }
            });
            ui.add_space(0.0);  // don't remove this, it's necessary
        });

        // color picker:
        egui::Panel::right(format!("editor_panel_{}_right", asset_id)).resizable(false).show(ui, |ui| {
            ui.add_space(5.0);
            self.color_picker.show(ui, wc);
        });

        // loop frames:
        egui::Panel::bottom(format!("editor_panel_{}_loop_frames", asset_id)).show(ui, |ui| {
            ui.add_space(8.0);
            if let Some(aloop) = animation.loops.get(self.selected_loop) {
                let view = SpriteFrameListView::new(&aloop.frame_indices, animation.foot_overlap, self.selected_loop_frame);
                let scroll = view.show(ui, wc, sprite, self.image_editor.display.is_transparent());
                let num_frames = aloop.frame_indices.len();
                if num_frames != 0 &&
                    let Some(pointer_pos) = scroll.inner.interact_pointer_pos() &&
                    scroll.inner_rect.contains(pointer_pos) {
                        let pos = pointer_pos - scroll.inner_rect.min + scroll.state.offset;
                        let frame_size = sprite.get_item_size();
                        self.selected_loop_frame = usize::min((pos.x / frame_size.x).floor() as usize, num_frames - 1);
                    }
            }
        });

        // body:
        egui::CentralPanel::default().show(ui, |ui| {
            if let Some(image_item) = animation.loops.get(self.selected_loop)
                .and_then(|aloop| aloop.frame_indices.get(self.selected_loop_frame))
                .and_then(|frame| frame.head_index)  {
                    self.image_editor.set_selected_image(image_item as u32, sprite);
                    self.image_editor.set_collision_rect(Some(animation.clip_rect));
                    let colors = (self.color_picker.state.left_color, self.color_picker.state.right_color);
                    self.image_editor.show(ui, wc, sprite, colors);
                    self.color_picker.maybe_set_colors(
                        self.image_editor.pick_left_color.take(),
                        self.image_editor.pick_right_color.take()
                    );
                    if let Some(rect) = self.image_editor.get_collision_rect() {
                        animation.clip_rect = rect;
                    }
                }
        });
    }

    fn frames_tab(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        animation: &mut SpriteAnimation,
        _sprite_ids: &AssetIdList,
        sprites: &mut AssetList<Sprite>
    ) {
        let sprite = match sprites.get_mut(&animation.sprite_id) {
            Some(s) => s,
            None => { return; }
        };

        let asset_id = animation.asset.id;

        egui::Panel::top(format!("editor_panel_{}_loop_sel_frames", asset_id)).show(ui, |ui| {
            ui.add_space(5.0);
            if let Some(aloop) = animation.loops.get_mut(self.selected_loop) {
                egui::Grid::new(format!("editor_panel_{}_prop_grid", animation.asset.id))
                    .num_columns(2)
                    .spacing([4.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut aloop.name_id);
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
                let view = SpriteFrameListView::new(&aloop.frame_indices, animation.foot_overlap, aloop.frame_indices.len() + 1);
                view.show(ui, wc, sprite, self.image_editor.display.is_transparent());
            }
        });

        egui::Panel::top(format!("editor_panel_{}_loop_all_frames", asset_id)).show(ui, |ui| {
            ui.add_space(5.0);
            ui.label("Sprite frames (drag to the lists below):");
            let view = SpriteFrameListView::new(&self.sprite_frames, 0, self.selected_sprite_frame);
            let scroll = view.show(ui, wc, sprite, self.image_editor.display.is_transparent());
            let num_frames = self.sprite_frames.len();
            if num_frames != 0 &&
                let Some(pointer_pos) = scroll.inner.interact_pointer_pos() &&
                scroll.inner_rect.contains(pointer_pos) &&
                scroll.inner.drag_started() {
                    let pos = pointer_pos - scroll.inner_rect.min + scroll.state.offset;
                    let frame_size = sprite.get_item_size();
                    self.selected_sprite_frame = usize::min((pos.x / frame_size.x).floor() as usize, num_frames - 1);
                    scroll.inner.dnd_set_drag_payload(FrameDragPayload::new(self.selected_sprite_frame));
                }
        });

        egui::CentralPanel::default().show(ui, |ui| {
            let drop_frame = egui::Frame::default().inner_margin(2.0);
            if let Some(aloop) = animation.loops.get_mut(self.selected_loop) {
                ui.take_available_space();
                ui.columns_const(|[head_ui, foot_ui]| {
                    head_ui.label("Head frames:");
                    head_ui.take_available_space();
                    egui::ScrollArea::vertical().id_salt("head").auto_shrink([false, false]).show(head_ui, |ui| {
                        for frame in &mut aloop.frame_indices {
                            let (_, dropped_payload) = ui.dnd_drop_zone::<FrameDragPayload, ()>(drop_frame, |ui| {
                                let name = match frame.head_index {
                                    Some(index) => &format!("{}", index),
                                    None => "(empty)",
                                };
                                let label = ui.add(egui::Label::new(name).selectable(false).sense(egui::Sense::click()));
                                egui::Popup::context_menu(&label).show(|ui| {
                                    if ui.add(menu_item_no_image(" Remove")).clicked() {
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
                    egui::ScrollArea::vertical().id_salt("foot").auto_shrink([false, false]).show(foot_ui, |ui| {
                        for frame in &mut aloop.frame_indices {
                            let (_, dropped_payload) = ui.dnd_drop_zone::<FrameDragPayload, ()>(drop_frame, |ui| {
                                let name = match frame.foot_index {
                                    Some(index) => &format!("{}", index),
                                    None => "(empty)",
                                };
                                let label = ui.add(egui::Label::new(name).selectable(false).sense(egui::Sense::click()));
                                egui::Popup::context_menu(&label).show(|ui| {
                                    if ui.add(menu_item_no_image(" Remove")).clicked() {
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

    pub fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, dialogs: &mut Dialogs,
                animation: &mut SpriteAnimation, sprite_ids: &AssetIdList, sprites: &mut AssetList<Sprite>) {
        if sprites.get(&animation.sprite_id).is_none() {
            return;  // animation has an invalid sprite id
        }

        // header:
        egui::Panel::top(format!("editor_panel_{}_top", self.asset_id)).show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Animation", |ui| {
                    if ui.add(menu_item(IMAGES.properties, " Properties...")).clicked() {
                        let dlg = dialogs.properties_dialog.get_or_insert_with(|| {
                            PropertiesDialog::new(animation.sprite_id)
                        });
                        dlg.set_open(wc, animation);
                    }
                });
            });
        });

        // loops:
        egui::Panel::left(format!("editor_panel_{}_left", self.asset_id)).resizable(false).max_size(120.0).show(ui, |ui| {
            ui.add_space(5.0);
            egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                for (loop_index, aloop) in animation.loops.iter().enumerate() {
                    let response = ui.selectable_label(self.selected_loop == loop_index, &aloop.name_id);
                    if response.clicked() {
                        self.select_loop(loop_index);
                    }
                }
            });
        });

        // tabs:
        egui::Panel::top(format!("editor_panel_{}_tabs", self.asset_id)).show(ui, |ui| {
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

        // keyboard:
        if wc.is_editor_on_top(self.asset_id) && let Some(sprite) = sprites.get_mut(&animation.sprite_id) {
            self.image_editor.handle_keyboard(ui, wc, sprite, self.color_picker.state.right_color);
        }
    }
}
