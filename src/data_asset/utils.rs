
use super::{
    DataAssetId,
    GenericAsset,
    AssetList,
};

pub fn sort_asset_ids_by_name<T: GenericAsset>(ids: &mut [DataAssetId], assets: &AssetList<T>) {
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
