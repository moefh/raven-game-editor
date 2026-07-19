use std::collections::HashMap;

use crate::data_asset::{
    DataAssetId,
    AssetList,
    World,
    WorldRegion,
    Room,
    RoomTrigger,
    RoomTriggerType,
    MapData,
    Tileset,
};

pub const DOOR_WIDTH: i16 = Tileset::TILE_SIZE as i16;
pub const DOOR_HEIGHT: i16 = 4 * Tileset::TILE_SIZE as i16;

pub fn get_world_size(world: &World) -> (i32, i32) {
    world.regions.iter().fold((0, 0), |size, region| {
        (
            size.0.max(region.x as i32 + region.width as i32),
            size.1.max(region.y as i32 + region.height as i32),
        )
    })
}

fn get_region_block(x: i32, y: i32, region: &WorldRegion) -> Option<u8> {
    if x < 0 || y < 0 || x > region.width as i32 || y > region.height as i32 {
        None
    } else {
        region.blocks[(y * WorldRegion::BLOCK_STRIDE as i32 + x) as usize]
    }
}

fn get_world_block(x: i32, y: i32, world: &World) -> Option<u32> {
    for (region_index, region) in world.regions.iter().enumerate() {
        let bx = x - region.x as i32;
        let by = y - region.y as i32;
        let rw = region.width as i32;
        let rh = region.height as i32;
        if bx >= 0 && by >= 0 && bx < rw && by < rh &&
            let Some(block) = region.blocks[(by * WorldRegion::BLOCK_STRIDE as i32 + bx) as usize].map(|block| {
                block as u32 | ((region_index as u32) << 8)
            }) {
                return Some(block);
            }
    }
    None
}

fn trigger_is_door(trigger: &RoomTrigger) -> bool {
    matches!(trigger.trigger_type, RoomTriggerType::Door {..})
}

fn count_room_doors(room: &Room) -> usize {
    room.triggers.iter().fold(0, |num_doors, trigger| {
        num_doors + if trigger_is_door(trigger) { 1 } else { 0 }
    })
}

fn count_region_doors(region: &WorldRegion, rooms: &AssetList<Room>) -> usize {
    region.rooms.iter().fold(0, |num_doors, room_id| {
        num_doors + rooms.get(room_id).map(|room| { count_room_doors(room) }).unwrap_or(0)
    })
}

#[derive(Clone, Copy)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

pub struct Door {
    pub index: usize,
    pub region_index: Option<usize>,
    pub pos: Option<Position>,
    pub room_id: Option<DataAssetId>,
    pub trigger_index: Option<usize>,
    pub dest_region_index: Option<usize>,
    pub dest_door_index: Option<usize>,
}

impl Door {
    fn new() -> Self {
        Door {
            index: 0,
            region_index: None,
            pos: None,
            room_id: None,
            trigger_index: None,
            dest_region_index: None,
            dest_door_index: None,
        }
    }

    pub fn get_info_with_region(&self, info: &mut String, world: &World, rooms: &AssetList<Room>) {
        if let Some(region) = self.get_region(world) {
            info.push('<');
            info.push_str(&region.name);
            info.push('>');
        } else {
            info.push_str("<UNKNOWN REGION>");
        }
        self.get_info(info, rooms);
    }

    pub fn get_info(&self, info: &mut String, rooms: &AssetList<Room>) {
        if let Some(room) = self.get_room(rooms) {
            use std::fmt::Write as _;
            info.push_str(&room.asset.name);
            info.push('[');
            if let Some(trigger) = self.get_trigger(rooms) {
                let _ = write!(info, "{}", trigger.trigger_id);
            } else {
                info.push_str("INVALID DOOR");
            }
            info.push(']');
        }
    }

    pub fn get_room<'a>(&self, rooms: &'a AssetList<Room>) -> Option<&'a Room> {
        self.room_id.and_then(|room_id| rooms.get(&room_id))
    }

    pub fn get_trigger<'a>(&self, rooms: &'a AssetList<Room>) -> Option<&'a RoomTrigger> {
        self.get_room(rooms).and_then(|room| {
            self.trigger_index.and_then(|trigger_index| room.triggers.get(trigger_index))
        })
    }

    pub fn get_room_mut<'a>(&self, rooms: &'a mut AssetList<Room>) -> Option<&'a mut Room> {
        self.room_id.and_then(|room_id| rooms.get_mut(&room_id))
    }

    pub fn get_trigger_mut<'a>(&self, rooms: &'a mut AssetList<Room>) -> Option<&'a mut RoomTrigger> {
        self.get_room_mut(rooms).and_then(|room| {
            self.trigger_index.and_then(|trigger_index| room.triggers.get_mut(trigger_index))
        })
    }

    pub fn get_dest_door<'a>(&self, doors: &'a [Door]) -> Option<&'a Door> {
        self.dest_door_index.and_then(|index| doors.get(index))
    }

    pub fn get_region<'a>(&self, world: &'a World) -> Option<&'a WorldRegion> {
        self.region_index.and_then(|index| world.regions.get(index))
    }

    pub fn is_in_region(&self, region_index: Option<usize>) -> bool {
        region_index.map(|region_index| {
            self.dest_region_index.map(|dest_region_index| dest_region_index == region_index).unwrap_or(false)
        }).unwrap_or(false)
    }

}

