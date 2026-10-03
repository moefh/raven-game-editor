mod item_properties;
mod effect_properties;

use item_properties::ItemPropertiesDialog;
use effect_properties::EffectPropertiesDialog;

use super::{
    AppWindowBase,
    AppWindowAction,
};
use super::super::{
    WindowContext,
    EditorAction,
};

use crate::misc::IMAGES;
use crate::data_asset::{
    DataStoreItem,
    DataAssetStore,
};

use super::super::editors::DataTableType;

enum TableItemAction {
    None,
    Edit(usize),
    Remove(usize),
    MoveUp(usize),
    MoveDown(usize),
}

struct Dialogs {
    item_properties: ItemPropertiesDialog,
    effect_properties: EffectPropertiesDialog,
}

impl Dialogs {
    fn new() -> Self {
        Dialogs {
            item_properties: ItemPropertiesDialog::new(),
            effect_properties: EffectPropertiesDialog::new(),
        }
    }

    fn show(&mut self, wc: &mut WindowContext, store: &mut DataAssetStore) {
        self.item_properties.show(wc, store);
        self.effect_properties.show(wc, store);
    }
}

pub struct DataTablesWindow {
    pub base: AppWindowBase,
    table_type: DataTableType,
    dialogs: Dialogs,
}

impl DataTablesWindow {
    pub fn new(base: AppWindowBase) -> Self {
        DataTablesWindow {
            base,
            table_type: DataTableType::Upgrades,
            dialogs: Dialogs::new(),
        }
    }

    pub fn clear(&mut self) {
    }

    pub fn open(&mut self, ctx: &egui::Context) {
        self.base.open = true;
        if self.base.open {
            self.base.bring_to_top(ctx);
        }
    }

