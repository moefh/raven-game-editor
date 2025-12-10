mod properties;
mod map_selection;

use crate::misc::IMAGES;
use crate::app::WindowContext;
use crate::data_asset::{
    Room, RoomEntity, RoomTrigger, RoomItem,
    MapData, Tileset, SpriteAnimation, Sprite,
    DataAssetId, AssetIdCollection, GenericAsset, AssetIdList, AssetList,
};

use properties::PropertiesDialog;
use map_selection::MapSelectionDialog;
use super::widgets::RoomEditorWidget;

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

#[derive(Clone, Copy, Debug)]
pub enum RoomItemRef {
    None,
    Map(usize),
    Entity(usize),
    Trigger(usize),
}

#[allow(dead_code)]
impl RoomItemRef {
    pub fn is_none(&self) -> bool {
        matches!(self, RoomItemRef::None)
    }

    pub fn is_some(&self) -> bool {
        ! self.is_none()
    }

    pub fn is_map(&self) -> bool {
        matches!(self, RoomItemRef::Map(_))
    }

    pub fn is_entity(&self) -> bool {
        matches!(self, RoomItemRef::Entity(_))
    }

    pub fn is_trigger(&self) -> bool {
        matches!(self, RoomItemRef::Trigger(_))
    }

    pub fn is_the_map(&self, map_index: usize) -> bool {
        match self {
            RoomItemRef::Map(sel_map) => *sel_map == map_index,
            _ => false,
        }
    }

    pub fn is_the_entity(&self, ent_index: usize) -> bool {
        match self {
            RoomItemRef::Entity(sel_entity) => *sel_entity == ent_index,
            _ => false,
        }
    }

    pub fn is_the_trigger(&self, trg_index: usize) -> bool {
        match self {
            RoomItemRef::Trigger(sel_trigger) => *sel_trigger == trg_index,
            _ => false,
        }
    }
}

pub struct RoomEditor {
    pub asset: super::DataAssetEditor,
    editor: Editor,
    dialogs: Dialogs,
}

impl RoomEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        RoomEditor {
            asset: super::DataAssetEditor::new(id, open),
            editor: Editor::new(id),
            dialogs: Dialogs::new(),
        }
    }

    pub fn prepare_for_saving(&mut self, _asset: &mut impl crate::data_asset::GenericAsset) {
    }

    pub fn show(&mut self, wc: &mut WindowContext, room: &mut Room, asset_ids: &AssetIdCollection, assets: &RoomEditorAssetLists) {
        self.dialogs.show(wc, &mut self.editor, room, asset_ids, assets);

        let title = format!("{} - Room", room.asset.name);
        let window = super::DataAssetEditor::create_window(&mut self.asset, wc, &title);
        window.min_size([400.0, 300.0]).default_size([600.0, 400.0]).show(wc.egui.ctx, |ui| {
            self.editor.show(ui, wc, &mut self.dialogs, room, asset_ids, assets);
        });
    }
}

struct Dialogs {
    properties_dialog: PropertiesDialog,
    map_selection_dialog: MapSelectionDialog,
}

impl Dialogs {
    fn new() -> Self {
        Dialogs {
            properties_dialog: PropertiesDialog::new(),
            map_selection_dialog: MapSelectionDialog::new(),
        }
    }

    fn show(&mut self, wc: &mut WindowContext, editor: &mut Editor, room: &mut Room,
            asset_ids: &AssetIdCollection, assets: &RoomEditorAssetLists) {
        if self.properties_dialog.open {
            self.properties_dialog.show(wc, room);
        }
        if self.map_selection_dialog.open &&
            self.map_selection_dialog.show(wc, room, &asset_ids.maps, assets.maps, assets.tilesets) &&
            editor.room_editor.selected_item.is_map() {
                editor.room_editor.selected_item = if room.maps.is_empty() { RoomItemRef::None } else { RoomItemRef::Map(0) };
            }
    }
}

struct Editor {
    asset_id: DataAssetId,
    room_editor: RoomEditorWidget,
}

