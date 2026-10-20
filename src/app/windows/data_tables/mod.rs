mod item_properties;
mod effect_properties;
mod confirmation;

use item_properties::ItemPropertiesDialog;
use effect_properties::EffectPropertiesDialog;
use confirmation::ConfirmationDialog;

use super::{
    AppWindowBase,
    AppWindowAction,
};
use super::super::{
    WindowContext,
    EditorAction,
    DialogResult,
};

use crate::misc::IMAGES;
use crate::data_asset::{
    DataStoreItem,
    DataAssetStore,
};

use super::super::editors::DataTableReference;

enum TableItemAction {
    None,
    Edit(usize),
    Remove(usize),
    MoveUp(usize),
    MoveDown(usize),
}

enum ConfirmationAction {
    None,
    RemoveTableItem { table_ref: DataTableReference, index: usize },
}

pub struct DataTablesWindow {
    pub base: AppWindowBase,
    dialogs: Dialogs,
    editor: Editor,
}

impl DataTablesWindow {
    pub fn new(base: AppWindowBase) -> Self {
        DataTablesWindow {
            base,
            dialogs: Dialogs::new(),
            editor: Editor::new(),
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

    pub fn show(&mut self, wc: &mut WindowContext, store: &mut DataAssetStore) -> AppWindowAction {
        self.dialogs.show(wc, store);

        let default_rect = self.base.default_rect(wc, 600.0, 300.0);
        self.base.show_window(wc, default_rect, [400.0, 200.0], |ui, wc, base| {
            let action = base.show_title_bar(ui, Some(IMAGES.table), "Items and Effects");
            self.editor.show(ui, wc, store, &mut self.dialogs);
            action
        })
    }
}

struct Dialogs {
    item_properties: ItemPropertiesDialog,
    effect_properties: EffectPropertiesDialog,
    confirmation: ConfirmationDialog,
    confirmation_result: DialogResult,
}

impl Dialogs {
    fn new() -> Self {
        Dialogs {
            item_properties: ItemPropertiesDialog::new(),
            effect_properties: EffectPropertiesDialog::new(),
            confirmation: ConfirmationDialog::new(),
            confirmation_result: DialogResult::None,
        }
    }

    fn show(&mut self, wc: &mut WindowContext, store: &mut DataAssetStore) {
        self.item_properties.show(wc, store);
        self.effect_properties.show(wc, store);

        if self.confirmation.open {
            self.confirmation_result = self.confirmation.show(wc);
        }
    }
}

struct Editor {
    table_ref: DataTableReference,
    confirmation_action: ConfirmationAction,
}

impl Editor {
    fn new() -> Self {
        Editor {
            table_ref: DataTableReference::Upgrades,
            confirmation_action: ConfirmationAction::None,
        }
    }

    fn show_items(
        &mut self,
        ui: &mut egui::Ui,
        store: &mut DataAssetStore,
    ) -> TableItemAction {
        let mut action = TableItemAction::None;
        if let Some(table) = self.table_ref.get_store_item_table(store) && ! table.items.is_empty() {
            for (index, item) in table.items.iter().enumerate()  {
                let can_move_up = index > 0;
                let can_move_down = index < table.items.len()-1;
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::image(IMAGES.trash)).on_hover_text(self.table_ref.remove_item_label()).clicked() {
                        action = TableItemAction::Remove(index);
                    }
                    if ui.add(egui::Button::image(IMAGES.pen)).on_hover_text(self.table_ref.edit_item_label()).clicked() {
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
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        store: &mut DataAssetStore,
        dialogs: &mut Dialogs
    ) {
        egui::Panel::top("editor_item_table_top").show(ui, |ui| {
            ui.add_space(4.0);
            ui.label(self.table_ref.title());
            ui.add_space(1.0);
        });
        egui::Panel::bottom("editor_item_table_bottom").show(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                let new_sprite_id = store.asset_ids.sprites.iter().next().copied();
                if ui.add(egui::Button::new(self.table_ref.add_item_label())).clicked() &&
                    let Some(table) = self.table_ref.get_store_item_table_mut(store) {
                        if let Some(new_sprite_id) = new_sprite_id {
                            table.items.push(DataStoreItem {
                                name: String::from(self.table_ref.new_item_name()),
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
                match self.show_items(ui, store) {
                    TableItemAction::Edit(index) => {
                        dialogs.item_properties.set_open(wc, self.table_ref, index, store);
                    }
                    TableItemAction::Remove(index) => {
                        if let Some(item_name) = self.table_ref.get_item_name(index, store) {
                            self.confirmation_action = ConfirmationAction::RemoveTableItem {
                                table_ref: self.table_ref,
                                index,
                            };
                            dialogs.confirmation.set_open(
                                wc,
                                "Remove Item",
                                format!("Remove this item?\n\n{}", item_name),
                                "Yes",
                                "No",
                            );
                        }
                    }
                    TableItemAction::MoveUp(index) => {
                        if index > 0 && let Some(table) = self.table_ref.get_store_item_table_mut(store) {
                            table.items.swap(index-1, index);
                            wc.add_editor_action(EditorAction::DataTableItemsSwapped {
                                table_ref: self.table_ref,
                                index1: index-1,
                                index2: index,
                            });
                        }
                    }
                    TableItemAction::MoveDown(index) => {
                        if let Some(table) = self.table_ref.get_store_item_table_mut(store) && index+1 < table.items.len() {
                            table.items.swap(index, index+1);
                            wc.add_editor_action(EditorAction::DataTableItemsSwapped {
                                table_ref: self.table_ref,
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
        &mut self,
        ui: &mut egui::Ui,
        store: &mut DataAssetStore,
    ) -> TableItemAction {
        let mut action = TableItemAction::None;
        if let Some(table) = self.table_ref.get_store_effect_table(store) && ! table.names.is_empty() {
            for (index, effect_name) in table.names.iter().enumerate()  {
                let can_move_up = index > 0;
                let can_move_down = index < table.names.len()-1;
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::image(IMAGES.trash)).on_hover_text(self.table_ref.remove_item_label()).clicked() {
                        action = TableItemAction::Remove(index);
                    }
                    if ui.add(egui::Button::image(IMAGES.pen)).on_hover_text(self.table_ref.edit_item_label()).clicked() {
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
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        store: &mut DataAssetStore,
        dialogs: &mut Dialogs
    ) {
        egui::Panel::top("editor_item_table_top").show(ui, |ui| {
            ui.add_space(4.0);
            ui.label(self.table_ref.title());
            ui.add_space(1.0);
        });
        egui::Panel::bottom("editor_tables.effect_bottom").show(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new(self.table_ref.add_item_label())).clicked() &&
                    let Some(table) = self.table_ref.get_store_effect_table_mut(store) {
                        table.names.push(String::from(self.table_ref.new_item_name()));
                    }
            });
        });
        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                match self.show_effects(ui, store) {
                    TableItemAction::Edit(index) => {
                        dialogs.effect_properties.set_open(wc, self.table_ref, index, store);
                    }
                    TableItemAction::Remove(index) => {
                        if let Some(item_name) = self.table_ref.get_item_name(index, store) {
                            self.confirmation_action = ConfirmationAction::RemoveTableItem {
                                table_ref: self.table_ref,
                                index,
                            };
                            dialogs.confirmation.set_open(
                                wc,
                                "Remove Item",
                                format!("Remove this item?\n\n{}", item_name),
                                "Yes",
                                "No",
                            );
                        }
                    }
                    TableItemAction::MoveUp(index) => {
                        if index > 0 && let Some(table) = self.table_ref.get_store_effect_table_mut(store) {
                            table.names.swap(index-1, index);
                            wc.add_editor_action(EditorAction::DataTableItemsSwapped {
                                table_ref: self.table_ref,
                                index1: index-1,
                                index2: index,
                            });
                        }
                    }
                    TableItemAction::MoveDown(index) => {
                        if let Some(table) = self.table_ref.get_store_effect_table_mut(store) && index+1 < table.names.len() {
                            table.names.swap(index, index+1);
                            wc.add_editor_action(EditorAction::DataTableItemsSwapped {
                                table_ref: self.table_ref,
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

    pub fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, store: &mut DataAssetStore, dialogs: &mut Dialogs) {
        if dialogs.confirmation_result == DialogResult::Yes &&
            let ConfirmationAction::RemoveTableItem { table_ref, index } = self.confirmation_action {
                self.table_ref.remove_item(index, store);
                wc.add_editor_action(EditorAction::DataTableItemRemoved {
                    table_ref,
                    index,
                });
            }
        if dialogs.confirmation_result != DialogResult::None {
            dialogs.confirmation_result = DialogResult::None;
            self.confirmation_action = ConfirmationAction::None;
        }

        egui::Panel::bottom("editor_tables_footer").show(ui, |ui| {
            let data_size = store.tables.upgrade.data_size() +
                store.tables.collectable.data_size() +
                store.tables.pickup.data_size();
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
        egui::Panel::left("project_table_refs").resizable(false).exact_size(180.0).show(ui, |ui| {
            ui.add_space(4.0);
            egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                for table_ref in [
                    DataTableReference::Upgrades,
                    DataTableReference::Collectables,
                    DataTableReference::Pickups,
                    DataTableReference::Effects,
                    DataTableReference::Flags,
                ] {
                    let button = egui::Button::new(table_ref.title())
                        .frame_when_inactive(self.table_ref == table_ref)
                        .gap(8.0);
                    if ui.add(button).clicked() {
                        self.table_ref = table_ref;
                    }
                }
                ui.add_space(2.0);
            });

        });
        if self.table_ref.is_item_table() {
            self.show_item_table(ui, wc, store, dialogs);
        } else if self.table_ref.is_effect_table() {
            self.show_effect_table(ui, wc, store, dialogs);
        }
    }
}
