use crate::data_asset::{
    AssetList,
    Room,
    MapData,
    Tileset,
};

pub const COLLISION_FLAGS_RAMP : u32   =  1<<0;
pub const COLLISION_FLAGS_DOWN : u32   =  1<<1;
pub const COLLISION_FLAGS_UP : u32     =  1<<2;
pub const COLLISION_FLAGS_LEFT : u32   =  1<<3;
pub const COLLISION_FLAGS_RIGHT : u32  =  1<<4;

pub const TILE_BLOCK: u8        =  0;
pub const TILE_L_RAMP_FULL: u8  =  1;
pub const TILE_R_RAMP_FULL: u8  =  2;
pub const TILE_L_RAMP_BOT: u8   =  3;
pub const TILE_L_RAMP_TOP: u8   =  4;
pub const TILE_R_RAMP_BOT: u8   =  5;
pub const TILE_R_RAMP_TOP: u8   =  6;

const TILE_SIZE: i32 = Tileset::TILE_SIZE as i32;

#[derive(Copy, Clone, Debug)]
pub struct CollisionRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

pub fn is_point_in_rect(x: i32, y: i32, rect: &CollisionRect) -> bool {
    return x >= rect.x && y >= rect.y && x < rect.x + rect.w && y < rect.y + rect.h;
}

pub fn get_room_tile_at(room: &Room, maps: &AssetList<MapData>, x: i32, y: i32) -> u8 {
    for room_map in &room.maps {
        if let Some(map) = maps.get(&room_map.map_id) {
            let mx = x - room_map.x as i32;
            let my = y - room_map.y as i32;
            if mx >= 0 && my >= 0 && (mx as u32) < map.width && (my as u32) < map.height {
                return map.fx_tiles[(map.width * my as u32 + mx as u32) as usize] & 0x0f;
            }
        }
    }
    TILE_BLOCK
}

