use super::{
    DataAssetId,
};

pub struct PlayerCharSuit {
    pub name: String,
    pub anim_id: DataAssetId,
    pub req_upgrade_id: u16,  // upgrade table item or 0xffff
}

impl PlayerCharSuit {
    pub fn new(name: impl Into<String>, anim_id: DataAssetId) -> Self {
        PlayerCharSuit {
            name: name.into(),
            anim_id,
            req_upgrade_id: 0xffff,
        }
    }
}

pub struct PlayerCharWeaponShoot {
    pub anim_id: DataAssetId,
    pub sfx_id: Option<DataAssetId>,
    pub anim_len: u8,  // in frames
    pub crouch_x: i8,
    pub crouch_y: i8,
}

impl PlayerCharWeaponShoot {
    fn new(anim_id: DataAssetId) -> Self {
        PlayerCharWeaponShoot {
            anim_id,
            sfx_id: None,
            anim_len: 60,
            crouch_x: 0,
            crouch_y: 0,
        }
    }
}

pub struct PlayerCharWeaponBullet {
    pub anim_id: DataAssetId,
    pub hit_anim_id: Option<DataAssetId>,
    pub hit_sfx_id: Option<DataAssetId>,
    pub ammo_pickup_id: u16,   // pickup table item or 0xffff
    pub initial_delay: u8,     // in frames
    pub interval: u8,          // in frames
    pub x: i8,                 // in pixels, relative to character collision rect
    pub y: i8,                 // in pixels, relative to character collision rect
    pub dx: i16,               // 8.8 fixpoint
    pub dy: i16,               // 8.8 fixpoint
    pub damage: u16,
}

impl PlayerCharWeaponBullet {
    fn new(anim_id: DataAssetId) -> Self {
        PlayerCharWeaponBullet {
            anim_id,
            hit_anim_id: None,
            hit_sfx_id: None,
            ammo_pickup_id: 0xffff,
            initial_delay: 0,
            interval: 0,
            x: 0,
            y: 0,
            dx: 0x0100,
            dy: 0x0000,
            damage: 1,
        }
    }
}

pub struct PlayerCharWeapon {
    pub name: String,
    pub req_collectable_id: u16,  // collectable table item or 0xffff
    pub shoot: PlayerCharWeaponShoot,
    pub bullet: PlayerCharWeaponBullet,
}

impl PlayerCharWeapon {
    pub fn new(name: impl Into<String>, shoot_anim_id: DataAssetId, bullet_anim_id: DataAssetId) -> Self {
        PlayerCharWeapon {
            name: name.into(),
            req_collectable_id: 0xffff,
            shoot: PlayerCharWeaponShoot::new(shoot_anim_id),
            bullet: PlayerCharWeaponBullet::new(bullet_anim_id),
        }
    }
}

pub struct PlayerChar {
    pub asset: super::DataAsset,
    pub suits: Vec<PlayerCharSuit>,
    pub weapons: Vec<PlayerCharWeapon>,
}

impl PlayerChar {
    pub fn new(id: super::DataAssetId, name: String) -> Self {
        PlayerChar {
            asset: super::DataAsset::new(super::DataAssetType::PlayerChar, id, name),
            suits: Vec::new(),
            weapons: Vec::new(),
        }
    }
}

impl super::HashedData for PlayerChar {
    fn data_hash(&self, hasher: &mut super::DataHasher) {
        hasher.add_data_asset(&self.asset);
        for suit in &self.suits {
            hasher.add_str(&suit.name);
            hasher.add_asset_id(suit.anim_id);
            hasher.add_u16(suit.req_upgrade_id);
        }

        for weapon in &self.weapons {
            hasher.add_str(&weapon.name);
            hasher.add_u16(weapon.req_collectable_id);

            hasher.add_asset_id(weapon.shoot.anim_id);
            if let Some(sfx_id) = weapon.shoot.sfx_id { hasher.add_asset_id(sfx_id); } else { hasher.add_u32(0); }
            hasher.add_u8(weapon.shoot.anim_len);
            hasher.add_i8(weapon.shoot.crouch_x);
            hasher.add_i8(weapon.shoot.crouch_y);

            hasher.add_asset_id(weapon.bullet.anim_id);
            if let Some(anim_id) = weapon.bullet.hit_anim_id { hasher.add_asset_id(anim_id); } else { hasher.add_u32(0); }
            if let Some(sfx_id) = weapon.bullet.hit_sfx_id { hasher.add_asset_id(sfx_id); } else { hasher.add_u32(0); }
            hasher.add_u8(weapon.bullet.initial_delay);
            hasher.add_u16(weapon.bullet.ammo_pickup_id);
            hasher.add_i8(weapon.bullet.x);
            hasher.add_i8(weapon.bullet.y);
            hasher.add_i16(weapon.bullet.dx);
            hasher.add_i16(weapon.bullet.dy);
            hasher.add_u16(weapon.bullet.damage);
        }
    }
}

impl super::DuplicableAsset<PlayerChar> for PlayerChar {
    fn duplicate(&self, dup_id: super::DataAssetId, dup_name: String) -> Self {
        PlayerChar {
            asset: self.asset.duplicate(dup_id, dup_name),
            suits: Vec::new(),
            weapons: Vec::new(),
        }
    }
}

impl super::GenericAsset for PlayerChar {
    fn asset(&self) -> &super::DataAsset { &self.asset }

    fn data_size(&self) -> usize {
        // header: num_suits(2) + num_weapons(2) + (suits, weapons) * pointer(4)
        let header = 2 + 2 + 4usize * 2;

        // suit: animation<ptr>(4) + req_upgrade_id(2) + pad(2)
        let suits = (4 + 2 + 2) * self.suits.len();

        // weapon:
        //   (bullet_anim, bullet_sfx, shoot_anim, shoot_sfx) * ptr(4) +
        //   req_collectable_id(2) + bullet_ammo_pickup_id(2) +
        //   crouch_x(1) + crouch_y(1) + bullet_x(1) + bullet_y(1) +
        //   bullet_dx(2) + bullet_dy(2) +
        //   bullet_damage(2) + bullet_initial_delay(1) + bullet_interval(1) +
        //   shoot_anim_len(1) + pad(3)
        let weapons = { (
            4usize * 4 +
                2 + 2 +
                1 + 1 + 1 + 1 +
                2 + 2 +
                2 + 1 + 1 +
                1 + 3
        ) * self.weapons.len() };

        header + suits + weapons
    }
}
