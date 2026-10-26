use crate::data_store::{
    DataStore,
    DataStoreItem,
    DataStoreItemTable,
    DataStoreEffectTable,
    RoomTriggerType,
};
use super::super::{
    WindowContext,
    EditorStore,
};

#[derive(Clone, Copy, PartialEq)]
pub enum DataTableReference {
    Upgrades,
    Collectables,
    Pickups,
    Effects,
    Flags,
}

impl DataTableReference {
    pub fn get_item_name(self, index: usize, store: &DataStore) -> Option<&str> {
        match self {
            DataTableReference::Upgrades => { store.tables.upgrade.items.get(index).map(|item| item.name.as_str()) }
            DataTableReference::Collectables => { store.tables.collectable.items.get(index).map(|item| item.name.as_str()) }
            DataTableReference::Pickups => { store.tables.pickup.items.get(index).map(|item| item.name.as_str()) }
            DataTableReference::Effects => { store.tables.effect.names.get(index).map(|name| name.as_str()) }
            DataTableReference::Flags => { store.tables.flag.names.get(index).map(|name| name.as_str()) }
        }
    }

    pub fn remove_item(self, index: usize, store: &mut DataStore) {
        match self {
            DataTableReference::Upgrades => { store.tables.upgrade.items.remove(index); }
            DataTableReference::Collectables => { store.tables.collectable.items.remove(index); }
            DataTableReference::Pickups => { store.tables.pickup.items.remove(index); }
            DataTableReference::Effects => { store.tables.effect.names.remove(index); }
            DataTableReference::Flags => { store.tables.flag.names.remove(index); }
        }
    }

    pub fn is_item_table(self) -> bool {
        match self {
            DataTableReference::Upgrades | DataTableReference::Collectables | DataTableReference::Pickups => { true }
            DataTableReference::Effects | DataTableReference::Flags => { false }
        }
    }

    pub fn is_effect_table(self) -> bool {
        match self {
            DataTableReference::Upgrades | DataTableReference::Collectables | DataTableReference::Pickups => { false }
            DataTableReference::Effects | DataTableReference::Flags => { true }
        }
    }

    pub fn get_store_item_table(self, store: &DataStore) -> Option<&DataStoreItemTable> {
        match self {
            DataTableReference::Upgrades => { Some(&store.tables.upgrade) }
            DataTableReference::Collectables => { Some(&store.tables.collectable) }
            DataTableReference::Pickups => { Some(&store.tables.pickup) }
            DataTableReference::Effects | DataTableReference::Flags => { None }
        }
    }

    pub fn get_store_item_table_mut(self, store: &mut DataStore) -> Option<&mut DataStoreItemTable> {
        match self {
            DataTableReference::Upgrades => { Some(&mut store.tables.upgrade) }
            DataTableReference::Collectables => { Some(&mut store.tables.collectable) }
            DataTableReference::Pickups => { Some(&mut store.tables.pickup) }
            DataTableReference::Effects | DataTableReference::Flags => { None }
        }
    }

    pub fn get_store_item(self, store: &DataStore, index: usize) -> Option<&DataStoreItem> {
        match self {
            DataTableReference::Upgrades => { store.tables.upgrade.items.get(index) }
            DataTableReference::Collectables => { store.tables.collectable.items.get(index) }
            DataTableReference::Pickups => { store.tables.pickup.items.get(index) }
            DataTableReference::Effects | DataTableReference::Flags => { None }
        }
    }

    pub fn get_store_item_mut(self, store: &mut DataStore, index: usize) -> Option<&mut DataStoreItem> {
        match self {
            DataTableReference::Upgrades => { store.tables.upgrade.items.get_mut(index) }
            DataTableReference::Collectables => { store.tables.collectable.items.get_mut(index) }
            DataTableReference::Pickups => { store.tables.pickup.items.get_mut(index) }
            DataTableReference::Effects | DataTableReference::Flags => { None }
        }
    }

    pub fn get_store_effect_table(self, store: &DataStore) -> Option<&DataStoreEffectTable> {
        match self {
            DataTableReference::Upgrades => { None }
            DataTableReference::Collectables => { None }
            DataTableReference::Pickups => { None }
            DataTableReference::Effects => { Some(&store.tables.effect) }
            DataTableReference::Flags => { Some(&store.tables.flag) }
        }
    }

