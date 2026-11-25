use super::DataAssetId;

#[derive(Copy, Clone, PartialEq)]
pub enum RoomEntityDirection {
    Right,
    Left,
}

impl RoomEntityDirection {
    pub fn value(self) -> u8 {
        match self {
            RoomEntityDirection::Right => { 0 }
            RoomEntityDirection::Left => { 1 }
        }
    }
}

impl From<u8> for RoomEntityDirection {
    fn from(value: u8) -> Self {
        if value == 0 {
            RoomEntityDirection::Right
        } else {
            RoomEntityDirection::Left
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum RoomTriggerMapLayer {
    Background2,
    Background,
    Foreground,
}

impl RoomTriggerMapLayer {
    pub fn value(self) -> u8 {
        match self {
            RoomTriggerMapLayer::Background2 => { 0 }
            RoomTriggerMapLayer::Background => { 1 }
            RoomTriggerMapLayer::Foreground => { 2 }
        }
    }
}

impl From<u8> for RoomTriggerMapLayer {
    fn from(value: u8) -> Self {
        match value {
            0 => { RoomTriggerMapLayer::Background2 }
            1 => { RoomTriggerMapLayer::Background }
            _ => { RoomTriggerMapLayer::Foreground }
        }
    }
}

#[derive(Clone)]
pub struct RoomMap {
    pub x: u16,
    pub y: u16,
    pub map_id: DataAssetId,
}

#[derive(Clone)]
pub enum RoomTriggerType {
    Unknown {
        data0: u16,
        data1: u16,
        data2: u16,
        data3: u16,
        data4: u16,
        data5: u16,
        data6: u16,
        data7: u16,
    },
    PlayerSpawn {
        direction: RoomEntityDirection,
    },
    EnemySpawn {
        enemy_id: DataAssetId,
        enemy_num: u16,
        direction: RoomEntityDirection,
        req_flag_id: u16,
    },
    Door {
        dest_room_id: DataAssetId,
        dest_trigger_id: u16,
        req_flag_id: u16,
    },
    WallButton {
        width: u16,
        height: u16,
        interact_prompt_id: u16,
        set_effect_id: u16,
        set_flag_id: u16,
        req_flag_id: u16,
        req_collectable_id: u16,
    },
    DetectorButton {
        width: u16,
        height: u16,
        set_effect_id: u16,
        set_flag_id: u16,
        req_flag_id: u16,
        req_collectable_id: u16,
    },
    UnblockTiles {
        width: u16,
        height: u16,
        on_effect_id: u16,
    },
    DisableAnimation {
        width: u16,
        height: u16,
        on_effect_id: u16,
    },
    ChangeTiles {
        map_index: u16,
        tile_change_index: u16,
        map_layer: RoomTriggerMapLayer,
        on_effect_id: u16,
    },
    RemoveEnemy {
        enemy_num: u16,
        on_effect_id: u16,
    },
    PlayAnimation {
        anim_id: DataAssetId,
        on_effect_id: u16,
        set_effect_id: u16,
        set_flag_id: u16,
    },
    PlayCutscene {
        cutscene_index: u16,
        on_effect_id: u16,
        req_flag_id: u16,
        deny_flag_id: u16,
    },
    PlaySfx {
        sfx_id: DataAssetId,
        sound_id: u16,
        volume: u16,
        pitch_shift: u16,
        play_loop: u8,
        on_effect_id: u16,
    },
    StopSfx {
        sound_id: u16,
        on_effect_id: u16,
    },
    GetUpgrade {
        get_upgrade_id: u16,
        req_flag_id: u16,
    },
    GetCollectable {
        get_collectable_id: u16,
        interact_prompt_id: u16,
        req_flag_id: u16,
    },
    GetPickup {
        get_pickup_id: u16,
        respawn_delay: u16,
        sfx_id: Option<DataAssetId>,
    },
    OnLoadRoom {
        set_effect_id: u16,
        set_flag_id: u16,
        req_flag_id: u16,
        deny_flag_id: u16,
    },
}

#[derive(Clone)]
pub struct RoomTrigger {
    pub trigger_id: u16,
    pub name_id: String,
    pub x: i16,
    pub y: i16,
    pub trigger_type: RoomTriggerType,
}

#[derive(Clone)]
pub struct RoomParallax {
    pub tileset_id: Option<DataAssetId>,
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<u8>,
}

#[derive(Clone)]
pub enum RoomCutsceneActionType {
    Unknown { data0: u16, data1: u16, data2: u16, data3: u16, data4: u16, data5: u16 },
    WaitFrames { num_frames: u16 },
    WaitPlayerAnimationEnd {},
    DispelDarkness { speed: u16 },
    SetPlayerDarkness { radius: u16, fall_off: u8, off_x: i8, off_y: i8 },
    MovePlayerDarkness { off_x: i8, off_y: i8, speed: u16 },
    SetPlayerState { state: u8 },
    ActivateEffect { set_effect_id: u16 },
    SetHudEnabled { enabled: u8 },
}

#[derive(Clone)]
pub struct RoomCutsceneAction {
    pub action_type: RoomCutsceneActionType,
}

#[derive(Clone)]
pub struct RoomCutscene {
    pub name_id: String,
    pub actions: Vec<RoomCutsceneAction>,
}

pub struct Room {
    pub asset: super::DataAsset,
    pub maps: Vec<RoomMap>,
    pub triggers: Vec<RoomTrigger>,
    pub cutscenes: Vec<RoomCutscene>,
    pub has_script: bool,
    pub parallax: RoomParallax,
}

impl Room {
    pub fn new(id: DataAssetId, name: String) -> Self {
        Room {
            asset: super::DataAsset::new(super::DataAssetType::Room, id, name),
            maps: Vec::new(),
            triggers: Vec::new(),
            cutscenes: Vec::new(),
            has_script: false,
            parallax: RoomParallax {
                tileset_id: None,
                width: 0,
                height: 0,
                tiles: Vec::new(),
            },
        }
    }
}

impl super::HashedData for Room {
    fn data_hash(&self, hasher: &mut super::DataHasher) {
        hasher.add_data_asset(&self.asset);
        if let Some(tileset_id) = self.parallax.tileset_id {
            hasher.add_u8(1);
            hasher.add_asset_id(tileset_id);
            hasher.add_u32(self.parallax.width);
            hasher.add_u32(self.parallax.height);
            hasher.add_bytes(&self.parallax.tiles);
        } else {
            hasher.add_u8(0);
        }
        for map in &self.maps {
            hasher.add_asset_id(map.map_id);
            hasher.add_u16(map.x);
            hasher.add_u16(map.y);
        }
        for cutscene in &self.cutscenes {
            hasher.add_str(&cutscene.name_id);
            for action in &cutscene.actions {
                match action.action_type {
                    RoomCutsceneActionType::Unknown { data0, data1, data2, data3, data4, data5 } => {
                        hasher.add_u8(0);
                        hasher.add_u16(data0);
                        hasher.add_u16(data1);
                        hasher.add_u16(data2);
                        hasher.add_u16(data3);
                        hasher.add_u16(data4);
                        hasher.add_u16(data5);
                    }
                    RoomCutsceneActionType::WaitFrames { num_frames } => {
                        hasher.add_u8(1);
                        hasher.add_u16(num_frames);
                    }
                    RoomCutsceneActionType::WaitPlayerAnimationEnd {} => {
                        hasher.add_u8(2);
                    }
                    RoomCutsceneActionType::DispelDarkness { speed } => {
                        hasher.add_u8(3);
                        hasher.add_u16(speed);
                    }
                    RoomCutsceneActionType::SetPlayerDarkness { radius, fall_off, off_x, off_y } => {
                        hasher.add_u8(4);
                        hasher.add_u16(radius);
                        hasher.add_u8(fall_off);
                        hasher.add_i8(off_x);
                        hasher.add_i8(off_y);
                    }
                    RoomCutsceneActionType::MovePlayerDarkness { off_x, off_y, speed } => {
                        hasher.add_u8(5);
                        hasher.add_i8(off_x);
                        hasher.add_i8(off_y);
                        hasher.add_u16(speed);
                    }
                    RoomCutsceneActionType::SetPlayerState { state } => {
                        hasher.add_u8(6);
                        hasher.add_u8(state);
                    }
                    RoomCutsceneActionType::ActivateEffect { set_effect_id } => {
                        hasher.add_u8(7);
                        hasher.add_u16(set_effect_id);
                    }
                    RoomCutsceneActionType::SetHudEnabled { enabled } => {
                        hasher.add_u8(8);
                        hasher.add_u8(enabled);
                    }
                }
            }
        }
        for trigger in &self.triggers {
            hasher.add_u16(trigger.trigger_id);
            hasher.add_str(&trigger.name_id);
            hasher.add_i16(trigger.x);
            hasher.add_i16(trigger.y);
            match trigger.trigger_type {
                RoomTriggerType::Unknown { data0, data1, data2, data3, data4, data5, data6, data7 } => {
                    hasher.add_u8(0);
                    hasher.add_u16(data0);
                    hasher.add_u16(data1);
                    hasher.add_u16(data2);
                    hasher.add_u16(data3);
                    hasher.add_u16(data4);
                    hasher.add_u16(data5);
                    hasher.add_u16(data6);
                    hasher.add_u16(data7);
                }
                RoomTriggerType::PlayerSpawn { direction } => {
                    hasher.add_u8(1);
                    hasher.add_u8(direction.value());
                }
                RoomTriggerType::EnemySpawn { enemy_id, enemy_num, direction, req_flag_id } => {
                    hasher.add_u8(2);
                    hasher.add_asset_id(enemy_id);
                    hasher.add_u16(enemy_num);
                    hasher.add_u8(direction.value());
                    hasher.add_u16(req_flag_id);
                }
                RoomTriggerType::Door { dest_room_id, dest_trigger_id, req_flag_id } => {
                    hasher.add_u8(3);
                    hasher.add_asset_id(dest_room_id);
                    hasher.add_u16(dest_trigger_id);
                    hasher.add_u16(req_flag_id);
                }
                RoomTriggerType::WallButton {
                    width,
                    height,
                    interact_prompt_id,
                    set_effect_id,
                    set_flag_id,
                    req_flag_id,
                    req_collectable_id,
                } => {
                    hasher.add_u8(5);
                    hasher.add_u16(width);
                    hasher.add_u16(height);
                    hasher.add_u16(interact_prompt_id);
                    hasher.add_u16(set_effect_id);
                    hasher.add_u16(set_flag_id);
                    hasher.add_u16(req_flag_id);
                    hasher.add_u16(req_collectable_id);
                }
                RoomTriggerType::DetectorButton { width, height, set_effect_id, set_flag_id, req_flag_id, req_collectable_id } => {
                    hasher.add_u8(6);
                    hasher.add_u16(width);
                    hasher.add_u16(height);
                    hasher.add_u16(set_effect_id);
                    hasher.add_u16(set_flag_id);
                    hasher.add_u16(req_flag_id);
                    hasher.add_u16(req_collectable_id);
                }
                RoomTriggerType::UnblockTiles { width, height, on_effect_id } => {
                    hasher.add_u8(7);
                    hasher.add_u16(width);
                    hasher.add_u16(height);
                    hasher.add_u16(on_effect_id);
                }
                RoomTriggerType::DisableAnimation { width, height, on_effect_id } => {
                    hasher.add_u8(8);
                    hasher.add_u16(width);
                    hasher.add_u16(height);
                    hasher.add_u16(on_effect_id);
                }
                RoomTriggerType::ChangeTiles { map_index, tile_change_index, map_layer, on_effect_id } => {
                    hasher.add_u8(9);
                    hasher.add_u16(map_index);
                    hasher.add_u16(tile_change_index);
                    hasher.add_u8(map_layer.value());
                    hasher.add_u16(on_effect_id);
                }
                RoomTriggerType::RemoveEnemy { enemy_num, on_effect_id } => {
                    hasher.add_u8(10);
                    hasher.add_u16(enemy_num);
                    hasher.add_u16(on_effect_id);
                }
                RoomTriggerType::PlayAnimation { anim_id, on_effect_id, set_effect_id, set_flag_id } => {
                    hasher.add_u8(11);
                    hasher.add_asset_id(anim_id);
                    hasher.add_u16(on_effect_id);
                    hasher.add_u16(set_effect_id);
                    hasher.add_u16(set_flag_id);
                }
                RoomTriggerType::PlayCutscene { cutscene_index, on_effect_id, req_flag_id, deny_flag_id } => {
                    hasher.add_u8(12);
                    hasher.add_u16(cutscene_index);
                    hasher.add_u16(on_effect_id);
                    hasher.add_u16(req_flag_id);
                    hasher.add_u16(deny_flag_id);
                }
                RoomTriggerType::PlaySfx { sfx_id, sound_id, volume, pitch_shift, play_loop, on_effect_id } => {
                    hasher.add_u8(13);
                    hasher.add_asset_id(sfx_id);
                    hasher.add_u16(sound_id);
                    hasher.add_u16(volume);
                    hasher.add_u16(pitch_shift);
                    hasher.add_u8(play_loop);
                    hasher.add_u16(on_effect_id);
                }
                RoomTriggerType::StopSfx { sound_id, on_effect_id } => {
                    hasher.add_u8(14);
                    hasher.add_u16(sound_id);
                    hasher.add_u16(on_effect_id);
                }
                RoomTriggerType::GetUpgrade { get_upgrade_id, req_flag_id } => {
                    hasher.add_u8(15);
                    hasher.add_u16(get_upgrade_id);
                    hasher.add_u16(req_flag_id);
                }
                RoomTriggerType::GetCollectable { get_collectable_id, interact_prompt_id, req_flag_id } => {
                    hasher.add_u8(16);
                    hasher.add_u16(get_collectable_id);
                    hasher.add_u16(interact_prompt_id);
                    hasher.add_u16(req_flag_id);
                }
                RoomTriggerType::GetPickup { get_pickup_id, respawn_delay, sfx_id } => {
                    hasher.add_u8(17);
                    hasher.add_u16(get_pickup_id);
                    hasher.add_u16(respawn_delay);
                    if let Some(sfx_id) = sfx_id {
                        hasher.add_u32(1);
                        hasher.add_asset_id(sfx_id);
                    } else {
                        hasher.add_u32(0);
                    }
                }
                RoomTriggerType::OnLoadRoom { set_effect_id, set_flag_id, req_flag_id, deny_flag_id } => {
                    hasher.add_u8(18);
                    hasher.add_u16(set_flag_id);
                    hasher.add_u16(set_effect_id);
                    hasher.add_u16(req_flag_id);
                    hasher.add_u16(deny_flag_id);
                }
            }
        }
        hasher.add_bool(self.has_script);
    }
}

impl super::DuplicableAsset<Room> for Room {
    fn duplicate(&self, dup_id: DataAssetId, dup_name: String) -> Self {
        Room {
            asset: self.asset.duplicate(dup_id, dup_name),
            maps: self.maps.clone(),
            triggers: self.triggers.clone(),
            cutscenes: self.cutscenes.clone(),
            has_script: self.has_script,
            parallax: self.parallax.clone(),
        }
    }
}

impl super::GenericAsset for Room {
    fn asset(&self) -> &super::DataAsset { &self.asset }

    fn data_size(&self) -> usize {
        // header: u16(2) * (num_maps, num_triggers, num_cutscenes) + ptr(4) * (maps, triggers, cutscenes)
        let header = 2usize * 3 + 4usize * 3;

        // map[0..num_maps]: u16(2) * (x, y) + map<ptr>(4)
        let maps = self.maps.len() * (2usize * 2 + 4usize);

        // trigger[0..num_triggers]: u16(2)*(type,id,x,y) + u16(2)*8
        let triggers = self.triggers.len() * (2usize * 4 + 2usize * 8);

        // cutscene[0..num_cutscenes]: u16(2) * (num_actions) + actions<ptr>(4)
        //   action[0..num_actions]: u16(2)*(type) + u16(2)*6
        let cutscenes = self.cutscenes.len() * (2usize + 2usize * 8) + self.cutscenes.iter().fold(0, |size, cutscene| {
            size + cutscene.actions.len() * (2usize + 2usize * 6)
        });

        // parallax:
        //   header: u16(2) * (tileset_index, width, height, pad) + tiles<ptr>(4)
        //   tiles:  w * h * [u8(1)]
        let parallax = 2usize * 4 + 4usize + (self.parallax.width as usize * self.parallax.height as usize);

        header + maps + triggers + cutscenes + parallax
    }
}

pub enum RoomTriggerTypeIdent {
    Unknown,
    PlayerSpawn,
    EnemySpawn,
    Door,
    WallButton,
    DetectorButton,
    UnblockTiles,
    DisableAnimation,
    ChangeTiles,
    RemoveEnemy,
    PlayAnimation,
    PlayCutscene,
    PlaySfx,
    StopSfx,
    GetUpgrade,
    GetCollectable,
    GetPickup,
    OnLoadRoom,
}

impl RoomTriggerTypeIdent {
    pub fn from_trigger_type(trigger_type: &RoomTriggerType) -> Self {
        match trigger_type {
            RoomTriggerType::Unknown {..} => { RoomTriggerTypeIdent::Unknown }
            RoomTriggerType::Door{..} => { RoomTriggerTypeIdent::Door }
            RoomTriggerType::PlayerSpawn{..} => { RoomTriggerTypeIdent::PlayerSpawn }
            RoomTriggerType::EnemySpawn{..} => { RoomTriggerTypeIdent::EnemySpawn }
            RoomTriggerType::WallButton {..} => { RoomTriggerTypeIdent::WallButton }
            RoomTriggerType::DetectorButton {..} => { RoomTriggerTypeIdent::DetectorButton }
            RoomTriggerType::UnblockTiles {..} => { RoomTriggerTypeIdent::UnblockTiles }
            RoomTriggerType::DisableAnimation {..} => { RoomTriggerTypeIdent::DisableAnimation }
            RoomTriggerType::ChangeTiles {..} => { RoomTriggerTypeIdent::ChangeTiles }
            RoomTriggerType::RemoveEnemy {..} => { RoomTriggerTypeIdent::RemoveEnemy }
            RoomTriggerType::PlayAnimation {..} => { RoomTriggerTypeIdent::PlayAnimation }
            RoomTriggerType::PlayCutscene {..} => { RoomTriggerTypeIdent::PlayCutscene }
            RoomTriggerType::PlaySfx {..} => { RoomTriggerTypeIdent::PlaySfx }
            RoomTriggerType::StopSfx {..} => { RoomTriggerTypeIdent::StopSfx }
            RoomTriggerType::GetUpgrade {..} => { RoomTriggerTypeIdent::GetUpgrade }
            RoomTriggerType::GetCollectable {..} => { RoomTriggerTypeIdent::GetCollectable }
            RoomTriggerType::GetPickup {..} => { RoomTriggerTypeIdent::GetPickup }
            RoomTriggerType::OnLoadRoom {..} => { RoomTriggerTypeIdent::OnLoadRoom }
        }
    }

    pub fn enum_ident(&self) -> &'static str {
        match self {
            RoomTriggerTypeIdent::Unknown => { "ROOM_TRIGGER_TYPE_UNKNOWN" }
            RoomTriggerTypeIdent::Door => { "ROOM_TRIGGER_TYPE_DOOR" }
            RoomTriggerTypeIdent::PlayerSpawn => { "ROOM_TRIGGER_TYPE_PLAYER_SPAWN" }
            RoomTriggerTypeIdent::EnemySpawn => { "ROOM_TRIGGER_TYPE_ENEMY_SPAWN" }
            RoomTriggerTypeIdent::WallButton => { "ROOM_TRIGGER_TYPE_WALL_BUTTON" }
            RoomTriggerTypeIdent::DetectorButton => { "ROOM_TRIGGER_TYPE_DETECTOR_BUTTON" }
            RoomTriggerTypeIdent::UnblockTiles => { "ROOM_TRIGGER_TYPE_UNBLOCK_TILES" }
            RoomTriggerTypeIdent::DisableAnimation => { "ROOM_TRIGGER_TYPE_DISABLE_ANIMATION" }
            RoomTriggerTypeIdent::ChangeTiles => { "ROOM_TRIGGER_TYPE_CHANGE_TILES" }
            RoomTriggerTypeIdent::RemoveEnemy => { "ROOM_TRIGGER_TYPE_REMOVE_ENEMY" }
            RoomTriggerTypeIdent::PlayAnimation => { "ROOM_TRIGGER_TYPE_PLAY_ANIMATION" }
            RoomTriggerTypeIdent::PlayCutscene => { "ROOM_TRIGGER_TYPE_PLAY_CUTSCENE" }
            RoomTriggerTypeIdent::PlaySfx => { "ROOM_TRIGGER_TYPE_PLAY_SFX" }
            RoomTriggerTypeIdent::StopSfx => { "ROOM_TRIGGER_TYPE_STOP_SFX" }
            RoomTriggerTypeIdent::GetUpgrade => { "ROOM_TRIGGER_TYPE_GET_UPGRADE" }
            RoomTriggerTypeIdent::GetCollectable => { "ROOM_TRIGGER_TYPE_GET_COLLECTABLE" }
            RoomTriggerTypeIdent::GetPickup => { "ROOM_TRIGGER_TYPE_GET_PICKUP" }
            RoomTriggerTypeIdent::OnLoadRoom => { "ROOM_TRIGGER_TYPE_ON_LOAD_ROOM" }
        }
    }

    pub fn matches_enum_ident(&self, enum_ident: &str, prefix: &str) -> bool {
        if let Some(enum_ident) = enum_ident.strip_prefix(prefix) {
            enum_ident.eq_ignore_ascii_case(self.enum_ident())
        } else {
            false
        }
    }
}

pub enum RoomCutsceneActionTypeIdent {
    Unknown,
    WaitFrames,
    WaitPlayerAnimationEnd,
    DispelDarkness,
    SetPlayerDarkness,
    MovePlayerDarkness,
    SetPlayerState,
    ActivateEffect,
    SetHudEnabled,
}

impl RoomCutsceneActionTypeIdent {
    pub fn from_action_type(action_type: &RoomCutsceneActionType) -> Self {
        match action_type {
            RoomCutsceneActionType::Unknown {..} => { RoomCutsceneActionTypeIdent::Unknown }
            RoomCutsceneActionType::WaitFrames  {..} => { RoomCutsceneActionTypeIdent::WaitFrames }
            RoomCutsceneActionType::WaitPlayerAnimationEnd {..} => { RoomCutsceneActionTypeIdent::WaitPlayerAnimationEnd }
            RoomCutsceneActionType::DispelDarkness {..} => { RoomCutsceneActionTypeIdent::DispelDarkness }
            RoomCutsceneActionType::SetPlayerDarkness {..} => { RoomCutsceneActionTypeIdent::SetPlayerDarkness }
            RoomCutsceneActionType::MovePlayerDarkness {..} => { RoomCutsceneActionTypeIdent::MovePlayerDarkness }
            RoomCutsceneActionType::SetPlayerState {..} => { RoomCutsceneActionTypeIdent::SetPlayerState }
            RoomCutsceneActionType::ActivateEffect {..} => { RoomCutsceneActionTypeIdent::ActivateEffect }
            RoomCutsceneActionType::SetHudEnabled {..} => { RoomCutsceneActionTypeIdent::SetHudEnabled }
        }
    }

    pub fn enum_ident(&self) -> &'static str {
        match self {
            RoomCutsceneActionTypeIdent::Unknown => { "ROOM_ACTION_TYPE_UNKNOWN" }
            RoomCutsceneActionTypeIdent::WaitFrames => { "ROOM_ACTION_TYPE_WAIT_FRAMES" }
            RoomCutsceneActionTypeIdent::WaitPlayerAnimationEnd => { "ROOM_ACTION_TYPE_WAIT_PLAYER_ANIM_END" }
            RoomCutsceneActionTypeIdent::DispelDarkness => { "ROOM_ACTION_TYPE_DISPEL_DARKNESS" }
            RoomCutsceneActionTypeIdent::SetPlayerDarkness => { "ROOM_ACTION_TYPE_SET_PLAYER_DARKNESS" }
            RoomCutsceneActionTypeIdent::MovePlayerDarkness => { "ROOM_ACTION_TYPE_MOVE_PLAYER_DARKNESS" }
            RoomCutsceneActionTypeIdent::SetPlayerState => { "ROOM_ACTION_TYPE_SET_PLAYER_STATE" }
            RoomCutsceneActionTypeIdent::ActivateEffect => { "ROOM_ACTION_TYPE_ACTIVATE_EFFECT" }
            RoomCutsceneActionTypeIdent::SetHudEnabled => { "ROOM_ACTION_TYPE_SET_HUD_ENABLED" }
        }
    }

    pub fn matches_enum_ident(&self, enum_ident: &str, prefix: &str) -> bool {
        if let Some(enum_ident) = enum_ident.strip_prefix(prefix) {
            enum_ident.eq_ignore_ascii_case(self.enum_ident())
        } else {
            false
        }
    }
}
