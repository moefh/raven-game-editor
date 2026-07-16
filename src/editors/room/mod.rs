mod properties;
mod map_selection;

use std::collections::HashMap;

use crate::misc::IMAGES;
use crate::app::{
    menu_item,
    WindowContext,
};
use crate::data_asset::{
    Room,
    RoomTrigger,
    RoomTriggerType,
    RoomItem,
    MapData,
    Tileset,
    SpriteAnimation,
    Sprite,
    DataAssetId,
    AssetIdCollection,
    GenericAsset,
    AssetList,
};

use properties::PropertiesDialog;
use map_selection::MapSelectionDialog;
use super::{
    RoomSize,
    AssetEditorBase,
    RoomTriggerTypeSel,
};
use super::widgets::RoomEditorWidget;

fn get_trigger_image(trigger: &RoomTrigger) -> egui::ImageSource<'static> {
    match trigger.trigger_type {
        RoomTriggerType::Trap {..} => { IMAGES.log }
        RoomTriggerType::Door {..} => { IMAGES.room }
        RoomTriggerType::PlayerSpawn {..} => { IMAGES.log }
        RoomTriggerType::EnemySpawn {..} => { IMAGES.animation }
       _ => { IMAGES.info }
    }
}

pub struct RoomEditorAssetLists<'a> {
    pub maps: &'a AssetList<MapData>,
    pub tilesets: &'a AssetList<Tileset>,
    pub animations: &'a AssetList<SpriteAnimation>,
    pub sprites: &'a AssetList<Sprite>,
    pub room_names: &'a HashMap<DataAssetId, String>,
}