    pub fn get_store_effect_table_mut(self, store: &mut DataStore) -> Option<&mut DataStoreEffectTable> {
        match self {
            DataTableReference::Upgrades => { None }
            DataTableReference::Collectables => { None }
            DataTableReference::Pickups => { None }
            DataTableReference::Effects => { Some(&mut store.tables.effect) }
            DataTableReference::Flags => { Some(&mut store.tables.flag) }
        }
    }

    pub fn get_store_effect(self, store: &DataStore, index: usize) -> Option<&String> {
        match self {
            DataTableReference::Upgrades => { None }
            DataTableReference::Collectables => { None }
            DataTableReference::Pickups => { None }
            DataTableReference::Effects => { store.tables.effect.names.get(index) }
            DataTableReference::Flags => { store.tables.flag.names.get(index) }
        }
    }

    pub fn get_store_effect_mut(self, store: &mut DataStore, index: usize) -> Option<&mut String> {
        match self {
            DataTableReference::Upgrades => { None }
            DataTableReference::Collectables => { None }
            DataTableReference::Pickups => { None }
            DataTableReference::Effects => { store.tables.effect.names.get_mut(index) }
            DataTableReference::Flags => { store.tables.flag.names.get_mut(index) }
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            DataTableReference::Upgrades => { "Upgrades" }
            DataTableReference::Collectables => { "Collectables" }
            DataTableReference::Pickups => { "Pickups" }
            DataTableReference::Effects => { "Effects" }
            DataTableReference::Flags => { "Flags" }
        }
    }

    pub fn new_item_name(self) -> &'static str {
        match self {
            DataTableReference::Upgrades => { "new_upgrade" }
            DataTableReference::Collectables => { "new_collectable" }
            DataTableReference::Pickups => { "new_pickup" }
            DataTableReference::Effects => { "new_effect" }
            DataTableReference::Flags => { "new_flag" }
        }
    }

    pub fn add_item_label(self) -> &'static str {
        match self {
            DataTableReference::Upgrades => { "Add Upgrade" }
            DataTableReference::Collectables => { "Add Collectable" }
            DataTableReference::Pickups => { "Add Pickup" }
            DataTableReference::Effects => { "Add Effect" }
            DataTableReference::Flags => { "Add Flag" }
        }
    }

    pub fn edit_item_label(self) -> &'static str {
        match self {
            DataTableReference::Upgrades => { "Edit Upgrade" }
            DataTableReference::Collectables => { "Edit Collectable" }
            DataTableReference::Pickups => { "Edit Pickup" }
            DataTableReference::Effects => { "Edit Effect" }
            DataTableReference::Flags => { "Edit Flag" }
        }
    }

    pub fn remove_item_label(self) -> &'static str {
        match self {
            DataTableReference::Upgrades => { "Remove Upgrade" }
            DataTableReference::Collectables => { "Remove Collectable" }
            DataTableReference::Pickups => { "Remove Pickup" }
            DataTableReference::Effects => { "Remove Effect" }
            DataTableReference::Flags => { "Remove Flag" }
        }
    }
}

fn fix_removed_effect(effect_id: &mut u16, data_table_ref: DataTableReference, index: usize) {
    if data_table_ref == DataTableReference::Effects {
        if *effect_id > index as u16 { *effect_id -= 1; }
        else if *effect_id == index as u16 { *effect_id = u16::MAX; }
    }
}

fn fix_removed_flag(flag_id: &mut u16, data_table_ref: DataTableReference, index: usize) {
    if data_table_ref == DataTableReference::Flags {
        if *flag_id > index as u16 { *flag_id -= 1; }
        else if *flag_id == index as u16 { *flag_id = u16::MAX; }
    }
}

fn fix_removed_upgrade(upgrade_id: &mut u16, data_table_ref: DataTableReference, index: usize) {
    if data_table_ref == DataTableReference::Upgrades {
        if *upgrade_id > index as u16 { *upgrade_id -= 1; }
        else if *upgrade_id == index as u16 { *upgrade_id = u16::MAX; }
    }
}

