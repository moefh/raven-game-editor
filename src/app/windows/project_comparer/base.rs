use crate::data_store::{
    DataStore,
    DataStoreItemTable,
    DataStoreEffectTable,
};

use super::{
    WindowContext,
    ProjectComparer,
    DiffState,
    ItemTableDiff,
    EffectTableDiff,
};

pub fn show_item_table(
    ui: &mut egui::Ui,
    table_name: &str,
    table_diff: &ItemTableDiff,
    cur_table: &DataStoreItemTable,
    other_table: &DataStoreItemTable,
    cur_store: &DataStore,
    other_store: &DataStore,
) {
    if ! table_diff.has_differences() { return; }

    ui.label(format!("-> {}s", table_name));
    for &index in &table_diff.diffs {
        if let Some(cur) = cur_table.items.get(index) && let Some(other) = other_table.items.get(index) {
            if cur.name != other.name {
                ui.label(format!("   -> item[{}]: current name: '{}', other name: '{}'", index, cur.name, other.name));
            }
            let cur_s = cur_store.assets.sprites.get(&cur.sprite_id).map(|s| &s.asset.name);
            let other_s = other_store.assets.sprites.get(&other.sprite_id).map(|s| &s.asset.name);
            if let Some(cur_s) = cur_s && let Some(other_s) = other_s {
                ui.label(format!("   -> item[{}]: current sprite: '{}', other sprite: '{}'", index, cur_s, other_s));
            }
        } else {
            ui.label(format!("   -> item[{}]: <error reading items>", index));
        }
    }
    for index in table_diff.cur_only.iter() {
        if let Some(item) = cur_table.items.get(index) {
            ui.label(format!("   -> item[{}]: current only: '{}'", index, item.name));
        } else {
            ui.label(format!("   -> item[{}]: current only: <invalid item>", index));
        }
    }
    for index in table_diff.other_only.iter() {
        if let Some(item) = other_table.items.get(index) {
            ui.label(format!("   -> item[{}]: other only: '{}'", index, item.name));
        } else {
            ui.label(format!("   -> item[{}]: other only: <invalid item>", index));
        }
    }
}

pub fn show_effect_table(
    ui: &mut egui::Ui,
    table_name: &str,
    table_diff: &EffectTableDiff,
    cur_table: &DataStoreEffectTable,
    other_table: &DataStoreEffectTable,
) {
    if ! table_diff.has_differences() { return; }

    ui.label(format!("-> {}s", table_name));
    for &index in &table_diff.diffs {
        let cur_name = cur_table.names.get(index).map(|name| name.as_ref()).unwrap_or("(invalid)");
        let other_name = other_table.names.get(index).map(|name| name.as_ref()).unwrap_or("(invalid)");
        if cur_name != other_name {
            ui.label(format!("   -> item[{}]: current='{}', other='{}'", index, cur_name, other_name));
        }
    }
    for index in table_diff.cur_only.iter() {
        let name = cur_table.names.get(index).map(|name| name.as_ref()).unwrap_or("(invalid)");
        ui.label(format!("   -> item[{}]: current only: '{}'", index, name));
    }
    for index in table_diff.other_only.iter() {
        let name = other_table.names.get(index).map(|name| name.as_ref()).unwrap_or("(invalid)");
        ui.label(format!("   -> item[{}]: other only: '{}'", index, name));
    }
}

pub fn show_base(
    ui: &mut egui::Ui,
    _wc: &mut WindowContext,
    comparer: &ProjectComparer,
    cur_store: &DataStore,
    other_store: &DataStore
) -> DiffState {
    let mut diff_state = DiffState::EMPTY;

    if comparer.base_project.cur_data_size != comparer.base_project.other_data_size {
        ui.label(format!("Current project size: {}", comparer.base_project.cur_data_size));
        ui.label(format!("Other project size:   {}", comparer.base_project.other_data_size));
    } else {
        ui.label("Projects have the same size");
    }

    if comparer.base_project.has_table_differences() {
        diff_state = DiffState::CHANGED;
        diff_state.open_tables();
        ui.label("");
        ui.label("Differences in tables:");
        show_item_table(
            ui,
            "upgrade",
            &comparer.base_project.upgrade_table_diff,
            &cur_store.tables.upgrade,
            &other_store.tables.upgrade,
            cur_store,
            other_store
        );
        show_item_table(
            ui,
            "collectable",
            &comparer.base_project.collectable_table_diff,
            &cur_store.tables.collectable,
            &other_store.tables.collectable,
            cur_store,
            other_store
        );
        show_item_table(
            ui,
            "pickup",
            &comparer.base_project.pickup_table_diff,
            &cur_store.tables.pickup,
            &other_store.tables.pickup,
            cur_store,
            other_store
        );
        show_effect_table(
            ui,
            "effect",
            &comparer.base_project.effect_table_diff,
            &cur_store.tables.effect,
            &other_store.tables.effect
        );
        show_effect_table(
            ui,
            "flag",
            &comparer.base_project.flag_table_diff,
            &cur_store.tables.flag,
            &other_store.tables.flag
        );
    } else {
        ui.label("Projects have the same tables");
    }

    diff_state
}
