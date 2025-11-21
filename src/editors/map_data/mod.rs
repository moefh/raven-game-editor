mod properties;

use crate::IMAGES;
use crate::app::WindowContext;
use crate::misc::{ImageCollection, TextureSlot};
use crate::data_asset::{MapData, Tileset, AssetIdList, AssetList, DataAssetId, GenericAsset};

use properties::PropertiesDialog;

pub struct MapDataEditor {
    pub asset: super::DataAssetEditor,
    properties_dialog: Option<PropertiesDialog>,
    state: super::widgets::MapEditorState,
}

fn calc_map_editor_window_size() -> (egui::Vec2, egui::Vec2) {
    let min_size = egui::Vec2::new(130.0 + 160.0, 80.0 + 100.0);
    let default_size = egui::Vec2::new(130.0 + 500.0, 80.0 + 300.0);
    (min_size, default_size)
}

impl MapDataEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        MapDataEditor {
            asset: super::DataAssetEditor::new(id, open),
            properties_dialog: None,
            state: super::widgets::MapEditorState::new(),
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, map_data: &mut MapData, tileset_ids: &AssetIdList, tilesets: &AssetList<Tileset>) {
        if let Some(dlg) = &mut self.properties_dialog && dlg.open {
            dlg.show(wc, map_data, tileset_ids, tilesets);
        }

        let asset_id = map_data.asset.id;
        let title = format!("{} - Map", map_data.asset.name);
        let window = super::create_editor_window(asset_id, &title, wc);
        let (min_size, default_size) = calc_map_editor_window_size();
        window.min_size(min_size).default_size(default_size).open(&mut self.asset.open).show(wc.egui.ctx, |ui| {
            // header:
            egui::TopBottomPanel::top(format!("editor_panel_{}_top", asset_id)).show_inside(ui, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("Map", |ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::Image::new(IMAGES.properties).max_width(14.0).max_height(14.0));
                            if ui.button("Properties...").clicked() {
                                let dlg = self.properties_dialog.get_or_insert_with(|| {
                                    PropertiesDialog::new(map_data.tileset_id)
                                });
                                dlg.set_open(map_data, self.state.right_draw_tile as u8);
                            }
                        });
                    });
                });
            });

            // footer:
            egui::TopBottomPanel::bottom(format!("editor_panel_{}_bottom", asset_id)).show_inside(ui, |ui| {
                ui.add_space(5.0);
                ui.label(format!("{} bytes", map_data.data_size()));
            });

            if let Some(tileset) = tilesets.get(&map_data.tileset_id) {
                let (image, texture) = ImageCollection::get_asset_texture(tileset, wc.tex_man, wc.egui.ctx, TextureSlot::Transparent);

                // tile picker:
                egui::SidePanel::left(format!("editor_panel_{}_left", asset_id)).resizable(false).show_inside(ui, |ui| {
                    ui.add_space(5.0);
                    let picker_zoom = 4.0;
                    let scroll = super::widgets::image_item_picker(ui, texture, &image, self.state.left_draw_tile, picker_zoom);
                    if let Some(pointer_pos) = scroll.inner.interact_pointer_pos() {
                        let pos = pointer_pos - scroll.inner_rect.min + scroll.state.offset;
                        if pos.x >= 0.0 && pos.x <= scroll.inner_rect.width() {
                            let frame_size = picker_zoom * image.get_item_size();
                            let sel_tile = u32::min((pos.y / frame_size.y).floor() as u32, image.num_items-1);
                            if scroll.inner.dragged_by(egui::PointerButton::Primary) {
                                self.state.left_draw_tile = sel_tile;
                            } else if scroll.inner.dragged_by(egui::PointerButton::Secondary) {
                                self.state.right_draw_tile = sel_tile;
                            }
                        }
                    };
                });

                // body:
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    super::widgets::map_editor(ui, wc, map_data, tileset, &image, &mut self.state);
                });
            }
        });
    }
}