fn fix_removed_collectable(collectable_id: &mut u16, data_table_ref: DataTableReference, index: usize) {
    if data_table_ref == DataTableReference::Collectables {
        if *collectable_id > index as u16 { *collectable_id -= 1; }
        else if *collectable_id == index as u16 { *collectable_id = u16::MAX; }
    }
}

fn fix_removed_pickup(pickup_id: &mut u16, data_table_ref: DataTableReference, index: usize) {
    if data_table_ref == DataTableReference::Pickups {
        if *pickup_id > index as u16 { *pickup_id -= 1; }
        else if *pickup_id == index as u16 { *pickup_id = u16::MAX; }
    }
}

pub fn fix_after_data_table_item_removed(
    _wc: &mut WindowContext,
    store: &mut DataStore,
    _editors: &mut EditorStore,
    data_table_ref: DataTableReference,
    index: usize
) {
    for room in store.assets.rooms.iter_mut() {
        for trigger in room.triggers.iter_mut() {
            match &mut trigger.trigger_type {
                RoomTriggerType::WallButton { set_effect_id, set_flag_id, req_flag_id, req_collectable_id, .. } => {
                    fix_removed_effect(set_effect_id, data_table_ref, index);
                    fix_removed_flag(set_flag_id, data_table_ref, index);
                    fix_removed_flag(req_flag_id, data_table_ref, index);
                    fix_removed_collectable(req_collectable_id, data_table_ref, index);
                }
                RoomTriggerType::DetectorButton { set_effect_id, set_flag_id, req_flag_id, req_collectable_id, .. } => {
                    fix_removed_effect(set_effect_id, data_table_ref, index);
                    fix_removed_flag(set_flag_id, data_table_ref, index);
                    fix_removed_flag(req_flag_id, data_table_ref, index);
                    fix_removed_collectable(req_collectable_id, data_table_ref, index);
                }
                RoomTriggerType::UnblockTiles { on_effect_id, .. } => {
                    fix_removed_effect(on_effect_id, data_table_ref, index);
                }
                RoomTriggerType::DisableAnimation { on_effect_id, .. } => {
                    fix_removed_effect(on_effect_id, data_table_ref, index);
                }
                RoomTriggerType::ChangeTiles { on_effect_id, .. } => {
                    fix_removed_effect(on_effect_id, data_table_ref, index);
                }
                RoomTriggerType::RemoveEnemy { on_effect_id, .. } => {
                    fix_removed_effect(on_effect_id, data_table_ref, index);
                }
                RoomTriggerType::GetUpgrade { get_upgrade_id } => {
                    fix_removed_upgrade(get_upgrade_id, data_table_ref, index);
                }
                RoomTriggerType::GetCollectable { get_collectable_id } => {
                    fix_removed_collectable(get_collectable_id, data_table_ref, index);
                }
                RoomTriggerType::GetPickup { get_pickup_id, .. } => {
                    fix_removed_pickup(get_pickup_id, data_table_ref, index);
                }
                RoomTriggerType::Door { req_flag_id, .. } => {
                    fix_removed_flag(req_flag_id, data_table_ref, index);
                }
                RoomTriggerType::EnemySpawn { req_flag_id, .. } => {
                    fix_removed_flag(req_flag_id, data_table_ref, index);
                }
                RoomTriggerType::Unknown {..} | RoomTriggerType::PlayerSpawn {..} => {
                    // nothing to fix
                }
            }
        }
    }
}

fn fix_swapped_effect(effect_id: &mut u16, data_table_ref: DataTableReference, index1: usize, index2: usize) {
    if data_table_ref == DataTableReference::Effects {
        if *effect_id == index1 as u16 { *effect_id = index2 as u16; }
        else if *effect_id == index2 as u16 { *effect_id = index1 as u16; }
    }
}

fn fix_swapped_flag(flag_id: &mut u16, data_table_ref: DataTableReference, index1: usize, index2: usize) {
    if data_table_ref == DataTableReference::Flags {
        if *flag_id == index1 as u16 { *flag_id = index2 as u16; }
        else if *flag_id == index2 as u16 { *flag_id = index1 as u16; }
    }
}

