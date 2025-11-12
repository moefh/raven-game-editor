use std::collections::HashSet;
use crate::IMAGES;
use crate::misc::WindowContext;
use crate::data_asset::{
    Room, RoomMap, RoomEntity, RoomTrigger, RoomItem,
    MapData, Tileset, SpriteAnimation, Sprite,
    DataAssetId, AssetIdCollection, GenericAsset, AssetIdList, AssetList,
};

pub struct RoomEditorAssetLists<'a> {
    pub maps: &'a AssetList<MapData>,
    pub tilesets: &'a AssetList<Tileset>,
    pub animations: &'a AssetList<SpriteAnimation>,
    pub sprites: &'a AssetList<Sprite>,
}

impl<'a> RoomEditorAssetLists<'a> {
    pub fn new(maps: &'a AssetList<MapData>, tilesets: &'a AssetList<Tileset>,
               animations: &'a AssetList<SpriteAnimation>, sprites: &'a AssetList<Sprite>) -> Self {
        RoomEditorAssetLists {
            maps,
            tilesets,
            animations,
            sprites,
        }
    }
}

#[derive(Clone, Copy)]
pub enum RoomItemRef {
    None,
    Map(usize),
    Entity(usize),
    Trigger(usize),
}

fn edit_prop_i16(ui: &mut egui::Ui, value: &mut i16, min: i16, max: i16) {
    let min = min as f32;
    let max = max as f32;
    let mut value_f = *value as f32;
    ui.add(egui::DragValue::new(&mut value_f).speed(1.0).range(min..=max));
    *value = value_f.clamp(min, max) as i16;
}

fn edit_prop_u16(ui: &mut egui::Ui, value: &mut u16, min: u16, max: u16) {
    let min = min as f32;
    let max = max as f32;
    let mut value_f = *value as f32;
    ui.add(egui::DragValue::new(&mut value_f).speed(1.0).range(min..=max));
    *value = value_f.clamp(min, max) as u16;
}

pub struct RoomEditor {
    pub asset: super::DataAssetEditor,
    room_editor_state: super::widgets::RoomEditorState,
    properties_dialog: PropertiesDialog,
    map_selection_dialog: MapSelectionDialog,
}