impl Editor {
    fn new(asset_id: DataAssetId) -> Self {
        Editor {
            asset_id,
            room_editor: RoomEditorWidget::new(),
        }
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
                wc.open_message_box("No Animation Available", "You must create a sprite animation first!");
                return;
            },
        };

        self.room_editor.selected_item = RoomItemRef::Entity(room.entities.len());
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
        if let RoomItemRef::Entity(sel_index) = self.room_editor.selected_item && sel_index == ent_index {
            self.room_editor.selected_item = RoomItemRef::None;
        }
    }

    fn add_trigger(&mut self, room: &mut Room) {
        self.room_editor.selected_item = RoomItemRef::Trigger(room.triggers.len());
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
        if let RoomItemRef::Trigger(sel_index) = self.room_editor.selected_item && sel_index == trg_index {
            self.room_editor.selected_item = RoomItemRef::None;
        }
    }

    fn show_map_tree(&self, ui: &mut egui::Ui, room: &Room, maps: &AssetList<MapData>) -> (bool, Option<usize>) {
        let (mut choose_maps, mut sel_map) = (false, None);

        let tree_node_id = ui.make_persistent_id(format!("editor_{}_map_tree", room.asset.id));
        let node = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), tree_node_id, true);
        let mut toggle_node_open = false;
        let mut node_resp = node.show_header(ui, |ui| {
            let resp = ui.add(egui::Label::new("Maps").selectable(false).sense(egui::Sense::click()));
            egui::Popup::context_menu(&resp).show(|ui| {
                ui.horizontal(|ui| {
                    ui.add(egui::Image::new(IMAGES.map_data).max_size(egui::Vec2::splat(crate::app::IMAGE_TREE_SIZE)));
                    if ui.button("Select maps...").clicked() {
                        choose_maps = true;
                    }
                });
            });
            toggle_node_open = resp.clicked();
        });
        if toggle_node_open {
            node_resp.toggle();
        }
        node_resp.body(|ui| {
            for (map_index, room_map) in room.maps.iter().enumerate() {
                if let Some(map) = maps.get(&room_map.map_id) {
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.map_data).max_size(egui::Vec2::splat(crate::app::IMAGE_TREE_SIZE)));
                        let mut selected = self.room_editor.selected_item.is_the_map(map_index);
                        let resp = ui.toggle_value(&mut selected, &map.asset.name);
                        if resp.clicked() || resp.secondary_clicked() {
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
        let node = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), tree_node_id, true);
        let mut toggle_node_open = false;
        let mut node_resp = node.show_header(ui, |ui| {
            let resp = ui.add(egui::Label::new("Entities").selectable(false).sense(egui::Sense::click()));
            egui::Popup::context_menu(&resp).show(|ui| {
                ui.horizontal(|ui| {
                    ui.add(egui::Image::new(IMAGES.sprite).max_size(egui::Vec2::splat(crate::app::IMAGE_TREE_SIZE)));
                    if ui.button("Add entity").clicked() {
                        add_entity = true;
                    }
                });
            });
            toggle_node_open = resp.clicked();
        });
        if toggle_node_open {
            node_resp.toggle();
        }
        node_resp.body(|ui| {
            for (ent_index, ent) in room.entities.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.add(egui::Image::new(IMAGES.sprite).max_size(egui::Vec2::splat(crate::app::IMAGE_TREE_SIZE)));
                    let mut selected = self.room_editor.selected_item.is_the_entity(ent_index);
                    let resp = ui.toggle_value(&mut selected, &ent.name);
                    if resp.clicked() || resp.secondary_clicked() {
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
        let node = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), tree_node_id, true);
        let mut toggle_node_open = false;
        let mut node_resp = node.show_header(ui, |ui| {
            let resp = ui.add(egui::Label::new("Triggers").selectable(false).sense(egui::Sense::click()));
            egui::Popup::context_menu(&resp).show(|ui| {
                ui.horizontal(|ui| {
                    ui.add(egui::Image::new(IMAGES.animation).max_size(egui::Vec2::splat(crate::app::IMAGE_TREE_SIZE)));
                    if ui.button("Add trigger").clicked() {
                        add_trigger = true;
                    }
                });
            });
            toggle_node_open = resp.clicked();
        });
        if toggle_node_open {
            node_resp.toggle();
        }
        node_resp.body(|ui| {
            for (trg_index, trg) in room.triggers.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.add(egui::Image::new(IMAGES.animation).max_size(egui::Vec2::splat(crate::app::IMAGE_TREE_SIZE)));
                    let mut selected = self.room_editor.selected_item.is_the_trigger(trg_index);
                    let resp = ui.toggle_value(&mut selected, &trg.name);
                    if resp.clicked() || resp.secondary_clicked() {
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
        let room_map = room.maps.get_mut(map_index)?;
        let tree_node_id = ui.make_persistent_id(format!("editor_{}_map_prop_tree", self.asset_id));

        let map_name = if let Some(map_data) = maps.get(&room_map.map_id) {
            &map_data.asset.name
        } else {
            &format!("?map_id:{}", room_map.map_id)
        };

        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), tree_node_id, true).show_header(ui, |ui| {
            ui.add(egui::Image::new(IMAGES.sprite).max_size(egui::Vec2::splat(crate::app::IMAGE_TREE_SIZE)));
            ui.add(egui::Label::new("Map").selectable(false));
        }).body(|ui| {
            egui::Grid::new(format!("editor_{}_map_prop_grid", self.asset_id))
                .num_columns(2)
                .spacing([8.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Map:");
                    ui.label(map_name);
                    ui.end_row();

                    ui.label("X:");
                    Self::edit_prop_u16(ui, &mut room_map.x, 0, 2048);
                    ui.end_row();

                    ui.label("Y:");
                    Self::edit_prop_u16(ui, &mut room_map.y, 0, 2048);
                    ui.end_row();
                });
        });
        None
    }

    fn show_entity_properties(&self, ui: &mut egui::Ui, ent_index: usize, room: &mut Room,
                              animations: &AssetList<SpriteAnimation>, animation_ids: &AssetIdList) -> Option<()> {
        let entity = room.entities.get_mut(ent_index)?;
        let tree_node_id = ui.make_persistent_id(format!("editor_{}_ent_prop_tree", self.asset_id));
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), tree_node_id, true).show_header(ui, |ui| {
            ui.add(egui::Image::new(IMAGES.sprite).max_size(egui::Vec2::splat(crate::app::IMAGE_TREE_SIZE)));
            ui.add(egui::Label::new("Entity").selectable(false));
        }).body(|ui| {
            egui::Grid::new(format!("editor_{}_ent_prop_grid", self.asset_id))
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
                    egui::ComboBox::from_id_salt(format!("editor_{}_ent_prop_animation", self.asset_id))
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
                    Self::edit_prop_i16(ui, &mut entity.x, -256, i16::MAX);
                    ui.end_row();

                    ui.label("Y:");
                    Self::edit_prop_i16(ui, &mut entity.y, -256, i16::MAX);
                    ui.end_row();

                    ui.label("Data0:");
                    Self::edit_prop_u16(ui, &mut entity.data0, 0, u16::MAX);
                    ui.end_row();

                    ui.label("Data1:");
                    Self::edit_prop_u16(ui, &mut entity.data1, 0, u16::MAX);
                    ui.end_row();

                    ui.label("Data2:");
                    Self::edit_prop_u16(ui, &mut entity.data2, 0, u16::MAX);
                    ui.end_row();

                    ui.label("Data3:");
                    Self::edit_prop_u16(ui, &mut entity.data3, 0, u16::MAX);
                    ui.end_row();
                });
        });
        None
    }

    fn show_trigger_properties(&self, ui: &mut egui::Ui, trg_index: usize, room: &mut Room) -> Option<()> {
        let trigger = room.triggers.get_mut(trg_index)?;
        let tree_node_id = ui.make_persistent_id(format!("editor_{}_trg_prop_tree", self.asset_id));
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), tree_node_id, true).show_header(ui, |ui| {
            ui.add(egui::Image::new(IMAGES.animation).max_size(egui::Vec2::splat(crate::app::IMAGE_TREE_SIZE)));
            ui.add(egui::Label::new("Trigger").selectable(false));
        }).body(|ui| {
            egui::Grid::new(format!("editor_{}_trg_prop_grid", self.asset_id))
                .num_columns(2)
                .spacing([8.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut trigger.name);
                    ui.end_row();

                    ui.label("X:");
                    Self::edit_prop_i16(ui, &mut trigger.x, -256, i16::MAX);
                    ui.end_row();

                    ui.label("Y:");
                    Self::edit_prop_i16(ui, &mut trigger.y, -256, i16::MAX);
                    ui.end_row();

                    ui.label("Width:");
                    Self::edit_prop_i16(ui, &mut trigger.width, 0, i16::MAX);
                    ui.end_row();

                    ui.label("Height:");
                    Self::edit_prop_i16(ui, &mut trigger.height, 0, i16::MAX);
                    ui.end_row();

                    ui.label("Data0:");
                    Self::edit_prop_u16(ui, &mut trigger.data0, 0, u16::MAX);
                    ui.end_row();

                    ui.label("Data1:");
                    Self::edit_prop_u16(ui, &mut trigger.data1, 0, u16::MAX);
                    ui.end_row();

                    ui.label("Data2:");
                    Self::edit_prop_u16(ui, &mut trigger.data2, 0, u16::MAX);
                    ui.end_row();

                    ui.label("Data3:");
                    Self::edit_prop_u16(ui, &mut trigger.data3, 0, u16::MAX);
                    ui.end_row();
                });
        });
        None
    }

    fn show_item_properties(&self, ui: &mut egui::Ui, room: &mut Room, maps: &AssetList<MapData>,
                            animations: &AssetList<SpriteAnimation>, animation_ids: &AssetIdList) {
        match self.room_editor.selected_item {
            RoomItemRef::None => {},
            RoomItemRef::Map(map_index) => { self.show_map_properties(ui, map_index, room, maps); },
            RoomItemRef::Entity(ent_index) => { self.show_entity_properties(ui, ent_index, room, animations, animation_ids); },
            RoomItemRef::Trigger(trg_index) => { self.show_trigger_properties(ui, trg_index, room); },
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, dialogs: &mut Dialogs,
                room: &mut Room, asset_ids: &AssetIdCollection, assets: &RoomEditorAssetLists) {
        // header:
        egui::TopBottomPanel::top(format!("editor_panel_{}_top", self.asset_id)).show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Room", |ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.properties).max_width(14.0).max_height(14.0));
                        if ui.button("Properties...").clicked() {
                            dialogs.properties_dialog.set_open(wc, room);
                        }
                    });
                });
            });
        });

        // footer:
        egui::TopBottomPanel::bottom(format!("editor_panel_{}_bottom", self.asset_id)).show_inside(ui, |ui| {
            ui.add_space(5.0);
            ui.label(format!(
                "{} bytes [maps: {}, entities: {}, triggers: {}]",
                room.data_size(),
                room.maps.len(),
                room.entities.len(),
                room.triggers.len()
            ));
        });

        // left panel:
        egui::SidePanel::left(format!("editor_panel_{}_left", self.asset_id)).resizable(false).show_inside(ui, |ui| {
            ui.add_space(5.0);
            let want_height = 70.0_f32.max(ui.available_height() / 2.0);
            ui.allocate_ui(egui::Vec2::new(200.0, want_height), |ui| {
                egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                    let (change_maps, sel_map) = self.show_map_tree(ui, room, assets.maps);
                    let (add_entity, sel_entity, rm_entity) = self.show_entity_tree(ui, room);
                    let (add_trigger, sel_trigger, rm_trigger) = self.show_trigger_tree(ui, room);

                    if change_maps { dialogs.map_selection_dialog.set_open(wc, room); }
                    if add_entity { self.add_entity(wc, room, &asset_ids.animations); }
                    if add_trigger { self.add_trigger(room); }
                    if let Some(map_index) = sel_map { self.room_editor.selected_item = RoomItemRef::Map(map_index); }
                    if let Some(ent_index) = sel_entity { self.room_editor.selected_item = RoomItemRef::Entity(ent_index); }
                    if let Some(trg_index) = sel_trigger { self.room_editor.selected_item = RoomItemRef::Trigger(trg_index); }
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
            self.room_editor.show(ui, wc, room, assets);
        });
    }
}
