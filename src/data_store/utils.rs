use super::{
    DataAssetId,
    DataAssetType,
    GenericAsset,
    DataAssetList,
};

pub fn sort_asset_ids_by_name<T: GenericAsset>(ids: &mut [DataAssetId], assets: &DataAssetList<T>) {
    ids.sort_by(|x, y| {
        if let Some(ax) = assets.get(x) {
            if let Some(ay) = assets.get(y) {
                ax.asset().name.cmp(&ay.asset().name)
            } else {
                std::cmp::Ordering::Greater
            }
        } else {
            if assets.get(y).is_some() {
                std::cmp::Ordering::Less
            } else {
                x.cmp(y)
            }
        }
    });
}

pub fn get_asset_type_upper_name(asset_type: DataAssetType) -> &'static str {
    match asset_type {
        DataAssetType::Sfx => { "SFX" }
        DataAssetType::MapData => { "MAP" }
        DataAssetType::Room => { "ROOM" }
        DataAssetType::Font => { "FONT" }
        DataAssetType::World => { "WORLD" }
        DataAssetType::Tileset => { "TILESET" }
        DataAssetType::Sprite => { "SPRITE" }
        DataAssetType::ModData => { "MOD" }
        DataAssetType::PropFont => { "PROP_FONT" }
        DataAssetType::PalSprite => { "PAL_SPRITE" }
        DataAssetType::SpriteAnimation => { "SPRITE_ANIMATION" }
        DataAssetType::TileAnimation => { "TILE_ANIMATION" }
        DataAssetType::PlayerChar => { "PLAYER_CHAR" }
        DataAssetType::Enemy => { "ENEMY" }
    }
}