fn h_move(rect: &mut CollisionRect, room: &Room, maps: &AssetList<MapData>, sx: i32) -> u32 {
    if rect.x + sx < 0 {
        rect.x = 0;
        return COLLISION_FLAGS_LEFT;
    }

    let rx = if sx < 0 { rect.x } else { rect.x + rect.w - 1 };
    let tx = (rx + sx) / TILE_SIZE;

    let ty_top = rect.y / TILE_SIZE;
    let ty_bot = (rect.y + rect.h - 1) / TILE_SIZE;
    for ty in (ty_top..=ty_bot).rev() {
        let tile = get_room_tile_at(room, maps, tx, ty);
        match tile {
            TILE_BLOCK => {
                return if sx > 0 { COLLISION_FLAGS_RIGHT } else { COLLISION_FLAGS_LEFT };
            }

            TILE_L_RAMP_FULL => {
                if ty < ty_bot {
                    // ramp above foot level
                    return if sx > 0 { COLLISION_FLAGS_RIGHT } else { COLLISION_FLAGS_LEFT };
                }
                let ix = (rx + sx) % TILE_SIZE;
                let iy = (rect.y + rect.h - 1) % TILE_SIZE;
                if iy >= TILE_SIZE - 1 - ix { // under ramp
                    if sx < 0 {
                        return COLLISION_FLAGS_LEFT;
                    }
                    rect.x += 1;
                    rect.y -= 1;
                    return COLLISION_FLAGS_RAMP;
                }
            }

            TILE_R_RAMP_FULL => {
                if ty < ty_bot {
                    // ramp above foot level
                    return if sx > 0 { COLLISION_FLAGS_RIGHT } else { COLLISION_FLAGS_LEFT };
                }
                let ix = (rx + sx) % TILE_SIZE;
                let iy = (rect.y + rect.h - 1) % TILE_SIZE;
                if iy >= ix { // under ramp
                    if sx > 0 {
                        // ramp from the wrong size
                        return COLLISION_FLAGS_RIGHT;
                    }
                    rect.x -= 1;
                    rect.y -= 1;
                    return COLLISION_FLAGS_RAMP;
                }
            }

            TILE_L_RAMP_TOP => {
                if ty < ty_bot {
                    // ramp above foot level
                    return if sx > 0 { COLLISION_FLAGS_RIGHT } else { COLLISION_FLAGS_LEFT };
                }
                let ix = (rx + sx) % TILE_SIZE;
                let iy = (rect.y + rect.h - 1) % TILE_SIZE;
                if 2*iy >= TILE_SIZE - 1 - ix { // under ramp
                    if sx < 0 {
                        // ramp from the wrong size
                        return COLLISION_FLAGS_LEFT;
                    }
                    rect.x += 1;
                    rect.y -= 1;
                    return COLLISION_FLAGS_RAMP;
                }
            }

            TILE_L_RAMP_BOT => {
                if ty < ty_bot {
                    // ramp above foot level
                    return if sx > 0 { COLLISION_FLAGS_RIGHT } else { COLLISION_FLAGS_LEFT };
                }
                let ix = (rx + sx) % TILE_SIZE;
                let iy = (rect.y + rect.h - 1) % TILE_SIZE;
                if 2*iy - TILE_SIZE >= TILE_SIZE - 1 - ix { // under ramp
                    if sx < 0 {
                        // ramp from the wrong size
                        return COLLISION_FLAGS_LEFT;
                    }
                    rect.x += 1;
                    rect.y -= 1;
                    return COLLISION_FLAGS_RAMP;
                }
            }

            TILE_R_RAMP_TOP => {
                if ty < ty_bot {
                    // ramp above foot level
                    return if sx > 0 { COLLISION_FLAGS_RIGHT } else { COLLISION_FLAGS_LEFT };
                }
                let ix = (rx + sx) % TILE_SIZE;
                let iy = (rect.y + rect.h - 1) % TILE_SIZE;
                if 2*iy >= ix { // under ramp
                    if sx > 0 {
                        // ramp from the wrong size
                        return COLLISION_FLAGS_RIGHT;
                    }
                    rect.x -= 1;
                    rect.y -= 1;
                    return COLLISION_FLAGS_RAMP;
                }
            }

            TILE_R_RAMP_BOT => {
                if ty < ty_bot {
                    // ramp above foot level
                    return if sx > 0 { COLLISION_FLAGS_RIGHT } else { COLLISION_FLAGS_LEFT };
                }
                let ix = (rx + sx) % TILE_SIZE;
                let iy = (rect.y + rect.h - 1) % TILE_SIZE;
                if 2*iy - TILE_SIZE >= ix { // under ramp
                    if sx > 0 {
                        // ramp from the wrong size
                        return COLLISION_FLAGS_RIGHT;
                    }
                    rect.x -= 1;
                    rect.y -= 1;
                    return COLLISION_FLAGS_RAMP;
                }
            }

            _ => {}
        }
    }

    rect.x += sx;
    0
}