impl RoomEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        RoomEditor {
            asset: super::DataAssetEditor::new(id, open),
            room_editor_state: super::widgets::RoomEditorState::new(),
            properties_dialog: PropertiesDialog::new(),
            map_selection_dialog: MapSelectionDialog::new(),
        }
    }

    fn get_new_item_name(items: &[impl RoomItem], base: &str) -> String {
        use std::fmt::Write;

        let mut num: usize = 1;
        let mut new_name = String::new();
        loop {
            new_name.clear();
            new_name.push_str(base);
            if write!(new_name, "{}", num).is_err() { break; }
            if ! items.iter().any(|e| e.name() == new_name) { break; }
            num += 1;
        }
        new_name
    }

    fn add_entity(&mut self, wc: &mut WindowContext, room: &mut Room, animations: &AssetIdList) {
        let animation_id = match animations.get_first() {
            Some(v) => v,
            None => {
                wc.dialogs.open_message_box("No Animation Available", "You must create a sprite animation first!");
                return;
            },
        };

        self.room_editor_state.selected_item = RoomItemRef::Entity(room.entities.len());
        let name = Self::get_new_item_name(&room.entities, "entity");
        room.entities.push(RoomEntity {
            name,
            x: 0,
            y: 0,
            animation_id,
            data0: 0,
            data1: 0,
            data2: 0,
            data3: 0,
        });
    }

    fn remove_entity(&mut self, room: &mut Room, ent_index: usize) {
        if ent_index < room.entities.len() {
            room.entities.remove(ent_index);
        }
        if let RoomItemRef::Entity(sel_index) = self.room_editor_state.selected_item && sel_index == ent_index {
            self.room_editor_state.selected_item = RoomItemRef::None;
        }
    }

    fn add_trigger(&mut self, room: &mut Room) {
        self.room_editor_state.selected_item = RoomItemRef::Trigger(room.triggers.len());
        let name = Self::get_new_item_name(&room.triggers, "trigger");
        room.triggers.push(RoomTrigger {
            name,
            x: 0,
            y: 0,
            width: 32,
            height: 32,
            data0: 0,
            data1: 0,
            data2: 0,
            data3: 0,
        });
    }

    fn remove_trigger(&mut self, room: &mut Room, trg_index: usize) {
        if trg_index < room.triggers.len() {
            room.triggers.remove(trg_index);
        }
        if let RoomItemRef::Trigger(sel_index) = self.room_editor_state.selected_item && sel_index == trg_index {
            self.room_editor_state.selected_item = RoomItemRef::None;
        }
    }

    fn show_map_tree(&self, ui: &mut egui::Ui, room: &Room, maps: &AssetList<MapData>) -> (bool, Option<usize>) {
        let (mut choose_maps, mut sel_map) = (false, None);

        let tree_node_id = ui.make_persistent_id(format!("editor_{}_map_tree", room.asset.id));
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), tree_node_id, true).show_header(ui, |ui| {
            let resp = ui.add(egui::Label::new("Maps").selectable(false).sense(egui::Sense::click()));
            egui::Popup::context_menu(&resp).show(|ui| {
                ui.horizontal(|ui| {
                    ui.add(egui::Image::new(IMAGES.map_data).max_size(egui::Vec2::splat(crate::app::IMAGE_TREE_SIZE)));
                    if ui.button("Select maps...").clicked() {
                        choose_maps = true;
                    }
                });
            });
        }).body(|ui| {
            for (map_index, room_map) in room.maps.iter().enumerate() {
                if let Some(map) = maps.get(&room_map.map_id) {
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.map_data).max_size(egui::Vec2::splat(crate::app::IMAGE_TREE_SIZE)));
                        let resp = ui.button(&map.asset.name);
                        if resp.clicked() {
                            sel_map = Some(map_index);
                        }
                        egui::Popup::context_menu(&resp).show(|ui| {
                            ui.horizontal(|ui| {
                                ui.add(egui::Image::new(IMAGES.map_data).max_size(egui::Vec2::splat(crate::app::IMAGE_TREE_SIZE)));
                                if ui.button("Select maps...").clicked() {
                                    choose_maps = true;
                                }
                            });
                        });
                    });
                }
            }
        });
        (choose_maps, sel_map)
    }

    fn show_entity_tree(&self, ui: &mut egui::Ui, room: &Room) -> (bool, Option<usize>, Option<usize>) {
        let (mut add_entity, mut sel_entity, mut rm_entity) = (false, None, None);
        let tree_node_id = ui.make_persistent_id(format!("editor_{}_ent_tree", room.asset.id));
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), tree_node_id, true).show_header(ui, |ui| {
            let resp = ui.add(egui::Label::new("Entities").selectable(false).sense(egui::Sense::click()));
            egui::Popup::context_menu(&resp).show(|ui| {
                ui.horizontal(|ui| {
                    ui.add(egui::Image::new(IMAGES.sprite).max_size(egui::Vec2::splat(crate::app::IMAGE_TREE_SIZE)));
                    if ui.button("Add entity").clicked() {
                        add_entity = true;
                    }
                });
            });
        }).body(|ui| {
            for (ent_index, ent) in room.entities.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.add(egui::Image::new(IMAGES.sprite).max_size(egui::Vec2::splat(crate::app::IMAGE_TREE_SIZE)));
                    let resp = ui.button(&ent.name);
                    if resp.clicked() {
                        sel_entity = Some(ent_index);
                    }
                    egui::Popup::context_menu(&resp).show(|ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::Image::new(IMAGES.sprite).max_size(egui::Vec2::splat(crate::app::IMAGE_TREE_SIZE)));
                            if ui.button("Add entity").clicked() {
                                add_entity = true;
                            }
                        });
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.add_space(crate::app::NO_IMAGE_TREE_SIZE);
                            if ui.button("Remove entity").clicked() {
                                rm_entity = Some(ent_index);
                            }
                        });
                    });
                });
            }
        });
        (add_entity, sel_entity, rm_entity)
    }

    fn show_trigger_tree(&self, ui: &mut egui::Ui, room: &Room) -> (bool, Option<usize>, Option<usize>) {
        let (mut add_trigger, mut sel_trigger, mut rm_trigger) = (false, None, None);
        let tree_node_id = ui.make_persistent_id(format!("editor_{}_trg_tree", room.asset.id));
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), tree_node_id, true).show_header(ui, |ui| {
            let resp = ui.add(egui::Label::new("Triggers").selectable(false).sense(egui::Sense::click()));
            egui::Popup::context_menu(&resp).show(|ui| {
                ui.horizontal(|ui| {
                    ui.add(egui::Image::new(IMAGES.animation).max_size(egui::Vec2::splat(crate::app::IMAGE_TREE_SIZE)));
                    if ui.button("Add trigger").clicked() {
                        add_trigger = true;
                    }
                });
            });
        }).body(|ui| {
            for (trg_index, trg) in room.triggers.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.add(egui::Image::new(IMAGES.animation).max_size(egui::Vec2::splat(crate::app::IMAGE_TREE_SIZE)));
                    let resp = ui.button(&trg.name);
                    if resp.clicked() {
                        sel_trigger = Some(trg_index);
                    }
                    egui::Popup::context_menu(&resp).show(|ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::Image::new(IMAGES.animation).max_size(egui::Vec2::splat(crate::app::IMAGE_TREE_SIZE)));
                            if ui.button("Add trigger").clicked() {
                                add_trigger = true;
                            }
                        });
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.add_space(crate::app::NO_IMAGE_TREE_SIZE);
                            if ui.button("Remove trigger").clicked() {
                                rm_trigger = Some(trg_index);
                            }
                        });
                    });
                });
            }
        });
        (add_trigger, sel_trigger, rm_trigger)
    }

    fn show_map_properties(&self, ui: &mut egui::Ui, map_index: usize, room: &mut Room,
                           maps: &AssetList<MapData>) -> Option<()> {
        let asset_id = room.asset.id;
        let room_map = room.maps.get_mut(map_index)?;
        let tree_node_id = ui.make_persistent_id(format!("editor_{}_map_prop_tree", asset_id));

        let map_name = if let Some(map_data) = maps.get(&room_map.map_id) {
            &map_data.asset.name
        } else {
            &format!("?map_id:{}", room_map.map_id)
        };

        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), tree_node_id, true).show_header(ui, |ui| {
            ui.add(egui::Image::new(IMAGES.sprite).max_size(egui::Vec2::splat(crate::app::IMAGE_TREE_SIZE)));
            ui.add(egui::Label::new("Map").selectable(false));
        }).body(|ui| {
            egui::Grid::new(format!("editor_{}_map_prop_grid", asset_id))
                .num_columns(2)
                .spacing([8.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Map:");
                    ui.label(map_name);
                    ui.end_row();

                    ui.label("X:");
                    edit_prop_u16(ui, &mut room_map.x, 0, 1024);
                    ui.end_row();

                    ui.label("Y:");
                    edit_prop_u16(ui, &mut room_map.y, 0, 1024);
                    ui.end_row();
                });
        });
        None
    }

    fn show_entity_properties(&self, ui: &mut egui::Ui, ent_index: usize, room: &mut Room,
                              animations: &AssetList<SpriteAnimation>, animation_ids: &AssetIdList) -> Option<()> {
        let asset_id = room.asset.id;
        let entity = room.entities.get_mut(ent_index)?;
        let tree_node_id = ui.make_persistent_id(format!("editor_{}_ent_prop_tree", asset_id));
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), tree_node_id, true).show_header(ui, |ui| {
            ui.add(egui::Image::new(IMAGES.sprite).max_size(egui::Vec2::splat(crate::app::IMAGE_TREE_SIZE)));
            ui.add(egui::Label::new("Entity").selectable(false));
        }).body(|ui| {
            egui::Grid::new(format!("editor_{}_ent_prop_grid", asset_id))
                .num_columns(2)
                .spacing([8.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut entity.name);
                    ui.end_row();

                    ui.label("Anim:");
                    let cur_animation_name = if let Some(animation) = animations.get(&entity.animation_id) {
                        &animation.asset.name
                    } else {
                        "??"
                    };
                    egui::ComboBox::from_id_salt(format!("editor_{}_ent_prop_animation", asset_id))
                        .selected_text(cur_animation_name)
                        .show_ui(ui, |ui| {
                            for animation_id in animation_ids.iter() {
                                if let Some(animation) = animations.get(animation_id) {
                                    ui.selectable_value(&mut entity.animation_id, animation.asset.id, &animation.asset.name);
                                }
                            }
                        });
                    ui.end_row();

                    ui.label("X:");
                    edit_prop_i16(ui, &mut entity.x, -256, 1024);
                    ui.end_row();

                    ui.label("Y:");
                    edit_prop_i16(ui, &mut entity.y, -256, 1024);
                    ui.end_row();

                    ui.label("Data0:");
                    edit_prop_u16(ui, &mut entity.data0, 0, u16::MAX);
                    ui.end_row();

                    ui.label("Data1:");
                    edit_prop_u16(ui, &mut entity.data1, 0, u16::MAX);
                    ui.end_row();

                    ui.label("Data2:");
                    edit_prop_u16(ui, &mut entity.data2, 0, u16::MAX);
                    ui.end_row();

                    ui.label("Data3:");
                    edit_prop_u16(ui, &mut entity.data3, 0, u16::MAX);
                    ui.end_row();
                });
        });
        None
    }

    fn show_trigger_properties(&self, ui: &mut egui::Ui, trg_index: usize, room: &mut Room) -> Option<()> {
        let asset_id = room.asset.id;
        let trigger = room.triggers.get_mut(trg_index)?;
        let tree_node_id = ui.make_persistent_id(format!("editor_{}_trg_prop_tree", asset_id));
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), tree_node_id, true).show_header(ui, |ui| {
            ui.add(egui::Image::new(IMAGES.animation).max_size(egui::Vec2::splat(crate::app::IMAGE_TREE_SIZE)));
            ui.add(egui::Label::new("Trigger").selectable(false));
        }).body(|ui| {
            egui::Grid::new(format!("editor_{}_trg_prop_grid", asset_id))
                .num_columns(2)
                .spacing([8.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut trigger.name);
                    ui.end_row();

                    ui.label("X:");
                    edit_prop_i16(ui, &mut trigger.x, -256, 1024);
                    ui.end_row();

                    ui.label("Y:");
                    edit_prop_i16(ui, &mut trigger.y, -256, 1024);
                    ui.end_row();

                    ui.label("Width:");
                    edit_prop_u16(ui, &mut trigger.width, 0, u16::MAX);
                    ui.end_row();

                    ui.label("Height:");
                    edit_prop_u16(ui, &mut trigger.height, 0, u16::MAX);
                    ui.end_row();

                    ui.label("Data0:");
                    edit_prop_u16(ui, &mut trigger.data0, 0, u16::MAX);
                    ui.end_row();

                    ui.label("Data1:");
                    edit_prop_u16(ui, &mut trigger.data1, 0, u16::MAX);
                    ui.end_row();

                    ui.label("Data2:");
                    edit_prop_u16(ui, &mut trigger.data2, 0, u16::MAX);
                    ui.end_row();

                    ui.label("Data3:");
                    edit_prop_u16(ui, &mut trigger.data3, 0, u16::MAX);
                    ui.end_row();
                });
        });
        None
    }

    fn show_item_properties(&self, ui: &mut egui::Ui, room: &mut Room, maps: &AssetList<MapData>,
                            animations: &AssetList<SpriteAnimation>, animation_ids: &AssetIdList) {
        match self.room_editor_state.selected_item {
            RoomItemRef::None => {},
            RoomItemRef::Map(map_index) => { self.show_map_properties(ui, map_index, room, maps); },
            RoomItemRef::Entity(ent_index) => { self.show_entity_properties(ui, ent_index, room, animations, animation_ids); },
            RoomItemRef::Trigger(trg_index) => { self.show_trigger_properties(ui, trg_index, room); },
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, room: &mut Room, asset_ids: &AssetIdCollection, assets: &RoomEditorAssetLists) {
        if self.properties_dialog.open {
            self.properties_dialog.show(wc, room);
        }
        if self.map_selection_dialog.open {
            self.map_selection_dialog.show(wc, room, &asset_ids.maps, assets.maps, assets.tilesets);
        }

        let asset_id = room.asset.id;
        let title = format!("{} - Room", room.asset.name);
        let window = super::create_editor_window(asset_id, &title, wc);
        let mut asset_open = self.asset.open;
        window.open(&mut asset_open).min_size([400.0, 300.0]).default_size([600.0, 400.0]).show(wc.egui.ctx, |ui| {
            // header:
            egui::TopBottomPanel::top(format!("editor_panel_{}_top", asset_id)).show_inside(ui, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("Room", |ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::Image::new(IMAGES.properties).max_width(14.0).max_height(14.0));
                            if ui.button("Properties...").clicked() {
                                self.properties_dialog.set_open(room);
                            }
                        });
                    });
                });
            });

            // footer:
            egui::TopBottomPanel::bottom(format!("editor_panel_{}_bottom", asset_id)).show_inside(ui, |ui| {
                ui.add_space(5.0);
                ui.label(format!("{} bytes", room.data_size()));
            });

            // left panel:
            egui::SidePanel::left(format!("editor_panel_{}_left", asset_id)).resizable(false).show_inside(ui, |ui| {
                ui.add_space(5.0);
                let want_height = 70.0_f32.max(ui.available_height() / 2.0);
                ui.allocate_ui(egui::Vec2::new(200.0, want_height), |ui| {
                    egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                        let (change_maps, sel_map) = self.show_map_tree(ui, room, assets.maps);
                        let (add_entity, sel_entity, rm_entity) = self.show_entity_tree(ui, room);
                        let (add_trigger, sel_trigger, rm_trigger) = self.show_trigger_tree(ui, room);

                        if change_maps { self.map_selection_dialog.set_open(room); }
                        if add_entity { self.add_entity(wc, room, &asset_ids.animations); }
                        if add_trigger { self.add_trigger(room); }
                        if let Some(map_index) = sel_map { self.room_editor_state.selected_item = RoomItemRef::Map(map_index); }
                        if let Some(ent_index) = sel_entity { self.room_editor_state.selected_item = RoomItemRef::Entity(ent_index); }
                        if let Some(trg_index) = sel_trigger { self.room_editor_state.selected_item = RoomItemRef::Trigger(trg_index); }
                        if let Some(ent_index) = rm_entity { self.remove_entity(room, ent_index); }
                        if let Some(trg_index) = rm_trigger { self.remove_trigger(room, trg_index); }
                    });
                });
                ui.separator();
                egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                    self.show_item_properties(ui, room, assets.maps, assets.animations, &asset_ids.animations);
                });
            });

            // body:
            egui::CentralPanel::default().show_inside(ui, |ui| {
                super::widgets::room_editor(ui, wc, &mut self.room_editor_state, room, assets);
            });
        });
        self.asset.open = asset_open;
    }
}

