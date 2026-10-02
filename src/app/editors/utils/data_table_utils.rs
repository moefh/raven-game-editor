use crate::data_asset::{
    DataAssetStore,
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
pub enum DataTableType {
    Upgrades,
    Collectables,
    Pickups,
    Effects,
    Flags,
}

impl DataTableType {
    pub fn is_item_table(self) -> bool {
        match self {
            DataTableType::Upgrades | DataTableType::Collectables | DataTableType::Pickups => { true }
            DataTableType::Effects | DataTableType::Flags => { false }
        }
    }

    pub fn is_effect_table(self) -> bool {
        match self {
            DataTableType::Upgrades | DataTableType::Collectables | DataTableType::Pickups => { false }
            DataTableType::Effects | DataTableType::Flags => { true }
        }
    }

    pub fn get_store_item_table(self, store: &DataAssetStore) -> Option<&DataStoreItemTable> {
        match self {
            DataTableType::Upgrades => { Some(&store.tables.upgrade) }
            DataTableType::Collectables => { Some(&store.tables.collectable) }
            DataTableType::Pickups => { Some(&store.tables.pickup) }
            DataTableType::Effects | DataTableType::Flags => { None }
        }
    }

    pub fn get_store_item_table_mut(self, store: &mut DataAssetStore) -> Option<&mut DataStoreItemTable> {
        match self {
            DataTableType::Upgrades => { Some(&mut store.tables.upgrade) }
            DataTableType::Collectables => { Some(&mut store.tables.collectable) }
            DataTableType::Pickups => { Some(&mut store.tables.pickup) }
            DataTableType::Effects | DataTableType::Flags => { None }
        }
    }

    pub fn get_store_item(self, store: &DataAssetStore, index: usize) -> Option<&DataStoreItem> {
        match self {
            DataTableType::Upgrades => { store.tables.upgrade.items.get(index) }
            DataTableType::Collectables => { store.tables.collectable.items.get(index) }
            DataTableType::Pickups => { store.tables.pickup.items.get(index) }
            DataTableType::Effects | DataTableType::Flags => { None }
        }
    }

    pub fn get_store_item_mut(self, store: &mut DataAssetStore, index: usize) -> Option<&mut DataStoreItem> {
        match self {
            DataTableType::Upgrades => { store.tables.upgrade.items.get_mut(index) }
            DataTableType::Collectables => { store.tables.collectable.items.get_mut(index) }
            DataTableType::Pickups => { store.tables.pickup.items.get_mut(index) }
            DataTableType::Effects | DataTableType::Flags => { None }
        }
    }

    pub fn get_store_effect_table(self, store: &DataAssetStore) -> Option<&DataStoreEffectTable> {
        match self {
            DataTableType::Upgrades => { None }
            DataTableType::Collectables => { None }
            DataTableType::Pickups => { None }
            DataTableType::Effects => { Some(&store.tables.effect) }
            DataTableType::Flags => { Some(&store.tables.flag) }
        }
    }

    pub fn get_store_effect_table_mut(self, store: &mut DataAssetStore) -> Option<&mut DataStoreEffectTable> {
        match self {
            DataTableType::Upgrades => { None }
            DataTableType::Collectables => { None }
            DataTableType::Pickups => { None }
            DataTableType::Effects => { Some(&mut store.tables.effect) }
            DataTableType::Flags => { Some(&mut store.tables.flag) }
        }
    }

    pub fn get_store_effect(self, store: &DataAssetStore, index: usize) -> Option<&String> {
        match self {
            DataTableType::Upgrades => { None }
            DataTableType::Collectables => { None }
            DataTableType::Pickups => { None }
            DataTableType::Effects => { store.tables.effect.names.get(index) }
            DataTableType::Flags => { store.tables.flag.names.get(index) }
        }
    }

    pub fn get_store_effect_mut(self, store: &mut DataAssetStore, index: usize) -> Option<&mut String> {
        match self {
            DataTableType::Upgrades => { None }
            DataTableType::Collectables => { None }
            DataTableType::Pickups => { None }
            DataTableType::Effects => { store.tables.effect.names.get_mut(index) }
            DataTableType::Flags => { store.tables.flag.names.get_mut(index) }
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            DataTableType::Upgrades => { "Upgrades" }
            DataTableType::Collectables => { "Collectables" }
            DataTableType::Pickups => { "Pickups" }
            DataTableType::Effects => { "Effects" }
            DataTableType::Flags => { "Flags" }
        }
    }

    pub fn new_item_name(self) -> &'static str {
        match self {
            DataTableType::Upgrades => { "new_upgrade" }
            DataTableType::Collectables => { "new_collectable" }
            DataTableType::Pickups => { "new_pickup" }
            DataTableType::Effects => { "new_effect" }
            DataTableType::Flags => { "new_flag" }
        }
    }

    pub fn add_item_label(self) -> &'static str {
        match self {
            DataTableType::Upgrades => { "Add Upgrade" }
            DataTableType::Collectables => { "Add Collectable" }
            DataTableType::Pickups => { "Add Pickup" }
            DataTableType::Effects => { "Add Effect" }
            DataTableType::Flags => { "Add Flag" }
        }
    }

    pub fn edit_item_label(self) -> &'static str {
        match self {
            DataTableType::Upgrades => { "Edit Upgrade" }
            DataTableType::Collectables => { "Edit Collectable" }
            DataTableType::Pickups => { "Edit Pickup" }
            DataTableType::Effects => { "Edit Effect" }
            DataTableType::Flags => { "Edit Flag" }
        }
    }

    pub fn remove_item_label(self) -> &'static str {
        match self {
            DataTableType::Upgrades => { "Remove Upgrade" }
            DataTableType::Collectables => { "Remove Collectable" }
            DataTableType::Pickups => { "Remove Pickup" }
            DataTableType::Effects => { "Remove Effect" }
            DataTableType::Flags => { "Remove Flag" }
        }
    }
}

