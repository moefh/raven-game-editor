mod tileset;
mod map_data;
mod room;
mod world;
mod sprite;
mod pal_sprite;
mod sprite_animation;
mod sfx;
mod mod_data;
mod font;
mod prop_font;
mod widgets;
mod utils;

pub use utils::{*};
pub use utils::world_grid;

pub use tileset::TilesetEditor;
pub use map_data::MapDataEditor;
pub use room::{RoomEditor, RoomEditorAssetLists};
pub use world::WorldEditor;
pub use sprite::SpriteEditor;
pub use pal_sprite::PalSpriteEditor;
pub use sprite_animation::SpriteAnimationEditor;
pub use sfx::SfxEditor;
pub use mod_data::ModDataEditor;
pub use font::FontEditor;
pub use prop_font::PropFontEditor;

pub use widgets::ColorPickerPopupWidget;

use crate::include_ref_image;
use crate::misc::{
    calc_hash,
    get_asset_type_image,
    IMAGES,
};
use crate::data_asset::{
    DataAssetId,
    GenericAsset,
};
use crate::image::{
    ImageCollection,
};
use crate::app::{
    WindowContext,
    AppWindowTracker,
};

#[derive(Copy, Clone, PartialEq, Eq)]
enum EditorWindowAction {
    None,
    Close,
    ToggleMaximize,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum MaximizedState {
    Maximized,
    UnmaxRequested,
    UnmaxSizeReset,
    Normal,
}

pub struct AssetEditorBase {
    pub id: DataAssetId,
    pub egui_id: egui::Id,
    pub open: bool,
    pub closed_last_frame: bool,
    title: String,
    maximized_state: MaximizedState,
    window_rect: egui::Rect,
    saved_hash: u64,
    cur_hash: u64,
}

impl AssetEditorBase {
    fn new(id: DataAssetId, open: bool) -> Self {
        AssetEditorBase {
            id,
            open,
            title: String::new(),
            closed_last_frame: false,
            egui_id: egui::Id::new(format!("editor_{}", id)),
            maximized_state: MaximizedState::Normal,
            window_rect: egui::Rect::ZERO,
            saved_hash: 0,
            cur_hash: 0,
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.cur_hash != self.saved_hash
    }

    pub fn clear_dirty(&mut self, asset: &impl std::hash::Hash) {
        self.cur_hash = calc_hash(asset);
        self.saved_hash = self.cur_hash;
    }

    pub fn update_dirty(&mut self, asset: &impl std::hash::Hash) {
        self.cur_hash = calc_hash(asset);
    }

    fn calc_image_editor_window_size(image: &impl ImageCollection) -> (egui::Vec2, egui::Vec2) {
        let img_w = image.width() as f32;
        let img_h = image.height() as f32;

        let min_width = (img_w + 320.0).max(500.0);
        let min_height = (img_h + 240.0).max(350.0);
        let min_size = egui::Vec2::new(min_width, min_height);

        let default_width = min_width + img_w * 0.5 + 100.0;
        let default_height = (min_height + img_h * 0.5 + 50.0).max(350.0);
        let default_size = egui::Vec2::new(default_width, default_height);

        (min_size, default_size)
    }

    pub fn toggle_open(&mut self, ctx: &egui::Context) {
        if self.open {
            AppWindowTracker::bring_to_top(self.egui_id, ctx);
        } else {
            self.open = true;
        }
    }

    pub fn close(&mut self) {
        self.open = false;
        self.closed_last_frame = true;
    }

    fn toggle_maximized(&mut self) {
        match self.maximized_state {
            MaximizedState::Maximized => {
                self.maximized_state = MaximizedState::UnmaxRequested;
            }
            MaximizedState::Normal => {
                self.maximized_state = MaximizedState::Maximized;
            }
            _ => {}
        }
    }

    fn title_bg_color(&self, wc: &WindowContext, asset_id: DataAssetId) -> egui::Color32 {
        if wc.is_editor_on_top(asset_id) {
            wc.egui.ctx.global_style().visuals.widgets.open.weak_bg_fill
        } else {
            wc.egui.ctx.global_style().visuals.faint_bg_color
        }
    }

    fn footer_bg_color(&self, wc: &WindowContext, asset_id: DataAssetId) -> egui::Color32 {
        if matches!(self.maximized_state, MaximizedState::Maximized) {
            wc.egui.ctx.global_style().visuals.panel_fill
        } else {
            self.title_bg_color(wc, asset_id)
        }
    }

    fn show_window<Asset>(&mut self, wc: &mut WindowContext, asset: &mut Asset,
                          min_size: impl Into<egui::Vec2>, default_size: impl Into<egui::Vec2>,
                          show_fn: impl FnOnce(&mut egui::Ui, &mut WindowContext, &mut Asset, &mut AssetEditorBase))
    where Asset: GenericAsset {
        self.title.clear();
        self.title.push_str(&asset.asset().name);
        if self.is_dirty() {
            self.title.push_str(" (modified)");
        };

        let maximized_state = self.maximized_state;
        let title = std::mem::take(&mut self.title);
        let mut open = self.open;
        let resp = self.create_window(wc, &mut open, &title, min_size, default_size).show(wc.egui.ctx, |ui| {
            let frame = egui::Frame::new().inner_margin(egui::Margin { left: 5, right: 5, top: 3, bottom: 0 });
            let action = frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(3.0);
                    let title_image = get_asset_type_image(asset.asset().asset_type);
                    ui.add(egui::Image::new(include_ref_image!(title_image)).max_size(egui::Vec2::splat(16.0)).shrink_to_fit());
                    ui.add(egui::Label::new(&title).selectable(false));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let mut action = EditorWindowAction::None;

                        ui.spacing_mut().item_spacing = egui::Vec2::new(3.0, 0.0);
                        if ui.add(egui::Button::image(IMAGES.close).frame_when_inactive(false)).clicked() {
                            action = EditorWindowAction::Close;
                        }
                        let image = match maximized_state {
                            MaximizedState::Maximized => { egui::Image::new(IMAGES.un_maximize) }
                            _ => { egui::Image::new(IMAGES.maximize) }
                        };
                        if ui.add(egui::Button::image(image).frame_when_inactive(false)).clicked() {
                            action = EditorWindowAction::ToggleMaximize;
                        }

                        action
                    }).inner
                }).inner
            }).inner;

            let size = egui::Vec2::new(ui.available_size_before_wrap().x, 1.0);
            let (rect, _response) = ui.allocate_at_least(size, egui::Sense::hover());
            ui.painter().hline(
                rect.left()..=rect.right(),
                rect.bottom() + 2.0,
                ui.style().visuals.window_stroke
            );

            show_fn(ui, wc, asset, self);
            action
        });
        self.open = open;
        self.title = title;

        if let Some(resp) = resp {
            // close window if show() above returned true
            if let Some(action) = resp.inner {
                match action {
                    EditorWindowAction::None => {}
                    EditorWindowAction::Close => { self.close(); }
                    EditorWindowAction::ToggleMaximize => { self.toggle_maximized(); }
                }
            }

            // save window position/size if not maximized
            if matches!(self.maximized_state, MaximizedState::Normal) {
                self.window_rect = resp.response.rect;
            }

            // consume CTRL+UP to maximize/unmaximize
            if wc.is_editor_on_top(self.id) {
                let ctrl_up = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::ArrowUp);
                if resp.response.ctx.input_mut(|i| i.consume_shortcut(&ctrl_up)) {
                    self.toggle_maximized();
                }
            }
        }
    }

    fn create_window<'a>(&mut self, wc: &WindowContext, open: &'a mut bool, title: &str,
                         min_size: impl Into<egui::Vec2>, default_size: impl Into<egui::Vec2>) -> egui::Window<'a> {
        let default_pos = wc.window_space.min + egui::Vec2::splat(10.0);
        let default_rect = egui::Rect {
            min: default_pos,
            max: default_pos + default_size.into(),
        };
        if self.window_rect.max.x == self.window_rect.min.x || self.window_rect.max.y == self.window_rect.min.x {
            self.window_rect = default_rect;
        }

        let mut frame = egui::Frame::window(&wc.egui.ctx.global_style())
            .outer_margin(egui::Margin { left: 0, right: 0, top: 0, bottom: 0 })
            .inner_margin(egui::Margin { left: 0, right: 0, top: 2, bottom: 0 })
            .fill(self.title_bg_color(wc, self.id));
        if self.open && ! matches!(self.maximized_state, MaximizedState::Normal) {
            frame = frame.corner_radius(0.0);
            let (win_rect, constrain_rect) = match self.maximized_state {
                MaximizedState::Maximized => {
                    (wc.window_space, wc.window_space)
                }
                MaximizedState::UnmaxRequested => {
                    self.maximized_state = MaximizedState::UnmaxSizeReset;
                    let rect = self.window_rect;
                    (rect, rect.with_max_x(rect.max.x-2.0).with_max_y(rect.max.y-2.0))
                }
                _ => {
                    self.maximized_state = MaximizedState::Normal;
                    let rect = self.window_rect;
                    (rect, rect.with_max_x(rect.max.x-2.0).with_max_y(rect.max.y-2.0))
                }
            };
            egui::Window::new(title)
                .id(self.egui_id)
                .frame(frame)
                .enabled(! wc.sys_dialogs.has_open_dialog())
                .fixed_rect(win_rect)
                .constrain_to(constrain_rect)
                .collapsible(false)
                .title_bar(false)
                .open(open)
        } else {
            egui::Window::new(title)
                .id(self.egui_id)
                .frame(frame)
                .enabled(! wc.sys_dialogs.has_open_dialog())
                .default_rect(default_rect)
                .min_size(min_size)
                .max_size(wc.window_space.size())
                .constrain_to(wc.window_space)
                .collapsible(false)
                .title_bar(false)
                .open(open)
        }
    }

    pub fn show_dialog_window<T>(wc: &mut WindowContext, id: egui::Id, width: f32, title: &str,
                                 show_fn: impl FnOnce(&mut egui::Ui, &mut WindowContext) -> T) -> egui::ModalResponse<T> {
        let title_bg = wc.egui.ctx.global_style().visuals.widgets.open.weak_bg_fill;
        let frame = egui::Frame::popup(&wc.egui.ctx.global_style()).inner_margin(egui::Margin::ZERO);
        egui::Modal::new(id).frame(frame).show(wc.egui.ctx, |ui| {
            wc.sys_dialogs.block_ui(ui);
            ui.set_width(width);
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                // title bar
                egui::Frame::NONE.fill(title_bg).show(ui, |ui| {
                    ui.add_space(8.0);
                    ui.add(egui::Label::new(egui::RichText::new(title).heading()).selectable(false));

                    let size = egui::Vec2::new(ui.available_size_before_wrap().x, 1.0);
                    let (rect, _response) = ui.allocate_at_least(size, egui::Sense::hover());
                    ui.painter().hline(
                        rect.left()..=rect.right(),
                        rect.bottom(),
                        ui.style().visuals.window_stroke
                    );
                });

                // content
                egui::Frame::NONE.inner_margin(ui.style().spacing.menu_margin).show(ui, |ui| {
                    show_fn(ui, wc)
                }).inner
            }).inner
        })
    }
}