pub struct RoomInfo {
    pub block_x: f32,
    pub block_y: f32,
    pub block_width: f32,
    pub block_height: f32,
    pub width: f32,
    pub height: f32,
}

impl RoomInfo {
    pub fn calculate(region: &WorldRegion, room: &Room, maps: &AssetList<MapData>) -> Option<RoomInfo> {
        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;
        for y in 0..region.height as i32 {
            for x in 0..region.width as i32 {
                if let Some(block) = get_region_block(x, y, region) && region.rooms.get(block as usize).copied() == Some(room.asset.id) {
                    min_x = min_x.min(x);
                    min_y = min_y.min(y);
                    max_x = max_x.max(x);
                    max_y = max_y.max(y);
                }
            }
        }
        if min_x < i32::MAX && min_y < i32::MAX && max_x > i32::MIN && max_y > i32::MIN {
            let tile_size = room.maps.iter().fold((0, 0), |max, room_map| {
                match maps.get(&room_map.map_id) {
                    Some(map) => (max.0.max(room_map.x as u32 + map.width), max.1.max(room_map.y as u32 + map.height)),
                    None => max,
                }
            });
            Some(RoomInfo {
                block_x: min_x as f32,
                block_y: min_y as f32,
                block_width: (max_x - min_x + 1) as f32,
                block_height: (max_y - min_y + 1) as f32,
                width: (tile_size.0 * Tileset::TILE_SIZE) as f32,
                height: (tile_size.1 * Tileset::TILE_SIZE) as f32,
            })
        } else {
            None
        }
    }
}

pub struct Grid {
    pub region_x: f32,
    pub region_y: f32,
    pub width: i32,
    pub height: i32,
    pub door_indices: Vec<usize>,
    borders: Vec<u8>,
}

impl Grid {
    pub const BORDER_TOP: u8 = 1 << 0;
    pub const BORDER_LEFT: u8 = 1 << 1;

    fn new() -> Self {
        Grid {
            region_x: 0.0,
            region_y: 0.0,
            width: 0,
            height: 0,
            borders: Vec::new(),
            door_indices: Vec::new(),
        }
    }

    fn update_borders<T, F>(&mut self, width: i32, height: i32, get_block: F)
    where
        T: std::cmp::PartialEq,
        F: Fn(i32, i32) -> Option<T>
    {
        let num_blocks = (width * height) as usize;
        if self.borders.len() < num_blocks {
            self.borders.resize(num_blocks, 0);
        }
        self.width = width;
        self.height = height;
        self.borders[..].fill(0);
        for y in 0..height {
            for x in 0..width {
                let block = get_block(x, y);
                let left = if block != get_block(x-1, y) { Self::BORDER_LEFT } else { 0 };
                let top  = if block != get_block(x, y-1) { Self::BORDER_TOP  } else { 0 };
                self.borders[(y*width + x) as usize] = left | top;
            }
        }
    }

    fn update_door_indices(&mut self, first_index: usize, num_indices: usize) {
        self.door_indices.resize(num_indices, 0);
        for (index, door_index) in self.door_indices.iter_mut().enumerate() {
            *door_index = first_index + index;
        }
    }

    pub fn get_block_borders(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            0
        } else {
            self.borders[(y * self.width + x) as usize]
        }
    }
}

pub struct WorldGridStore {
    state_hash: u64,
    pub world_grid: Grid,
    pub region_grids: Vec<Grid>,
    pub doors: Vec<Door>,
}

impl WorldGridStore {
    pub fn new() -> Self {
        WorldGridStore {
            state_hash: 0,
            world_grid: Grid::new(),
            region_grids: Vec::new(),
            doors: Vec::new(),
        }
    }

    pub fn update(&mut self, world: &World, rooms: &AssetList<Room>, maps: &AssetList<MapData>) {
        use std::hash::{Hash, Hasher};

        let mut hasher = std::hash::DefaultHasher::new();
        world.hash(&mut hasher);
        for room in rooms.iter() { room.hash(&mut hasher); }
        for map in maps.iter() { map.hash(&mut hasher); }
        let hash = hasher.finish();

        if self.state_hash != hash {
            self.state_hash = hash;
            self.update_borders(world);
            self.update_doors(world, rooms, maps);
        }
    }