struct PropertiesDialog {
    open: bool,
    name: String,
}

impl PropertiesDialog {
    fn new() -> Self {
        PropertiesDialog {
            open: false,
            name: String::new(),
        }
    }

    fn set_open(&mut self, room: &Room) {
        self.name.clear();
        self.name.push_str(&room.asset.name);
        self.open = true;
    }

    fn confirm(&mut self, room: &mut Room) {
        room.asset.name.clear();
        room.asset.name.push_str(&self.name);
    }

    fn show(&mut self, wc: &WindowContext, room: &mut Room) {
        if egui::Modal::new(egui::Id::new("dlg_room_properties")).show(wc.egui.ctx, |ui| {
            ui.set_width(250.0);
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                ui.heading("Room Properties");
                ui.add_space(16.0);

                egui::Grid::new(format!("editor_panel_{}_prop_grid", room.asset.id))
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.name);
                        ui.end_row();
                    });

                ui.add_space(16.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui.button("Cancel").clicked() {
                        ui.close();
                    }
                    if ui.button("Ok").clicked() {
                        self.confirm(room);
                        ui.close();
                    }
                });
            });
        }).should_close() {
            self.open = false;
        }
    }
}

struct MapSelectionDialog {
    open: bool,
    sel_map_ids: HashSet<DataAssetId>,
    display_map_id: Option<DataAssetId>,
}