    fn show_items(
        ui: &mut egui::Ui,
        table_type: DataTableType,
        store: &mut DataAssetStore,
    ) -> TableItemAction {
        let mut action = TableItemAction::None;
        if let Some(table) = table_type.get_store_item_table(store) && ! table.items.is_empty() {
            for (index, item) in table.items.iter().enumerate()  {
                let can_move_up = index > 0;
                let can_move_down = index < table.items.len()-1;
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::image(IMAGES.trash)).on_hover_text(table_type.remove_item_label()).clicked() {
                        action = TableItemAction::Remove(index);
                    }
                    if ui.add(egui::Button::image(IMAGES.pen)).on_hover_text(table_type.edit_item_label()).clicked() {
                        action = TableItemAction::Edit(index);
                    }
                    if ui.add_enabled(can_move_up, egui::Button::image(IMAGES.arrow_up)).on_hover_text("Move Up").clicked() {
                        action = TableItemAction::MoveUp(index);
                    }
                    if ui.add_enabled(can_move_down, egui::Button::image(IMAGES.arrow_down)).on_hover_text("Move Down").clicked() {
                        action = TableItemAction::MoveDown(index);
                    }
                    ui.add(egui::Label::new(&item.name).truncate());
                });
            }
            ui.add_space(4.0);
        }
        action
    }

    fn show_item_table(
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        table_type: DataTableType,
        store: &mut DataAssetStore,
        dialogs: &mut Dialogs
    ) {
        egui::Panel::top("editor_item_table_top").show(ui, |ui| {
            ui.add_space(4.0);
            ui.label(table_type.title());
            ui.add_space(1.0);
        });
        egui::Panel::bottom("editor_item_table_bottom").show(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                let new_sprite_id = store.asset_ids.sprites.iter().next().copied();
                if ui.add(egui::Button::new(table_type.add_item_label())).clicked() &&
                    let Some(table) = table_type.get_store_item_table_mut(store) {
                        if let Some(new_sprite_id) = new_sprite_id {
                            table.items.push(DataStoreItem {
                                name: String::from(table_type.new_item_name()),
                                sprite_id: new_sprite_id,
                            });
                        } else {
                            wc.open_message_box("No Sprite Available", "You must create a sprite first!");
                        }
                    }
            });
        });
        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                match Self::show_items(ui, table_type, store) {
                    TableItemAction::Edit(index) => {
                        dialogs.item_properties.set_open(wc, table_type, index, store);
                    }
                    TableItemAction::Remove(index) => {
                        if let Some(table) = table_type.get_store_item_table_mut(store) {
                            table.items.remove(index);
                            wc.add_editor_action(EditorAction::DataTableItemRemoved {
                                table_type,
                                index,
                            });
                        }
                    }
                    TableItemAction::MoveUp(index) => {
                        if index > 0 && let Some(table) = table_type.get_store_item_table_mut(store) {
                            table.items.swap(index-1, index);
                            wc.add_editor_action(EditorAction::DataTableItemsSwapped {
                                table_type,
                                index1: index-1,
                                index2: index,
                            });
                        }
                    }
                    TableItemAction::MoveDown(index) => {
                        if let Some(table) = table_type.get_store_item_table_mut(store) && index+1 < table.items.len() {
                            table.items.swap(index, index+1);
                            wc.add_editor_action(EditorAction::DataTableItemsSwapped {
                                table_type,
                                index1: index,
                                index2: index+1,
                            });
                        }
                    }
                    TableItemAction::None => {}
                }
            });
        });
    }

    fn show_effects(
        ui: &mut egui::Ui,
        table_type: DataTableType,
        store: &mut DataAssetStore,
    ) -> TableItemAction {
        let mut action = TableItemAction::None;
        if let Some(table) = table_type.get_store_effect_table(store) && ! table.names.is_empty() {
            for (index, effect_name) in table.names.iter().enumerate()  {
                let can_move_up = index > 0;
                let can_move_down = index < table.names.len()-1;
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::image(IMAGES.trash)).on_hover_text(table_type.remove_item_label()).clicked() {
                        action = TableItemAction::Remove(index);
                    }
                    if ui.add(egui::Button::image(IMAGES.pen)).on_hover_text(table_type.edit_item_label()).clicked() {
                        action = TableItemAction::Edit(index);
                    }
                    if ui.add_enabled(can_move_up, egui::Button::image(IMAGES.arrow_up)).on_hover_text("Move Up").clicked() {
                        action = TableItemAction::MoveUp(index);
                    }
                    if ui.add_enabled(can_move_down, egui::Button::image(IMAGES.arrow_down)).on_hover_text("Move Down").clicked() {
                        action = TableItemAction::MoveDown(index);
                    }
                    ui.add(egui::Label::new(effect_name).truncate());
                });
            }
            ui.add_space(4.0);
        }
        action
    }

    fn show_effect_table(
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        table_type: DataTableType,
        store: &mut DataAssetStore,
        dialogs: &mut Dialogs
    ) {
        egui::Panel::top("editor_item_table_top").show(ui, |ui| {
            ui.add_space(4.0);
            ui.label(table_type.title());
            ui.add_space(1.0);
        });
        egui::Panel::bottom("editor_tables.effect_bottom").show(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new(table_type.add_item_label())).clicked() &&
                    let Some(table) = table_type.get_store_effect_table_mut(store) {
                        table.names.push(String::from(table_type.new_item_name()));
                    }
            });
        });
        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                match Self::show_effects(ui, table_type, store) {
                    TableItemAction::Edit(index) => {
                        dialogs.effect_properties.set_open(wc, table_type, index, store);
                    }
                    TableItemAction::Remove(index) => {
                        if let Some(table) = table_type.get_store_effect_table_mut(store) {
                            table.names.remove(index);
                            wc.add_editor_action(EditorAction::DataTableItemRemoved {
                                table_type,
                                index,
                            });
                        }
                    }
                    TableItemAction::MoveUp(index) => {
                        if index > 0 && let Some(table) = table_type.get_store_effect_table_mut(store) {
                            table.names.swap(index-1, index);
                            wc.add_editor_action(EditorAction::DataTableItemsSwapped {
                                table_type,
                                index1: index-1,
                                index2: index,
                            });
                        }
                    }
                    TableItemAction::MoveDown(index) => {
                        if let Some(table) = table_type.get_store_effect_table_mut(store) && index+1 < table.names.len() {
                            table.names.swap(index, index+1);
                            wc.add_editor_action(EditorAction::DataTableItemsSwapped {
                                table_type,
                                index1: index,
                                index2: index+1,
                            });
                        }
                    }
                    TableItemAction::None => {}
                }
            });
        });
    }

    pub fn show(&mut self, wc: &mut WindowContext, store: &mut DataAssetStore) -> AppWindowAction {
        self.dialogs.show(wc, store);

        let default_rect = self.base.default_rect(wc, 600.0, 300.0);
        self.base.show_window(wc, default_rect, [400.0, 200.0], |ui, wc, base| {
            let action = base.show_title_bar(ui, Some(IMAGES.table), "Items and Effects");
            egui::Panel::bottom("editor_tables_footer").show(ui, |ui| {
                let data_size = store.tables.upgrade.data_size() + store.tables.collectable.data_size() + store.tables.pickup.data_size();
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "[{} bytes] - {} upgrade{} / {} collectable{} / {} pickup{} / {} effect{} / {} flag{}",
                        data_size,
                        store.tables.upgrade.items.len(),
                        if store.tables.upgrade.items.len() != 1 { "s" } else { "" },
                        store.tables.collectable.items.len(),
                        if store.tables.collectable.items.len() != 1 { "s" } else { "" },
                        store.tables.pickup.items.len(),
                        if store.tables.pickup.items.len() != 1 { "s" } else { "" },
                        store.tables.effect.names.len(),
                        if store.tables.effect.names.len() != 1 { "s" } else { "" },
                        store.tables.flag.names.len(),
                        if store.tables.flag.names.len() != 1 { "s" } else { "" },
                    ));
                });
            });
            egui::Panel::left("project_table_types").resizable(false).exact_size(180.0).show(ui, |ui| {
                ui.add_space(4.0);
                egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                    for table_type in [
                        DataTableType::Upgrades,
                        DataTableType::Collectables,
                        DataTableType::Pickups,
                        DataTableType::Effects,
                        DataTableType::Flags,
                    ] {
                        let button = egui::Button::new(table_type.title())
                            .frame_when_inactive(self.table_type == table_type)
                            .gap(8.0);
                        if ui.add(button).clicked() {
                            self.table_type = table_type;
                        }
                    }
                    ui.add_space(2.0);
                });

            });
            if self.table_type.is_item_table() {
                Self::show_item_table(ui, wc, self.table_type, store, &mut self.dialogs);
            } else if self.table_type.is_effect_table() {
                Self::show_effect_table(ui, wc, self.table_type, store, &mut self.dialogs);
            }
            action
        })
    }
}