impl<'a> RoomEditorAssetLists<'a> {
    pub fn new(maps: &'a AssetList<MapData>, tilesets: &'a AssetList<Tileset>,
               animations: &'a AssetList<SpriteAnimation>, sprites: &'a AssetList<Sprite>,
               room_names: &'a HashMap<DataAssetId, String>) -> Self {
        RoomEditorAssetLists {
            maps,
            tilesets,
            animations,
            sprites,
            room_names,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum RoomItemRef {
    None,
    Screen,
    Map(usize),
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

    pub fn is_screen(&self) -> bool {
        matches!(self, RoomItemRef::Screen)
    }

    pub fn is_map(&self) -> bool {
        matches!(self, RoomItemRef::Map(_))
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

    pub fn is_the_trigger(&self, trg_index: usize) -> bool {
        match self {
            RoomItemRef::Trigger(sel_trigger) => *sel_trigger == trg_index,
            _ => false,
        }
    }
}

pub struct RoomEditor {
    pub base: AssetEditorBase,
    editor: Editor,
    dialogs: Dialogs,
}

impl RoomEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        RoomEditor {
            base: AssetEditorBase::new(id, open),
            editor: Editor::new(id),
            dialogs: Dialogs::new(),
        }
    }

    pub fn prepare_for_saving(&mut self, _asset: &mut impl crate::data_asset::GenericAsset) {
    }

    fn show_footer(ui: &mut egui::Ui, wc: &WindowContext, room: &Room, maps: &AssetList<MapData>, base: &AssetEditorBase) {
        let margin = egui::Margin { left: 5, right: 5, top: 4, bottom: 0 };
        let bottom_frame = egui::Frame::NONE.inner_margin(margin).fill(base.footer_bg_color(wc, room.asset.id));
        egui::Panel::bottom(format!("editor_panel_{}_bottom", room.asset.id)).frame(bottom_frame).show(ui, |ui| {
            let dirty = if base.is_dirty() { " (modified)" } else { "" };
            let room_size = RoomSize::from_room(room, maps);
            ui.add(egui::Label::new(format!(
                "{} bytes [{}x{}, {} map{}, {} trigger{}]{}",
                room.data_size(),
                room_size.width,
                room_size.height,
                room.maps.len(),
                if room.maps.len() != 1 { "s" } else { "" },
                room.triggers.len(),
                if room.triggers.len() != 1 { "s" } else { "" },
                dirty
            )).truncate());
        });
    }

    pub fn show(&mut self, wc: &mut WindowContext, room: &mut Room, asset_ids: &AssetIdCollection, assets: &RoomEditorAssetLists) {
        self.dialogs.show(wc, &mut self.editor, room, assets);

        self.base.show_window(wc, room, [600.0, 300.0], [700.0, 400.0], |ui, wc, room, base| {
            Self::show_footer(ui, wc, room, assets.maps, base);
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

    fn get_selected_map_id(editor: &Editor, room: &Room) -> Option<DataAssetId> {
        if let RoomItemRef::Map(map_index) = editor.room_editor.get_selected_item() {
            room.maps.get(map_index).map(|m| m.map_id)
        } else {
            None
        }
    }

    fn select_map_id(map_id: Option<DataAssetId>, editor: &mut Editor, room: &Room) {
        if let Some(map_id) = map_id {
            let new_selection = if let Some(map_index) = room.maps.iter().position(|m| m.map_id == map_id) {
                RoomItemRef::Map(map_index)
            } else {
                RoomItemRef::None
            };
            editor.room_editor.set_selected_item(new_selection, false);
        }
    }

    fn show(&mut self, wc: &mut WindowContext, editor: &mut Editor, room: &mut Room, assets: &RoomEditorAssetLists) {
        if self.properties_dialog.open {
            self.properties_dialog.show(wc, room);
        }
        let selected_map_id = Self::get_selected_map_id(editor, room);
        if self.map_selection_dialog.open && self.map_selection_dialog.show(wc, room, assets.maps, assets.tilesets) {
            Self::select_map_id(selected_map_id, editor, room);
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

    fn get_new_item_name_id(items: &[impl RoomItem], base: &str) -> String {
        use std::fmt::Write;

        let mut num: usize = 1;
        let mut new_name_id = String::new();
        loop {
            new_name_id.clear();
            new_name_id.push_str(base);
            if write!(new_name_id, "{}", num).is_err() { break; }
            if ! items.iter().any(|e| e.name_id() == new_name_id) { break; }
            num += 1;
        }
        new_name_id
    }

    fn get_new_trigger_id(triggers: &[RoomTrigger]) -> u16 {
        let mut new_id = 0;
        loop {
            if ! triggers.iter().any(|tr| tr.trigger_id == new_id) { break; }
            new_id += 1;
        }
        new_id
    }

    fn add_trigger(&mut self, room: &mut Room) {
        let trigger_id = Self::get_new_trigger_id(&room.triggers);
        let name_id = Self::get_new_item_name_id(&room.triggers, "trigger");
        let pos = if self.room_editor.zoom <= 0.0 { egui::Vec2::ZERO } else { self.room_editor.scroll / -self.room_editor.zoom };
        let new_trigger_index = room.triggers.len();
        room.triggers.push(RoomTrigger {
            trigger_id,
            name_id,
            x: pos.x.floor().clamp(0.0, i16::MAX as f32) as i16,
            y: pos.y.floor().clamp(0.0, i16::MAX as f32) as i16,
            trigger_type: RoomTriggerType::Unknown { data0: 0, data1: 0, data2: 0, data3: 0 },
        });
        self.room_editor.set_selected_item(RoomItemRef::Trigger(new_trigger_index), true);
    }

    fn remove_trigger(&mut self, room: &mut Room, trg_index: usize) {
        if trg_index < room.triggers.len() {
            room.triggers.remove(trg_index);
        }
        if let RoomItemRef::Trigger(sel_index) = self.room_editor.get_selected_item() && sel_index == trg_index {
            self.room_editor.set_selected_item(RoomItemRef::None, false);
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
                if ui.add(menu_item(IMAGES.map_data, " Select maps...")).clicked() {
                    choose_maps = true;
                }
            });
            toggle_node_open = resp.clicked();
        });
        if toggle_node_open {
            node_resp.toggle();
        }
        node_resp.body(|ui| {
            for (map_index, room_map) in room.maps.iter().enumerate() {
                if let Some(map) = maps.get(&room_map.map_id) {
                    let item = ui.horizontal(|ui| {
                        let mut selected = self.room_editor.get_selected_item().is_the_map(map_index);
                        let button = egui::Button::image_and_text(IMAGES.map_data, &map.asset.name)
                            .frame_when_inactive(selected)
                            .frame(true);
                        let resp = ui.add(button);
                        if resp.clicked() {
                            selected = ! selected;
                            sel_map = Some(map_index);
                        }
                        egui::Popup::context_menu(&resp).show(|ui| {
                            if ui.add(menu_item(IMAGES.map_data, " Select maps...")).clicked() {
                                choose_maps = true;
                            }
                        });
                        selected
                    });
                    if item.inner && self.room_editor.has_selected_item_changed() {
                        item.response.scroll_to_me(None);
                    }
                }
            }
        });
        (choose_maps, sel_map)
    }

    fn show_trigger_tree(&self, ui: &mut egui::Ui, room: &Room) -> (bool, Option<usize>, Option<usize>) {
        let (mut add_trigger, mut sel_trigger, mut rm_trigger) = (false, None, None);
        let tree_node_id = ui.make_persistent_id(format!("editor_{}_trg_tree", room.asset.id));
        let node = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), tree_node_id, true);
        let mut toggle_node_open = false;
        let mut node_resp = node.show_header(ui, |ui| {
            let resp = ui.add(egui::Label::new("Triggers").selectable(false).sense(egui::Sense::click()));
            egui::Popup::context_menu(&resp).show(|ui| {
                if ui.add(menu_item(IMAGES.info, " Add trigger")).clicked() {
                    add_trigger = true;
                }
            });
            toggle_node_open = resp.clicked();
        });
        if toggle_node_open {
            node_resp.toggle();
        }
        node_resp.body(|ui| {
            for (trg_index, trg) in room.triggers.iter().enumerate() {
                let mut selected = self.room_editor.get_selected_item().is_the_trigger(trg_index);
                let button = egui::Button::image_and_text(get_trigger_image(trg), &trg.name_id)
                    .frame_when_inactive(selected)
                    .frame(true);
                let resp = ui.add(button);
                if resp.clicked() {
                    selected = ! selected;
                    sel_trigger = Some(trg_index);
                }
                egui::Popup::context_menu(&resp).show(|ui| {
                    if ui.add(menu_item(IMAGES.info, " Add trigger")).clicked() {
                        add_trigger = true;
                    }
                    ui.separator();
                    if ui.add(menu_item(IMAGES.trash, " Remove trigger")).clicked() {
                        rm_trigger = Some(trg_index);
                    }
                });
                if selected && self.room_editor.has_selected_item_changed() {
                    resp.scroll_to_me(None);
                }
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
            ui.add(egui::Button::image_and_text(IMAGES.map_data, "Map").frame(false));
        }).body(|ui| {
            egui::Grid::new(format!("editor_{}_map_prop_grid", self.asset_id))
                .num_columns(2)
                .spacing([8.0, 2.0])
                .show(ui, |ui| {
                    ui.label("Map:");
                    ui.label(map_name);
                    ui.end_row();

                    if self.room_editor.lock_maps {
                        ui.label("X:"); ui.label(format!("{}", room_map.x)); ui.end_row();
                        ui.label("Y:"); ui.label(format!("{}", room_map.y)); ui.end_row();
                    } else {
                        ui.label("X:"); ui.add(egui::DragValue::new(&mut room_map.x).speed(1.0).range(0..=2048)); ui.end_row();
                        ui.label("Y:"); ui.add(egui::DragValue::new(&mut room_map.y).speed(1.0).range(0..=2048)); ui.end_row();
                    }
                    if let Some(map) = maps.get(&room_map.map_id) {
                        ui.label("Width:"); ui.label(format!("{}", map.width)); ui.end_row();
                        ui.label("Height:"); ui.label(format!("{}", map.height)); ui.end_row();
                    }
                });
        });
        None
    }

    fn show_trigger_properties_grid(
        &self,
        ui: &mut egui::Ui,
        trigger: &mut RoomTrigger,
        asset_ids: &AssetIdCollection,
        assets: &RoomEditorAssetLists
    ) {
        ui.label("Trigger id:");
        ui.horizontal(|ui| {
            if ui.button("\u{2796}").clicked() && trigger.trigger_id > 0 { trigger.trigger_id -= 1; }
            ui.label(format!("{}", trigger.trigger_id));
            if ui.button("\u{2795}").clicked() && trigger.trigger_id < u16::MAX-1 { trigger.trigger_id += 1; }
        });
        ui.end_row();

        ui.label("Type:");
        let mut type_sel = RoomTriggerTypeSel::from_trigger_type(&trigger.trigger_type);
        egui::ComboBox::from_id_salt(format!("editor_{}_ent_prop_type", self.asset_id))
            .selected_text(type_sel.text())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut type_sel, RoomTriggerTypeSel::Unknown, RoomTriggerTypeSel::Unknown.text());
                ui.selectable_value(&mut type_sel, RoomTriggerTypeSel::EnemySpawn, RoomTriggerTypeSel::EnemySpawn.text());
                ui.selectable_value(&mut type_sel, RoomTriggerTypeSel::Door, RoomTriggerTypeSel::Door.text());
                ui.selectable_value(&mut type_sel, RoomTriggerTypeSel::Trap, RoomTriggerTypeSel::Trap.text());
                ui.selectable_value(&mut type_sel, RoomTriggerTypeSel::PlayerSpawn, RoomTriggerTypeSel::PlayerSpawn.text());
            });
        type_sel.convert_trigger_type(&mut trigger.trigger_type, asset_ids);
        ui.end_row();

        ui.label("Name:");
        ui.text_edit_singleline(&mut trigger.name_id);
        ui.end_row();

        ui.label("X:"); ui.add(egui::DragValue::new(&mut trigger.x).speed(1.0).range(-256..=i16::MAX)); ui.end_row();
        ui.label("Y:"); ui.add(egui::DragValue::new(&mut trigger.y).speed(1.0).range(-256..=i16::MAX)); ui.end_row();

        match &mut trigger.trigger_type {
            RoomTriggerType::Unknown { data0, data1, data2, data3 } => {
                ui.label("Data0:"); ui.add(egui::DragValue::new(data0).speed(1.0).range(0..=u16::MAX)); ui.end_row();
                ui.label("Data1:"); ui.add(egui::DragValue::new(data1).speed(1.0).range(0..=u16::MAX)); ui.end_row();
                ui.label("Data2:"); ui.add(egui::DragValue::new(data2).speed(1.0).range(0..=u16::MAX)); ui.end_row();
                ui.label("Data3:"); ui.add(egui::DragValue::new(data3).speed(1.0).range(0..=u16::MAX)); ui.end_row();
            }

            RoomTriggerType::Trap { width, height, trap_type } => {
                ui.label("Width:");
                ui.add(egui::DragValue::new(width).speed(1.0).range(0..=u16::MAX));
                ui.end_row();

                ui.label("Height:");
                ui.add(egui::DragValue::new(height).speed(1.0).range(0..=u16::MAX));
                ui.end_row();

                ui.label("Type:");
                ui.add(egui::DragValue::new(trap_type).speed(1.0).range(0..=u16::MAX));
                ui.end_row();
            }

            RoomTriggerType::PlayerSpawn { direction } => {
                ui.label("Dir:");
                egui::ComboBox::from_id_salt(format!("editor_{}_trg_prop_direction", self.asset_id))
                    .selected_text(if *direction == 0 { "Right" } else { "Left" })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(direction, 0, "Right");
                        ui.selectable_value(direction, 1, "Left");
                    });
                ui.end_row();
            }