impl MapSelectionDialog {
    fn new() -> Self {
        MapSelectionDialog {
            open: false,
            sel_map_ids: HashSet::new(),
            display_map_id: None,
        }
    }

    fn set_open(&mut self, room: &Room) {
        self.sel_map_ids.clear();
        for map in &room.maps {
            self.sel_map_ids.insert(map.map_id);
        }
        self.display_map_id = room.maps.first().map(|m| m.map_id);
        self.open = true;
    }

    fn confirm(&mut self, room: &mut Room) {
        // remove maps not in selection
        room.maps.retain(|m| self.sel_map_ids.contains(&m.map_id));

        // add selected maps that are not in the room
        for &map_id in self.sel_map_ids.iter() {
            if ! room.maps.iter().any(|m| m.map_id == map_id) {
                room.maps.push(RoomMap {
                    x: 0,
                    y: 0,
                    map_id,
                });
            }
        }

        room.maps.sort_by(|m1, m2| m1.map_id.cmp(&m2.map_id));
    }

    fn show(&mut self, wc: &mut WindowContext, room: &mut Room, all_map_ids: &AssetIdList,
            maps: &AssetList<MapData>, tilesets: &AssetList<Tileset>) {
        if egui::Modal::new(egui::Id::new("dlg_room_maps")).show(wc.egui.ctx, |ui| {
            let asset_id = room.asset.id;
            egui::TopBottomPanel::top(format!("editor_panel_{}_maps_top", asset_id)).show_inside(ui, |ui| {
                ui.add_space(2.0);
                ui.heading("Map Selection");
                ui.add_space(2.0);
            });

            egui::TopBottomPanel::bottom(format!("editor_panel_{}_maps_bot", asset_id)).show_separator_line(false).show_inside(ui, |ui| {
                ui.add_space(5.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui.button("Cancel").clicked() {
                        ui.close();
                    }
                    if ui.button("Ok").clicked() {
                        self.confirm(room);
                        ui.close();
                    }
                });
            });

            egui::SidePanel::left(format!("editor_panel_{}_maps_left", asset_id))
                .resizable(false)
                .show_separator_line(false)
                .min_width(200.0)
                .show_inside(ui, |ui| {
                    ui.add_space(5.0);
                    ui.label("Available maps:");
                    ui.add_space(5.0);
                    egui::ScrollArea::both().auto_shrink([false, true]).show(ui, |ui| {
                        for &map_id in all_map_ids.iter() {
                            if let Some(map) = maps.get(&map_id) {
                                ui.horizontal(|ui| {
                                    let mut checked = self.sel_map_ids.contains(&map_id);
                                    let old_checked = checked;
                                    ui.add(egui::Checkbox::without_text(&mut checked));
                                    if checked != old_checked {
                                        self.display_map_id = Some(map_id);
                                        if checked {
                                            self.sel_map_ids.insert(map_id);
                                        } else  {
                                            self.sel_map_ids.remove(&map_id);
                                        }
                                    }
                                    if ui.button(&map.asset.name).clicked() {
                                        self.display_map_id = Some(map_id);
                                    }
                                });
                            }
                        }
                    });
                });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                if let Some(map_data) = self.display_map_id.and_then(|map_id| maps.get(&map_id)) &&
                    let Some(tileset) = tilesets.get(&map_data.tileset_id) {
                        ui.label(format!("Map: {} ({}x{})", map_data.asset.name, map_data.width, map_data.height));
                        super::widgets::map_view(ui, wc, map_data, tileset);
                    }
            });

        }).should_close() {
            self.open = false;
        }
    }
}