fn fix_removed_effect(effect_id: &mut u16, table_type: DataTableType, index: usize) {
    if table_type == DataTableType::Effects {
        if *effect_id > index as u16 { *effect_id -= 1; }
        else if *effect_id == index as u16 { *effect_id = u16::MAX; }
    }
}

fn fix_removed_upgrade(upgrade_id: &mut u16, table_type: DataTableType, index: usize) {
    if table_type == DataTableType::Upgrades {
        if *upgrade_id > index as u16 { *upgrade_id -= 1; }
        else if *upgrade_id == index as u16 { *upgrade_id = u16::MAX; }
    }
}

fn fix_removed_collectable(collectable_id: &mut u16, table_type: DataTableType, index: usize) {
    if table_type == DataTableType::Collectables {
        if *collectable_id > index as u16 { *collectable_id -= 1; }
        else if *collectable_id == index as u16 { *collectable_id = u16::MAX; }
    }
}

fn fix_removed_pickup(pickup_id: &mut u16, table_type: DataTableType, index: usize) {
    if table_type == DataTableType::Pickups {
        if *pickup_id > index as u16 { *pickup_id -= 1; }
        else if *pickup_id == index as u16 { *pickup_id = u16::MAX; }
    }
}

pub fn fix_after_data_table_item_removed(
    _wc: &mut WindowContext,
    store: &mut DataAssetStore,
    _editors: &mut EditorStore,
    table_type: DataTableType,
    index: usize
) {
    for room in store.assets.rooms.iter_mut() {
        for trigger in room.triggers.iter_mut() {
            match &mut trigger.trigger_type {
                RoomTriggerType::WallButton { effect_id, required_collectable_id, .. } => {
                    fix_removed_effect(effect_id, table_type, index);
                    fix_removed_collectable(required_collectable_id, table_type, index);
                }
                RoomTriggerType::FloorButton { effect_id, .. } => {
                    fix_removed_effect(effect_id, table_type, index);
                }
                RoomTriggerType::UnblockEffect { effect_id, .. } => {
                    fix_removed_effect(effect_id, table_type, index);
                }
                RoomTriggerType::DisableAnimationEffect { effect_id, .. } => {
                    fix_removed_effect(effect_id, table_type, index);
                }
                RoomTriggerType::GetUpgrade { upgrade_id } => {
                    fix_removed_upgrade(upgrade_id, table_type, index);
                }
                RoomTriggerType::GetCollectable { collectable_id } => {
                    fix_removed_collectable(collectable_id, table_type, index);
                }
                RoomTriggerType::GetPickup { pickup_id } => {
                    fix_removed_pickup(pickup_id, table_type, index);
                }
                RoomTriggerType::Unknown {..} |
                RoomTriggerType::Door {..} |
                RoomTriggerType::Trap {..} |
                RoomTriggerType::PlayerSpawn {..} |
                RoomTriggerType::EnemySpawn {..} => {
                    // nothing to fix
                }
            }
        }
    }
}

