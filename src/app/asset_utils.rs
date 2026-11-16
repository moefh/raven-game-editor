use crate::misc::asset_defs::{
    ASSET_DEFS,
};
use crate::data_store::{
    DataAssetType,
    DataAssetId,
    DataStore,
};
use crate::data_store::metadata::{
    DataStoreMetadata,
};

use super::{
    EditorStore,
    AppWindowTracker,
};

pub fn make_unique_asset_name(
    asset_type: DataAssetType,
    name: &str,
    force_number: bool,
    store: &DataStore,
) -> String {
    let mut num = 1;
    let mut unique_name = String::from(name);
    if force_number {
        unique_name.push_str(&format!("{}", num));
        num += 1;
    }
    loop {
        if ! store.asset_ids
            .ids_of_type(asset_type)
            .any(|&id| store.assets.get_asset(id).is_some_and(|a| a.name == unique_name)) {
                return unique_name;
            }
        unique_name.replace_range(.., &format!("{}{}", name, num));
        num += 1;
    }
}

fn new_asset_name(asset_type: DataAssetType, given_prefix: Option<String>, store: &DataStore) -> String {
    let name = if let Some(prefix) = ASSET_DEFS.iter().find(|def| def.asset_type == asset_type).map(|def| def.default_name_prefix) {
        match &given_prefix {
            Some(s) => { format!("{}/new_{}", s, prefix) }
            None => { format!("new_{}", prefix) }
        }
    } else {
        format!("{:?}", asset_type)
    };
    make_unique_asset_name(asset_type, &name, true, store)
}

pub fn remove_asset(
    id: DataAssetId,
    store: &mut DataStore,
    metadata: &mut DataStoreMetadata,
    editors: &mut EditorStore,
    window_tracker: &mut AppWindowTracker,
) {
    store.remove_asset(id, metadata);
    editors.remove_editor(id);
    window_tracker.remove_editor(id);
}

pub fn duplicate_asset(
    id: DataAssetId,
    asset_type: DataAssetType,
    store: &mut DataStore,
    metadata: &mut DataStoreMetadata,
    editors: &mut EditorStore,
    window_tracker: &mut AppWindowTracker,
) -> Result<DataAssetId, &'static str> {
    let dup_name = if let Some(asset) = store.assets.get_asset(id) {
        make_unique_asset_name(asset_type, &format!("{}_copy", asset.name), false, store)
    } else {
        new_asset_name(asset_type, None, store)
    };
    if let Some(dup_id) = store.duplicate_asset(id, &dup_name, metadata) {
        let egui_id = editors.add_asset(dup_id, asset_type);
        window_tracker.add_editor(egui_id, dup_id);
        Ok(dup_id)
    } else {
        Err("Couldn't duplicate asset.")
    }
}

pub fn add_asset(
    asset_type: DataAssetType,
    name_prefix: Option<String>,
    store: &mut DataStore,
    editors: &mut EditorStore,
    window_tracker: &mut AppWindowTracker,
) -> Result<DataAssetId, (&'static str, &'static str)> {
    let added = match asset_type {
        DataAssetType::Tileset => {
            store.add_tileset(new_asset_name(asset_type, name_prefix, store)).map(|id| (id, editors.add_tileset(id)))
        }
        DataAssetType::MapData => {
            if let Some(tileset_id) = store.asset_ids.tilesets.get_first() {
                store.add_map(new_asset_name(asset_type, name_prefix, store), tileset_id).map(|id| {
                    (id, editors.add_map(id))
                })
            } else {
                return Err(("No Tileset Available", "You must create a tileset first!"));
            }
        }
        DataAssetType::Room => {
            store.add_room(new_asset_name(asset_type, name_prefix, store)).map(|id| (id, editors.add_room(id)))
        }
        DataAssetType::World => {
            store.add_world(new_asset_name(asset_type, name_prefix, store)).map(|id| (id, editors.add_world(id)))
        }
        DataAssetType::Sprite => {
            store.add_sprite(new_asset_name(asset_type, name_prefix, store)).map(|id| (id, editors.add_sprite(id)))
        }
        DataAssetType::PalSprite => {
            store.add_pal_sprite(new_asset_name(asset_type, name_prefix, store)).map(|id| {
                (id, editors.add_pal_sprite(id))
            })
        }
        DataAssetType::SpriteAnimation => {
            if let Some(sprite_id) = store.asset_ids.sprites.get_first() {
                store.add_animation(new_asset_name(asset_type, name_prefix, store), sprite_id).map(|id| {
                    (id, editors.add_animation(id))
                })
            } else {
                return Err(("No Sprite Available", "You must create a sprite first!"));
            }
        }
        DataAssetType::TileAnimation => {
            if let Some(tileset_id) = store.asset_ids.tilesets.get_first() {
                store.add_tile_anim(new_asset_name(asset_type, name_prefix, store), tileset_id, tileset_id).map(|id| {
                    (id, editors.add_tile_anim(id))
                })
            } else {
                return Err(("No Tileset Available", "You must create a tileset first!"));
            }
        }
        DataAssetType::Sfx => {
            store.add_sfx(new_asset_name(asset_type, name_prefix, store)).map(|id| (id, editors.add_sfx(id)))
        }
        DataAssetType::ModData => {
            store.add_mod(new_asset_name(asset_type, name_prefix, store)).map(|id| (id, editors.add_mod(id)))
        }
        DataAssetType::Font => {
            store.add_font(new_asset_name(asset_type, name_prefix, store)).map(|id| (id, editors.add_font(id)))
        }
        DataAssetType::PropFont => {
            store.add_prop_font(new_asset_name(asset_type, name_prefix, store)).map(|id| {
                (id, editors.add_prop_font(id))
            })
        }
        DataAssetType::PlayerChar => {
            if let Some(font_id) = store.asset_ids.fonts.get_first() &&
                    let Some(gamepad_sprite_id) = store.asset_ids.sprites.get_first() {
                        store.add_player_char(
                            new_asset_name(asset_type, name_prefix, store),
                            font_id,
                            gamepad_sprite_id
                        ).map(|id| {
                            (id, editors.add_player_char(id))
                        })
                    } else {
                        return Err(("No Font Available", "You must create a font first!"));
                    }
        }
        DataAssetType::Enemy => {
            if let Some(anim_id) = store.asset_ids.animations.get_first() {
                store.add_enemy(new_asset_name(asset_type, name_prefix, store), anim_id).map(|id| {
                    (id, editors.add_enemy(id))
                })
            } else {
                return Err(("No Sprite Animation Available", "You must create a sprite animation first!"));
            }
        }
    };

    if let Some((asset_id, egui_id)) = added {
        window_tracker.add_editor(egui_id, asset_id);
        Ok(asset_id)
    } else {
        Err(("Error", "Error adding asset"))
    }
}