            RoomTriggerType::EnemySpawn { animation_id } => {
                ui.label("Anim:");
                let cur_anim_name = if let Some(anim) = assets.animations.get(animation_id) {
                    &anim.asset.name
                } else {
                    "??"
                };
                egui::ComboBox::from_id_salt(format!("editor_{}_trg_prop_animation", self.asset_id))
                    .selected_text(cur_anim_name)
                    .show_ui(ui, |ui| {
                        for anim_id in asset_ids.animations.iter() {
                            if let Some(anim) = assets.animations.get(anim_id) {
                                ui.selectable_value(animation_id, *anim_id, &anim.asset.name);
                            }
                        }
                    });
                ui.end_row();
            }

            RoomTriggerType::Door { dest_room_id, dest_trigger_id } => {
                ui.label("To room:");
                let cur_room_name = if let Some(name) = assets.room_names.get(dest_room_id) {
                    name
                } else {
                    "??"
                };
                egui::ComboBox::from_id_salt(format!("editor_{}_trg_prop_door", self.asset_id))
                    .selected_text(cur_room_name)
                    .show_ui(ui, |ui| {
                        for sel_room_id in asset_ids.rooms.iter() {
                            if let Some(sel_room_name) = assets.room_names.get(sel_room_id) {
                                ui.selectable_value(dest_room_id, *sel_room_id, sel_room_name);
                            }
                        }
                    });
                ui.end_row();

                ui.label("To trigger:");
                ui.add(egui::DragValue::new(dest_trigger_id).speed(1.0).range(0..=u16::MAX));
                ui.end_row();
            }
        }
    }

    fn show_trigger_properties(
        &self,
        ui: &mut egui::Ui,
        trg_index: usize,
        room: &mut Room,
        asset_ids: &AssetIdCollection,
        assets: &RoomEditorAssetLists
    ) -> Option<()> {
        let trigger = room.triggers.get_mut(trg_index)?;
        let tree_node_id = ui.make_persistent_id(format!("editor_{}_trg_prop_tree", self.asset_id));
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), tree_node_id, true)
            .show_header(ui, |ui| {
                ui.add(egui::Button::image_and_text(get_trigger_image(trigger), "Trigger").frame(false));
            }).body(|ui| {
                egui::Grid::new(format!("editor_{}_trg_prop_grid", self.asset_id))
                    .num_columns(2)
                    .spacing([8.0, 2.0])
                    .show(ui, |ui| {
                        self.show_trigger_properties_grid(ui, trigger, asset_ids, assets);
                    });
            });
        None
    }

    fn show_item_properties(&self, ui: &mut egui::Ui, room: &mut Room, asset_ids: &AssetIdCollection, assets: &RoomEditorAssetLists) {
        match self.room_editor.get_selected_item() {
            RoomItemRef::None | RoomItemRef::Screen => {}
            RoomItemRef::Map(map_index) => { self.show_map_properties(ui, map_index, room, assets.maps); },
            RoomItemRef::Trigger(trg_index) => { self.show_trigger_properties(ui, trg_index, room, asset_ids, assets); },
        }
    }

    fn show_toolbar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top(format!("editor_panel_{}_toolbar", self.asset_id)).show(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.add_space(2.0);
                ui.spacing_mut().item_spacing = egui::Vec2::new(1.0, 0.0);
                if ui.add(egui::Button::image_and_text(IMAGES.lock, "Maps")
                          .selected(self.room_editor.lock_maps)
                          .frame_when_inactive(self.room_editor.lock_maps))
                    .on_hover_text("Lock maps in place").clicked() {
                        self.room_editor.lock_maps = ! self.room_editor.lock_maps;
                    }
                ui.add_space(2.0);
                ui.separator();
                ui.add_space(2.0);
                if ui.add(egui::Button::image(IMAGES.screen)
                          .selected(self.room_editor.show_screen)
                          .frame_when_inactive(self.room_editor.show_screen))
                    .on_hover_text("Show screen size").clicked() {
                        self.room_editor.show_screen = ! self.room_editor.show_screen;
                    }
                ui.spacing_mut().item_spacing = egui::Vec2::new(1.0, 0.0);
            });
            ui.add_space(0.0);  // don't remove this, it's necessary
        });
    }

    fn show_header(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, dialogs: &mut Dialogs,
                   room: &mut Room, _asset_ids: &AssetIdCollection, assets: &RoomEditorAssetLists) {
        egui::Panel::top(format!("editor_panel_{}_top", self.asset_id)).show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Room", |ui| {
                    if ui.add(menu_item(IMAGES.properties, " Properties...")).clicked() {
                        dialogs.properties_dialog.set_open(wc, room);
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.add(menu_item(IMAGES.map_data, " Select maps...")).clicked() {
                        dialogs.map_selection_dialog.set_open(wc, room, assets.maps);
                    }
                    if ui.add(menu_item(IMAGES.info, " Add trigger")).clicked() {
                        self.add_trigger(room);
                    }
                });
            });
        });
    }

    pub fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, dialogs: &mut Dialogs,
                room: &mut Room, asset_ids: &AssetIdCollection, assets: &RoomEditorAssetLists) {
        self.show_header(ui, wc, dialogs, room, asset_ids, assets);
        self.show_toolbar(ui);

        // properties panel:
        egui::Panel::right(format!("editor_panel_{}_properties", self.asset_id)).resizable(false).show(ui, |ui| {
            ui.add_space(5.0);
            let want_height = 70.0_f32.max(ui.available_height() / 2.0);
            ui.allocate_ui(egui::Vec2::new(300.0, want_height), |ui| {
                egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                    let (change_maps, sel_map) = self.show_map_tree(ui, room, assets.maps);
                    let (add_trigger, sel_trigger, rm_trigger) = self.show_trigger_tree(ui, room);
                    self.room_editor.clear_selected_item_changed();

                    if change_maps { dialogs.map_selection_dialog.set_open(wc, room, assets.maps); }
                    if add_trigger { self.add_trigger(room); }
                    if let Some(map_index) = sel_map {
                        self.room_editor.set_selected_item(RoomItemRef::Map(map_index), true);
                    }
                    if let Some(trg_index) = sel_trigger {
                        self.room_editor.set_selected_item(RoomItemRef::Trigger(trg_index), true);
                    }
                    if let Some(trg_index) = rm_trigger { self.remove_trigger(room, trg_index); }
                });
            });
            ui.separator();
            egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                self.show_item_properties(ui, room, asset_ids, assets);
            });
        });

        // body:
        egui::CentralPanel::default().show(ui, |ui| {
            self.room_editor.show(ui, wc, room, assets);
        });
    }
}
