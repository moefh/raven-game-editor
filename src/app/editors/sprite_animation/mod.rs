mod properties;

use crate::misc::IMAGES;
use crate::data_asset::{
    self,
    SpriteAnimation,
    SpriteAnimationFrame,
    Sprite,
    DataAssetId,
    GenericAsset,
    AssetList,
    AssetIdCollection,
};

use super::{
    get_animation_step,
    IMAGE_ZOOM_OPTIONS,
    ImageZoomOption,
    AssetEditorBase,
    WindowContext,
    SysDialogResponse,
    SysDialogOpenFile,
    SpriteFrameFixer,
    EditorAction,
};
use super::widgets::{
    ImageEditorWidget,
    SpriteFrameListView,
    ImageDisplay,
    ImageDrawingTool,
};
use super::super::{
    menu_item,
};

use properties::PropertiesDialog;

enum EditorTabs {
    Loop,
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
        egui::Panel::bottom(base.footer_panel_id).frame(bottom_frame).show(ui, |ui| {
            let dirty = if base.is_dirty() { " (modified)" } else { "" };
            let num_loops = animation.loops.iter().fold(0, |n, aloop| {
                n + if aloop.frame_indices.is_empty() { 0 } else { 1 }
            });
            ui.add(egui::Label::new(format!("{} bytes [{} loops]{}", animation.data_size(), num_loops, dirty)).truncate());
        });
    }

    pub fn show(
        &mut self,
        wc: &mut WindowContext,
        animation: &mut SpriteAnimation,
        asset_ids: &AssetIdCollection,
        sprites: &mut AssetList<Sprite>
    ) {
        self.dialogs.show(wc, animation, asset_ids, sprites, &mut self.editor);

        self.base.show_window(wc, animation, [640.0, 450.0], [640.0, 450.0], |ui, wc, animation, base| {
            Self::show_footer(ui, wc, animation, base);
            self.editor.show(ui, wc, &mut self.dialogs, animation, asset_ids, sprites);
        });
    }
}