    fn update_borders(&mut self, world: &World) {
        // update world
        let (world_width, world_height) = get_world_size(world);
        self.world_grid.update_borders(world_width + 1, world_height + 1, |x, y| { get_world_block(x, y, world) });

        // update regions
        if self.region_grids.len() < world.regions.len() {
            self.region_grids.resize_with(world.regions.len(), Grid::new);
        }
        for (region_grid, region) in self.region_grids.iter_mut().zip(world.regions.iter()) {
            let width = region.width as i32 + 1;
            let height = region.height as i32 + 1;
            region_grid.update_borders(width, height, |x, y| { get_region_block(x, y, region) });
        }
    }

    fn update_doors(&mut self, world: &World, rooms: &AssetList<Room>, maps: &AssetList<MapData>) {
        // update door indices
        let mut num_world_doors = 0;
        for (region_grid, region) in self.region_grids.iter_mut().zip(world.regions.iter()) {
            let num_region_doors = count_region_doors(region, rooms);
            region_grid.update_door_indices(num_world_doors, num_region_doors);
            num_world_doors += num_region_doors;
        }
        self.world_grid.update_door_indices(0, num_world_doors);
        if self.doors.len() != num_world_doors {
            self.doors.resize_with(num_world_doors, Door::new);
            self.doors.iter_mut().enumerate().for_each(|(index, door)| { door.index = index; });
        }

        // build map of: room_id<DataAssetId> -> trigger_id<u16> -> (region_index<usize>, door_index<usize>)
        let mut trigger_to_door = HashMap::new();
        let mut door_index = 0;
        for (region_index, region) in world.regions.iter().enumerate() {
            for room in region.rooms.iter().filter_map(|room_id| { rooms.get(room_id) }) {
                for trigger in room.triggers.iter() {
                    if matches!(trigger.trigger_type, RoomTriggerType::Door { .. }) &&
                        self.doors.get(door_index).is_some() {
                            trigger_to_door.insert((room.asset.id, trigger.trigger_id), (region_index, door_index));
                            door_index += 1;
                        }
                }
            }
        }

        // update doors
        let mut door_index = 0;
        for (region_index, (region_grid, region)) in self.region_grids.iter_mut().zip(world.regions.iter()).enumerate() {
            region_grid.region_x = region.x as f32;
            region_grid.region_y = region.y as f32;
            for room in region.rooms.iter().filter_map(|room_id| rooms.get(room_id)) {
                let room_info = RoomInfo::calculate(region, room, maps);
                for (trigger_index, trigger) in room.triggers.iter().enumerate() {
                    if let RoomTriggerType::Door { dest_room_id, dest_trigger_id } = trigger.trigger_type &&
                        let Some(door) = self.doors.get_mut(door_index) {
                            door.region_index = Some(region_index);
                            door.room_id = Some(room.asset.id);
                            door.trigger_index = Some(trigger_index);
                            if let Some(room_info) = &room_info {
                                let room_x = region.x as f32 + room_info.block_x;
                                let room_y = region.y as f32 + room_info.block_y;
                                let door_x = (trigger.x + DOOR_WIDTH/2) as f32 / room_info.width * room_info.block_width;
                                let door_y = (trigger.y + DOOR_HEIGHT/2) as f32 / room_info.height * room_info.block_height;
                                door.pos = Some(Position {
                                    x: room_x + door_x,
                                    y: room_y + door_y
                                });
                            } else {
                                door.pos = None;
                            }
                            if let Some((region_index, door_index)) = trigger_to_door.get(&(dest_room_id, dest_trigger_id)) {
                                door.dest_region_index = Some(*region_index);
                                door.dest_door_index = Some(*door_index);
                            } else {
                                door.dest_region_index = None;
                                door.dest_door_index = None;
                            }
                            door_index += 1;
                        }
                }
            }
        }
    }

    pub fn set_door_dest(&self, door_index: usize, dest_door_index: usize, rooms: &mut AssetList<Room>) -> bool {
        if let Some(door) = self.doors.get(door_index) &&
            let Some(dest_door) = self.doors.get(dest_door_index) &&
            let Some(dest_room_id) = dest_door.room_id &&
            let Some(dest_trigger_id) = dest_door.get_trigger(rooms).map(|trigger| trigger.trigger_id) &&
            let Some(trigger) = door.get_trigger_mut(rooms) {
                trigger.trigger_type = RoomTriggerType::Door {
                    dest_room_id,
                    dest_trigger_id,
                };
                true
            } else {
                false
            }
    }
}