fn v_move(rect: &mut CollisionRect, room: &Room, maps: &AssetList<MapData>, sy: i32) -> u32 {
    if rect.y + sy < 0 {
        rect.y = 0;
        return COLLISION_FLAGS_UP;
    }

    let ry = if sy < 0 { rect.y } else { rect.y + rect.h - 1 };
    let ty = (ry + sy) / TILE_SIZE;

    let tx_left = rect.x / TILE_SIZE;
    let tx_right = (rect.x + rect.w - 1) / TILE_SIZE;
    for tx in tx_left..=tx_right {
        let tile = get_room_tile_at(room, maps, tx, ty);
        match tile {
            TILE_BLOCK => {
                return if sy < 0 { COLLISION_FLAGS_UP } else { COLLISION_FLAGS_DOWN };
            }

            TILE_L_RAMP_FULL => {
                if sy < 0 { return COLLISION_FLAGS_UP; }
                if tx < tx_right {
                    // hanging over ramp
                    return COLLISION_FLAGS_DOWN;
                }

                let ix = (rect.x + rect.w - 1) % TILE_SIZE;
                let iy = (ry + 1) % TILE_SIZE;
                if iy >= TILE_SIZE - 1 - ix {
                    return COLLISION_FLAGS_DOWN;
                }
            }

            TILE_R_RAMP_FULL => {
                if sy < 0 { return COLLISION_FLAGS_UP; }
                if tx > tx_left {
                    // hanging over ramp
                    return COLLISION_FLAGS_DOWN;
                }

                let ix = rect.x % TILE_SIZE;
                let iy = (ry + 1) % TILE_SIZE;
                if iy >= ix {
                    return COLLISION_FLAGS_DOWN;
                }
            }

            TILE_L_RAMP_TOP => {
                if sy < 0 { return COLLISION_FLAGS_UP; }
                if tx < tx_right {
                    // hanging over ramp
                    return COLLISION_FLAGS_DOWN;
                }

                let ix = (rect.x + rect.w - 1) % TILE_SIZE;
                let iy = (ry + 1) % TILE_SIZE;
                if 2*iy >= TILE_SIZE - 1 - ix {
                    return COLLISION_FLAGS_DOWN;
                }
            }

            TILE_L_RAMP_BOT => {
                if sy < 0 { return COLLISION_FLAGS_UP; }
                if tx < tx_right && (ry + 1) % TILE_SIZE >= TILE_SIZE/2 {
                    // hanging over ramp
                    return COLLISION_FLAGS_DOWN;
                }

                let ix = (rect.x + rect.w - 1) % TILE_SIZE;
                let iy = (ry + 1) % TILE_SIZE;
                if 2*iy - TILE_SIZE >= TILE_SIZE - 1 - ix {
                    return COLLISION_FLAGS_DOWN;
                }
            }

            TILE_R_RAMP_TOP => {
                if sy < 0 { return COLLISION_FLAGS_UP; }
                if tx > tx_left {
                    // hanging over ramp
                    return COLLISION_FLAGS_DOWN;
                }

                let ix = rect.x % TILE_SIZE;
                let iy = (ry + 1) % TILE_SIZE;
                if 2*iy >= ix {
                    return COLLISION_FLAGS_DOWN;
                }
            }

            TILE_R_RAMP_BOT => {
                if sy < 0 { return COLLISION_FLAGS_UP; }
                if tx > tx_left && (ry + 1) % TILE_SIZE >= TILE_SIZE/2 {
                    // hanging over ramp
                    return COLLISION_FLAGS_DOWN;
                }

                let ix = rect.x % TILE_SIZE;
                let iy = (ry + 1) % TILE_SIZE;
                if 2*iy - TILE_SIZE >= ix {
                    return COLLISION_FLAGS_DOWN;
                }
            }

            _ => {}
        }
    }

    rect.y += sy;
    0
}

pub fn collision_move(rect: &mut CollisionRect, room: &Room, maps: &AssetList<MapData>, dx: i32, dy: i32) -> u32 {
    if dx == 0 && dy == 0 { return 0; }
    let sx = if dx < 0 { -1 } else { 1 };
    let sy = if dy < 0 { -1 } else { 1 };

    if dx == 0 {
        let mut flags = 0;
        for _y in 0..dy.abs() {
            flags |= v_move(rect, room, maps, sy);
        }
        return flags;
    }

    if dy == 0 {
        let mut flags = 0;
        for _x in 0..dx.abs() {
            flags |= h_move(rect, room, maps, sx);
        }
        return flags;
    }

    let x_end = dx;
    let y_end = dy;
    let dx = if dx < 0 { -dx } else { dx };
    let dy = if dy > 0 { -dy } else { dy };
    let mut error = dx + dy;
    let mut x = 0;
    let mut y = 0;
    let mut flags = 0;
    loop {
        if x == x_end && y == y_end { break; }
        let e2 = 2 * error;
        if e2 >= dy {
            error += dy;
            flags |= h_move(rect, room, maps, sx);
            x += sx;
        }
        if e2 <= dx {
            error += dx;
            flags |= v_move(rect, room, maps, sy);
            y += sy;
        }
    }
    flags
}