impl SpriteFrameFixer for SpriteAnimationEditor {
    fn move_frame(&mut self, src_index: u32, dest_index: u32) {
        self.editor.image_editor.move_frame_undo_history(src_index, dest_index);
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

    pub fn show(
        &mut self,
        wc: &mut WindowContext,
        animation: &mut SpriteAnimation,
        asset_ids: &AssetIdCollection,
        sprites: &mut AssetList<Sprite>,
        editor: &mut Editor
    ) {
        if let Some(dlg) = &mut self.properties_dialog && dlg.open {
            dlg.show(wc, animation, &asset_ids.sprites, sprites);
        }

        if let Some(sprite) = sprites.get(&animation.sprite_id) && sprite.num_frames as usize != editor.sprite_frames.len() {
            Editor::build_sprite_frames(&mut editor.sprite_frames, sprite.num_frames);
        }
    }
}

struct Editor {
    asset_id: DataAssetId,
    import_sys_dlg_id: String,
    panel_id_menu_bar: egui::Id,
    panel_id_loops: egui::Id,
    panel_id_tab_header: egui::Id,
    panel_id_display_bar: egui::Id,
    panel_id_loop_info_bar: egui::Id,
    panel_id_loop_frames: egui::Id,
    panel_id_loop_properties_grid: egui::Id,
    panel_id_loop_sel_frames: egui::Id,
    panel_id_loop_all_frames: egui::Id,
    combo_salt_zoom: egui::Id,
    selected_tab: EditorTabs,
    selected_loop: usize,
    sprite_frames: Vec<SpriteAnimationFrame>,
    image_editor: ImageEditorWidget<Sprite>,
    main_frames_list: SpriteFrameListView,
    loop_frames_list: SpriteFrameListView,
    all_frames_list: SpriteFrameListView,
    playing: bool,
    reverse_play: bool,
}

impl Editor {
    pub fn new(asset_id: DataAssetId) -> Self {
        Editor {
            asset_id,
            import_sys_dlg_id: format!("editor_{}_import_sprite_animation", asset_id),
            panel_id_menu_bar: egui::Id::new(format!("editor_panel_{}_top", asset_id)),
            panel_id_loops: egui::Id::new(format!("editor_panel_{}_left", asset_id)),
            panel_id_tab_header: egui::Id::new(format!("editor_panel_{}_tabs", asset_id)),
            panel_id_display_bar: egui::Id::new(format!("editor_panel_{}_display_toolbar", asset_id)),
            panel_id_loop_info_bar: egui::Id::new(format!("editor_panel_{}_loop_info_bar", asset_id)),
            panel_id_loop_frames: egui::Id::new(format!("editor_panel_{}_loop_frames", asset_id)),
            panel_id_loop_properties_grid: egui::Id::new(format!("editor_panel_{}_prop_grid", asset_id)),
            panel_id_loop_sel_frames: egui::Id::new(format!("editor_panel_{}_loop_sel_frames", asset_id)),
            panel_id_loop_all_frames: egui::Id::new(format!("editor_panel_{}_loop_all_frames", asset_id)),
            combo_salt_zoom: egui::Id::new(format!("pal_sprite_editor_{}_zoom_combo", asset_id)),

            selected_tab: EditorTabs::Loop,
            selected_loop: 0,
            sprite_frames: Vec::new(),
            image_editor: ImageEditorWidget::<Sprite>::new()
                .with_image_display(ImageDisplay::new(ImageDisplay::TRANSPARENT | ImageDisplay::GRID))
                .with_tool(ImageDrawingTool::Collision)
                .readonly(),
            main_frames_list: SpriteFrameListView::new(true, false),
            loop_frames_list: SpriteFrameListView::new(false, false),
            all_frames_list: SpriteFrameListView::new(true, true),
            playing: false,
            reverse_play: false,
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
        self.main_frames_list.selected_frame = 0;
        self.all_frames_list.selected_frame = 0;
    }

    fn loop_tab(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        animation: &mut SpriteAnimation,
        _asset_ids: &AssetIdCollection,
        sprites: &mut AssetList<Sprite>
    ) {
        let sprite = match sprites.get_mut(&animation.sprite_id) {
            Some(s) => s,
            None => { return; }
        };

        // Loop name
        egui::Panel::top(self.panel_id_loop_info_bar).resizable(false).show(ui, |ui| {
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.add_space(2.0);
                if let Some(aloop) = animation.loops.get_mut(self.selected_loop) {
                    ui.label("Name:");
                    ui.add(egui::TextEdit::singleline(&mut aloop.name_id).desired_width(200.0));
                }
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.add_space(2.0);
                ui.spacing_mut().item_spacing = egui::Vec2::new(0.0, 0.0);

                if let Some(aloop) = animation.loops.get_mut(self.selected_loop) {
                    ui.label("Frame speed: ");
                    ui.add(egui::DragValue::new(&mut aloop.frame_speed).speed(1.0).range(1..=256));

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    ui.label("Don't loop: ");
                    ui.checkbox(&mut aloop.dont_loop, "");
                }
            });

            ui.add_space(4.0);

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
            });
            ui.add_space(4.0);
        });

        // Display
        egui::Panel::top(self.panel_id_display_bar).resizable(false).show(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.add_space(2.0);
                let spacing = ui.spacing().item_spacing;
                ui.spacing_mut().item_spacing = egui::Vec2::new(1.0, 0.0);

                // playback
                let can_play = animation.loops.get(self.selected_loop).is_some_and(|aloop| aloop.frame_indices.len() > 1);
                if ui.add_enabled(
                    can_play && (! self.playing || ! self.reverse_play),
                    egui::Button::new("\u{23f4}")
                ).on_hover_text("Play reversed").clicked() {
                    self.playing = true;
                    self.reverse_play = true;
                }
                ui.add_space(6.0);
                if ui.add_enabled(
                    self.playing,
                    egui::Button::new("\u{23f8}")
                ).on_hover_text("Pause").clicked() {
                    self.playing = false;
                }
                ui.add_space(6.0);
                if ui.add_enabled(
                    can_play && (! self.playing || self.reverse_play),
                    egui::Button::new("\u{23f5}")
                ).on_hover_text("Play").clicked() {
                    self.playing = true;
                    self.reverse_play = false;
                }

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                if let Some(aloop) = animation.loops.get(self.selected_loop) &&
                    let Some(frame) = aloop.frame_indices.get(self.main_frames_list.selected_frame) &&
                    let Some(sprite_frame_index) = frame.head_index {
                        ui.label(format!(
                            "Frame {} [sprite {}]",
                            self.main_frames_list.selected_frame,
                            sprite_frame_index
                        ));
                    }

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                // display
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
                        egui::ComboBox::from_id_salt(self.combo_salt_zoom)
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

        // loop frames:
        egui::Panel::bottom(self.panel_id_loop_frames).show(ui, |ui| {
            ui.add_space(8.0);
            if let Some(aloop) = animation.loops.get(self.selected_loop) {
                self.main_frames_list.show(
                    ui,
                    wc,
                    &aloop.frame_indices,
                    animation.foot_overlap,
                    sprite,
                    self.image_editor.display.is_transparent(),
                );
            }
        });

        // body:
        egui::CentralPanel::default().show(ui, |ui| {
            if let Some(aloop) = animation.loops.get(self.selected_loop) {
                let loop_len = aloop.frame_indices.len();
                let frame_index = if self.playing && loop_len != 0 {
                    wc.request_animation_repaint();
                    if self.reverse_play {
                        loop_len - 1 - get_animation_step(wc) as usize % loop_len
                    } else {
                        get_animation_step(wc) as usize % loop_len
                    }
                } else {
                    self.main_frames_list.selected_frame
                };
                let sprite_index = aloop.frame_indices.get(frame_index).and_then(|frame| frame.head_index).unwrap_or(0);
                self.image_editor.set_selected_image(sprite_index as u32, sprite);
                self.image_editor.set_collision_rect(Some(animation.clip_rect));
                let colors = (0xff, 0xff);
                self.image_editor.show(ui, wc, sprite, colors);
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
        _asset_ids: &AssetIdCollection,
        sprites: &mut AssetList<Sprite>
    ) {
        let sprite = match sprites.get_mut(&animation.sprite_id) {
            Some(s) => s,
            None => { return; }
        };

        egui::Panel::top(self.panel_id_loop_sel_frames).show(ui, |ui| {
            ui.add_space(5.0);
            if let Some(aloop) = animation.loops.get_mut(self.selected_loop) {
                egui::Grid::new(self.panel_id_loop_properties_grid)
                    .num_columns(2)
                    .spacing([4.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("Loop length:");
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
                self.loop_frames_list.show(
                    ui,
                    wc,
                    &aloop.frame_indices,
                    animation.foot_overlap,
                    sprite,
                    self.image_editor.display.is_transparent()
                );
            }
        });

        egui::Panel::top(self.panel_id_loop_all_frames).show(ui, |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.label("Sprite frames (drag to the lists below):");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if let Some(hovered_frame) = self.all_frames_list.hovered_frame {
                        ui.label(format!("Frame {}", hovered_frame));
                    } else {
                        ui.label(format!("Frame {}", self.all_frames_list.selected_frame));
                    }
                    ui.separator();
                });
            });
            let scroll = self.all_frames_list.show(
                ui,
                wc,
                &self.sprite_frames,
                animation.foot_overlap,
                sprite,
                self.image_editor.display.is_transparent()
            );
            if let Some(pointer_pos) = scroll.inner.interact_pointer_pos() &&
                scroll.inner_rect.contains(pointer_pos) &&
                scroll.inner.drag_started() {
                    scroll.inner.dnd_set_drag_payload(FrameDragPayload::new(self.all_frames_list.selected_frame));
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
                        for (frame_num, frame) in aloop.frame_indices.iter_mut().enumerate() {
                            let (_, dropped_payload) = ui.dnd_drop_zone::<FrameDragPayload, ()>(drop_frame, |ui| {
                                let name = match frame.head_index {
                                    Some(sprite_index) => &format!("[{:02}]: {}", frame_num, sprite_index),
                                    None => &format!("[{:02}]: (empty)", frame_num),
                                };
                                let label = ui.add(egui::Label::new(name).selectable(false).sense(egui::Sense::click()));
                                egui::Popup::context_menu(&label).show(|ui| {
                                    if ui.add(egui::Button::new("Remove")).clicked() {
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
                        for (frame_num, frame) in aloop.frame_indices.iter_mut().enumerate() {
                            let (_, dropped_payload) = ui.dnd_drop_zone::<FrameDragPayload, ()>(drop_frame, |ui| {
                                let name = match frame.foot_index {
                                    Some(sprite_index) => &format!("[{:02}]: {}", frame_num, sprite_index),
                                    None => &format!("[{:02}]: (empty)", frame_num),
                                };
                                let label = ui.add(egui::Label::new(name).selectable(false).sense(egui::Sense::click()));
                                egui::Popup::context_menu(&label).show(|ui| {
                                    if ui.add(egui::Button::new("Remove")).clicked() {
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

    fn import_sprite_animation(
        &mut self,
        wc: &mut WindowContext,
        file: SysDialogOpenFile,
        animation: &mut SpriteAnimation,
        asset_ids: &AssetIdCollection
    ) {
        let result = file.read_string().and_then(|content| {
            data_asset::deserialize_sprite_animation(&content, self.asset_id, asset_ids, wc.logger)
        });
        match result {
            Ok(mut new_animation) => {
                std::mem::swap(animation, &mut new_animation)
            }

            Err(e) => {
                wc.logger.log(format!("ERROR reading sprite animation file from {}:", file.filename()));
                wc.logger.log(format!("{}", e));
                wc.open_message_box(
                    "Error importing Sprite Animation",
                    "Error importing sprite animation file.\n\nConsult the log window for more information."
                );
            }
        }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        dialogs: &mut Dialogs,
        animation: &mut SpriteAnimation,
        asset_ids: &AssetIdCollection,
        sprites: &mut AssetList<Sprite>
    ) {
        if let Some(SysDialogResponse::File(file)) = wc.sys_dialogs.get_response_for(&self.import_sys_dlg_id) {
            self.import_sprite_animation(wc, file, animation, asset_ids);
        }
        if sprites.get(&animation.sprite_id).is_none() {
            return;  // animation has an invalid sprite id
        }

        // header:
        egui::Panel::top(self.panel_id_menu_bar).show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Animation", |ui| {
                    if ui.add(menu_item(IMAGES.import, " Import...")).clicked() {
                        wc.sys_dialogs.open_file(
                            Some(wc.egui.window),
                            self.import_sys_dlg_id.clone(),
                            "sprite_animation",
                            "Import Sprite Animation file",
                            &[
                                ("Raven Map files (*.ravanim)", &["ravanim"]),
                                ("All files (*.*)", &["*"]),
                            ]
                        );
                    }

                    if ui.add(menu_item(IMAGES.export, " Export...")).clicked() {
                        wc.add_editor_action(EditorAction::ExportSpriteAnimation { animation_id: self.asset_id });
                    }

                    ui.separator();

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
        egui::Panel::left(self.panel_id_loops).resizable(false).max_size(120.0).show(ui, |ui| {
            ui.add_space(5.0);
            egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                for (loop_index, aloop) in animation.loops.iter().enumerate() {
                    let selected = self.selected_loop == loop_index;
                    let button = egui::Button::selectable(selected, &aloop.name_id).wrap_mode(egui::TextWrapMode::Truncate);
                    if ui.add(button).clicked() {
                        self.select_loop(loop_index);
                    }
                }
            });
        });

        // tabs:
        egui::Panel::top(self.panel_id_tab_header).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                if ui.selectable_label(matches!(self.selected_tab, EditorTabs::Loop), "Loop").clicked() {
                    self.selected_tab = EditorTabs::Loop;
                }
                if ui.selectable_label(matches!(self.selected_tab, EditorTabs::Frames), "Frames").clicked() {
                    self.selected_tab = EditorTabs::Frames;
                }
            });
        });

        match self.selected_tab {
            EditorTabs::Loop => self.loop_tab(ui, wc, animation, asset_ids, sprites),
            EditorTabs::Frames => self.frames_tab(ui, wc, animation, asset_ids, sprites),
        };
    }
}
