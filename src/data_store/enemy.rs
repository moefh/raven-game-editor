use super::{
    DataAssetId,
};

#[derive(Copy, Clone, PartialEq)]
pub enum EnemyType {
    Walker,
    Chiller,
    Hopper,
    Floater,
    Other(u16),
}

impl EnemyType {
    pub fn value(self) -> u16 {
        match self {
            EnemyType::Walker   => { 0 }
            EnemyType::Chiller  => { 1 }
            EnemyType::Hopper   => { 2 }
            EnemyType::Floater  => { 3 }
            EnemyType::Other(n) => { n }
        }
    }
}

impl From<u16> for EnemyType {
    fn from(value: u16) -> Self {
        match value {
            0 => { EnemyType::Walker }
            1 => { EnemyType::Chiller }
            2 => { EnemyType::Hopper }
            3 => { EnemyType::Floater }
            n => { EnemyType::Other(n) }
        }
    }
}

pub struct Enemy {
    pub asset: super::DataAsset,
    pub anim_id: DataAssetId,
    pub enemy_type: EnemyType,
    pub health: u16,
}

impl Enemy {
    pub fn new(id: super::DataAssetId, name: String, anim_id: DataAssetId) -> Self {
        Enemy {
            asset: super::DataAsset::new(super::DataAssetType::Enemy, id, name),
            anim_id,
            enemy_type: EnemyType::Walker,
            health: 16,
        }
    }
}

impl super::HashedData for Enemy {
    fn data_hash(&self, hasher: &mut super::DataHasher) {
        hasher.add_data_asset(&self.asset);
        hasher.add_asset_id(self.anim_id);
        hasher.add_u16(self.enemy_type.value());
        hasher.add_u16(self.health);
    }
}

impl super::DuplicableAsset<Enemy> for Enemy {
    fn duplicate(&self, dup_id: super::DataAssetId, dup_name: String) -> Self {
        Enemy {
            asset: self.asset.duplicate(dup_id, dup_name),
            anim_id: self.anim_id,
            enemy_type: self.enemy_type,
            health: self.health,
        }
    }
}

impl super::GenericAsset for Enemy {
    fn asset(&self) -> &super::DataAsset { &self.asset }

    fn data_size(&self) -> usize {
        // sprite_animation<ptr>(4) + enemy_type(2) + health(2)
        4 + 2 + 2
    }
}