fn fix_swapped_effect(effect_id: &mut u16, table_type: DataTableType, index1: usize, index2: usize) {
    if table_type == DataTableType::Effects {
        if *effect_id == index1 as u16 { *effect_id = index2 as u16; }
        else if *effect_id == index2 as u16 { *effect_id = index1 as u16; }
    }
}

fn fix_swapped_upgrade(upgrade_id: &mut u16, table_type: DataTableType, index1: usize, index2: usize) {
    if table_type == DataTableType::Upgrades {
        if *upgrade_id == index1 as u16 { *upgrade_id = index2 as u16; }
        else if *upgrade_id == index2 as u16 { *upgrade_id = index1 as u16; }
    }
}

fn fix_swapped_collectable(collectable_id: &mut u16, table_type: DataTableType, index1: usize, index2: usize) {
    if table_type == DataTableType::Collectables {
        if *collectable_id == index1 as u16 { *collectable_id = index2 as u16; }
        else if *collectable_id == index2 as u16 { *collectable_id = index1 as u16; }
    }
}

fn fix_swapped_pickup(pickup_id: &mut u16, table_type: DataTableType, index1: usize, index2: usize) {
    if table_type == DataTableType::Pickups {
        if *pickup_id == index1 as u16 { *pickup_id = index2 as u16; }
        else if *pickup_id == index2 as u16 { *pickup_id = index1 as u16; }
    }
}

pub fn fix_after_data_table_items_swapped(
    _wc: &mut WindowContext,
    store: &mut DataAssetStore,
    _editors: &mut EditorStore,
    table_type: DataTableType,
    index1: usize,
    index2: usize
) {
    for room in store.assets.rooms.iter_mut() {
        for trigger in room.triggers.iter_mut() {
            match &mut trigger.trigger_type {
                RoomTriggerType::WallButton { effect_id, required_collectable_id, .. } => {
                    fix_swapped_effect(effect_id, table_type, index1, index2);
                    fix_swapped_collectable(required_collectable_id, table_type, index1, index2);
                }
                RoomTriggerType::FloorButton { effect_id, .. } => {
                    fix_swapped_effect(effect_id, table_type, index1, index2);
                }
                RoomTriggerType::UnblockEffect { effect_id, .. } => {
                    fix_swapped_effect(effect_id, table_type, index1, index2);
                }
                RoomTriggerType::DisableAnimationEffect { effect_id, .. } => {
                    fix_swapped_effect(effect_id, table_type, index1, index2);
                }
                RoomTriggerType::GetUpgrade { upgrade_id } => {
                    fix_swapped_upgrade(upgrade_id, table_type, index1, index2);
                }
                RoomTriggerType::GetCollectable { collectable_id } => {
                    fix_swapped_collectable(collectable_id, table_type, index1, index2);
                }
                RoomTriggerType::GetPickup { pickup_id } => {
                    fix_swapped_pickup(pickup_id, table_type, index1, index2);
                }
                RoomTriggerType::Unknown {..} |
                RoomTriggerType::Door {..} |
                RoomTriggerType::Trap {..} |
                RoomTriggerType::PlayerSpawn {..} |
                RoomTriggerType::EnemySpawn {..} => {
                    // nothing to fix
                }
            }
        }
    }
}