fn fix_swapped_upgrade(upgrade_id: &mut u16, data_table_ref: DataTableReference, index1: usize, index2: usize) {
    if data_table_ref == DataTableReference::Upgrades {
        if *upgrade_id == index1 as u16 { *upgrade_id = index2 as u16; }
        else if *upgrade_id == index2 as u16 { *upgrade_id = index1 as u16; }
    }
}

fn fix_swapped_collectable(collectable_id: &mut u16, data_table_ref: DataTableReference, index1: usize, index2: usize) {
    if data_table_ref == DataTableReference::Collectables {
        if *collectable_id == index1 as u16 { *collectable_id = index2 as u16; }
        else if *collectable_id == index2 as u16 { *collectable_id = index1 as u16; }
    }
}

fn fix_swapped_pickup(pickup_id: &mut u16, data_table_ref: DataTableReference, index1: usize, index2: usize) {
    if data_table_ref == DataTableReference::Pickups {
        if *pickup_id == index1 as u16 { *pickup_id = index2 as u16; }
        else if *pickup_id == index2 as u16 { *pickup_id = index1 as u16; }
    }
}

pub fn fix_after_data_table_items_swapped(
    _wc: &mut WindowContext,
    store: &mut DataStore,
    _editors: &mut EditorStore,
    data_table_ref: DataTableReference,
    index1: usize,
    index2: usize
) {
    for room in store.assets.rooms.iter_mut() {
        for trigger in room.triggers.iter_mut() {
            match &mut trigger.trigger_type {
                RoomTriggerType::WallButton { set_effect_id, set_flag_id, req_flag_id, req_collectable_id, .. } => {
                    fix_swapped_effect(set_effect_id, data_table_ref, index1, index2);
                    fix_swapped_flag(set_flag_id, data_table_ref, index1, index2);
                    fix_swapped_flag(req_flag_id, data_table_ref, index1, index2);
                    fix_swapped_collectable(req_collectable_id, data_table_ref, index1, index2);
                }
                RoomTriggerType::DetectorButton { set_effect_id, set_flag_id, req_flag_id, req_collectable_id, .. } => {
                    fix_swapped_effect(set_effect_id, data_table_ref, index1, index2);
                    fix_swapped_flag(set_flag_id, data_table_ref, index1, index2);
                    fix_swapped_flag(req_flag_id, data_table_ref, index1, index2);
                    fix_swapped_collectable(req_collectable_id, data_table_ref, index1, index2);
                }
                RoomTriggerType::UnblockTiles { on_effect_id, .. } => {
                    fix_swapped_effect(on_effect_id, data_table_ref, index1, index2);
                }
                RoomTriggerType::DisableAnimation { on_effect_id, .. } => {
                    fix_swapped_effect(on_effect_id, data_table_ref, index1, index2);
                }
                RoomTriggerType::ChangeTiles { on_effect_id, .. } => {
                    fix_swapped_effect(on_effect_id, data_table_ref, index1, index2);
                }
                RoomTriggerType::RemoveEnemy { on_effect_id, .. } => {
                    fix_swapped_effect(on_effect_id, data_table_ref, index1, index2);
                }
                RoomTriggerType::GetUpgrade { get_upgrade_id } => {
                    fix_swapped_upgrade(get_upgrade_id, data_table_ref, index1, index2);
                }
                RoomTriggerType::GetCollectable { get_collectable_id } => {
                    fix_swapped_collectable(get_collectable_id, data_table_ref, index1, index2);
                }
                RoomTriggerType::GetPickup { get_pickup_id, .. } => {
                    fix_swapped_pickup(get_pickup_id, data_table_ref, index1, index2);
                }
                RoomTriggerType::Door { req_flag_id, ..} => {
                    fix_swapped_flag(req_flag_id, data_table_ref, index1, index2);
                }
                RoomTriggerType::EnemySpawn { req_flag_id, ..} => {
                    fix_swapped_flag(req_flag_id, data_table_ref, index1, index2);
                }
                RoomTriggerType::Unknown {..} | RoomTriggerType::PlayerSpawn {..} => {
                    // nothing to fix
                }
            }
        }
    }
}
