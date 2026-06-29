use crate::data_asset::{
    DataAssetId,
    AssetList,
    MapData,
};

#[derive(Clone, Copy, PartialEq)]
pub enum MapLayer {
    Foreground,
    Background,
    Effects,
    Parallax,
    Screen,
}

#[derive(Clone, Copy)]
pub struct MapRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl MapRect {
    pub fn from_rect(rect: egui::Rect, map_data: &MapData, layer: MapLayer) -> Option<Self> {
        let (map_width, map_height) = match layer {
            MapLayer::Parallax => (map_data.para_width, map_data.para_height),
            _ => (map_data.width, map_data.height),
        };
        let rect = rect.intersect(egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(map_width as f32, map_height as f32)));
        Some(MapRect {
            x: rect.min.x as u32,
            y: rect.min.y as u32,
            width: rect.width().max(0.0) as u32,
            height: rect.height().max(0.0) as u32,
        })
    }
}

#[derive(Clone)]
pub struct MapLayerFragment {
    pub width: u32,
    pub height: u32,
    pub layer: MapLayer,
    pub data: Vec<u8>,
}

impl MapLayerFragment {
    pub fn copy_map(map_data: &mut MapData, layer: MapLayer, rect: MapRect) -> Option<MapLayerFragment> {
        let (map_width, map_height, map_data) = match layer {
            MapLayer::Foreground => (map_data.width, map_data.height, &mut map_data.fg_tiles),
            MapLayer::Background => (map_data.width, map_data.height, &mut map_data.bg_tiles),
            MapLayer::Effects    => (map_data.width, map_data.height, &mut map_data.fx_tiles),
            MapLayer::Parallax   => (map_data.para_width, map_data.para_height, &mut map_data.para_tiles),
            _ => { return None; }
        };
        if rect.width == 0 || rect.height == 0 || rect.x + rect.width > map_width || rect.y + rect.height > map_height { return None; }

        let mut data = Vec::with_capacity((rect.width * rect.height) as usize);
        for y in 0..rect.height {
            for x in 0..rect.width {
                data.push(map_data[((rect.y + y) * map_width + rect.x + x) as usize]);
            }
        }
        Some(MapLayerFragment {
            width: rect.width,
            height: rect.height,
            layer,
            data,
        })
    }

    pub fn cut_map(map_data: &mut MapData, layer: MapLayer, rect: MapRect, fill_tile: u8) -> Option<MapLayerFragment> {
        let frag = Self::copy_map(map_data, layer, rect);
        if frag.is_some() {
            let (map_width, map_height, map_data) = match layer {
                MapLayer::Foreground => (map_data.width, map_data.height, &mut map_data.fg_tiles),
                MapLayer::Background => (map_data.width, map_data.height, &mut map_data.bg_tiles),
                MapLayer::Effects    => (map_data.width, map_data.height, &mut map_data.fx_tiles),
                MapLayer::Parallax   => (map_data.para_width, map_data.para_height, &mut map_data.para_tiles),
                _ => { return None; }
            };
            if rect.width > 0 && rect.height > 0 && rect.x + rect.width <= map_width && rect.y + rect.height <= map_height {
                for y in 0..rect.height {
                    for x in 0..rect.width {
                        map_data[((rect.y + y) * map_width + rect.x + x) as usize] = fill_tile;
                    }
                }
            }
        }
        frag
    }

    pub fn paste_in_map(&self, x: i32, y: i32, map_data: &mut MapData, layer: MapLayer) {
        let (map_width, map_height, map_data, transparent) = match layer {
            MapLayer::Foreground => (map_data.width, map_data.height, &mut map_data.fg_tiles, true),
            MapLayer::Background => (map_data.width, map_data.height, &mut map_data.bg_tiles, true),
            MapLayer::Effects    => (map_data.width, map_data.height, &mut map_data.fx_tiles, true),
            MapLayer::Parallax   => (map_data.para_width, map_data.para_height, &mut map_data.para_tiles, false),
            _ => { return; }
        };

        if (x > 0 &&   x  as u32 >= map_width) || (y > 0 &&   y  as u32 >= map_height) { return; }
        if (x < 0 && (-x) as u32 >= map_width) || (y < 0 && (-y) as u32 >= map_height) { return; }

        let mut src_x = 0;
        let mut src_y = 0;
        let mut width = self.width;
        let mut height = self.height;
        let mut x = x;
        let mut y = y;
        if x < 0 { src_x = (-x) as u32; width -= src_x; x = 0; }
        if y < 0 { src_y = (-y) as u32; height -= src_y; y = 0; }
        let x = x as u32;
        let y = y as u32;
        if width > map_width - x { width = map_width - x; }
        if height > map_height - y { height = map_height - y; }

        for iy in 0..height {
            let src = ((iy + src_y) * self.width + src_x) as usize;
            let dest = ((iy + y) * map_width + x) as usize;
            if transparent {
                for ix in 0..width as usize {
                    let tile = self.data[src+ix];
                    if tile != MapData::NO_TILE {
                        map_data[dest+ix] = tile;
                    }
                }
            } else {
                map_data[dest .. dest + width as usize].clone_from_slice(&self.data[src .. src + width as usize]);
            }
        }
    }

    pub fn get_tile(&self, x: u32, y: u32) -> u8 {
        if x > self.width || y > self.height { return MapData::NO_TILE; }
        self.data[(y * self.width + x) as usize]
    }
}

pub struct MapUndoData {
    pub width: u32,
    pub height: u32,
    pub para_width: u32,
    pub para_height: u32,
    pub fg_tiles: Vec<u8>,
    pub bg_tiles: Vec<u8>,
    pub fx_tiles: Vec<u8>,
    pub para_tiles: Vec<u8>,
}

impl MapUndoData {
    pub fn from_map(map_data: &MapData) -> Self {
        let num_full_tiles = (map_data.width * map_data.height) as usize;
        let mut fg_tiles = Vec::with_capacity(num_full_tiles);
        let mut bg_tiles = Vec::with_capacity(num_full_tiles);
        let mut fx_tiles = Vec::with_capacity(num_full_tiles);
        for y in 0..map_data.height {
            for x in 0..map_data.width {
                let tile_index = (map_data.width * y + x) as usize;
                fg_tiles.push(map_data.fg_tiles[tile_index]);
                bg_tiles.push(map_data.bg_tiles[tile_index]);
                fx_tiles.push(map_data.fx_tiles[tile_index]);
            }
        }
        let num_para_tiles = (map_data.para_width * map_data.para_height) as usize;
        let mut para_tiles = Vec::with_capacity(num_para_tiles);
        for y in 0..map_data.para_height {
            for x in 0..map_data.para_width {
                let tile_index = (map_data.para_width * y + x) as usize;
                para_tiles.push(map_data.para_tiles[tile_index]);
            }
        }
        MapUndoData {
            width: map_data.width,
            height: map_data.height,
            para_width: map_data.para_width,
            para_height: map_data.para_height,
            fg_tiles,
            bg_tiles,
            fx_tiles,
            para_tiles,
        }
    }

    pub fn to_map(&self, map_data: &mut MapData) -> bool {
        if self.width != map_data.width || self.height != map_data.height ||
            self.para_width != map_data.para_width || self.para_height != map_data.para_height {
                return false;
            }
        let map_width = map_data.width as usize;
        for y in 0..map_data.height {
            let tile_index = (map_data.width * y) as usize;
            map_data.fg_tiles[tile_index..tile_index+map_width].copy_from_slice(&self.fg_tiles[tile_index..tile_index+map_width]);
            map_data.bg_tiles[tile_index..tile_index+map_width].copy_from_slice(&self.bg_tiles[tile_index..tile_index+map_width]);
            map_data.fx_tiles[tile_index..tile_index+map_width].copy_from_slice(&self.fx_tiles[tile_index..tile_index+map_width]);
        }
        let map_para_width = map_data.para_width as usize;
        for y in 0..map_data.para_height {
            let tile_index = (map_data.para_width * y) as usize;
            map_data.para_tiles[tile_index..tile_index+map_para_width].copy_from_slice(&self.para_tiles[tile_index..tile_index+map_para_width]);
        }
        true
    }
}

#[derive(Clone)]
pub struct MapWholeFragment {
    pub width: u32,
    pub height: u32,
    pub fg_data: Vec<u8>,
    pub bg_data: Vec<u8>,
    pub fx_data: Vec<u8>,
    pub para_data: Vec<u8>,
}

impl MapWholeFragment {
    pub fn copy_map(map_data: &MapData, rect: MapRect, include_para: bool) -> Option<MapWholeFragment> {
        let map_width = map_data.width;
        let map_height = map_data.height;
        if rect.width == 0 || rect.height == 0 || rect.x + rect.width > map_width || rect.y + rect.height > map_height { return None; }

        let copy_para = include_para && map_data.para_width == map_data.width && map_data.para_height == map_data.height;

        let num_tiles = (rect.width * rect.height) as usize;
        let mut fg_data = Vec::with_capacity(num_tiles);
        let mut bg_data = Vec::with_capacity(num_tiles);
        let mut fx_data = Vec::with_capacity(num_tiles);
        let mut para_data = if copy_para { Vec::with_capacity(num_tiles) } else { Vec::new() };
        for y in 0..rect.height {
            for x in 0..rect.width {
                let tile_index = ((rect.y + y) * map_width + rect.x + x) as usize;
                fg_data.push(map_data.fg_tiles[tile_index]);
                bg_data.push(map_data.bg_tiles[tile_index]);
                fx_data.push(map_data.fx_tiles[tile_index]);
                if copy_para {
                    para_data.push(map_data.bg_tiles[tile_index]);
                }
            }
        }
        Some(MapWholeFragment {
            width: rect.width,
            height: rect.height,
            fg_data,
            bg_data,
            fx_data,
            para_data,
        })
    }

    pub fn cut_map(map_data: &mut MapData, rect: MapRect, fill_tile: u8, include_para: bool) -> Option<MapWholeFragment> {
        let frag = Self::copy_map(map_data, rect, include_para);
        if frag.is_some() {
            let map_width = map_data.width;
            let map_height = map_data.height;
            if rect.width > 0 && rect.height > 0 && rect.x + rect.width <= map_width && rect.y + rect.height <= map_height {
                let erase_para = include_para && map_data.para_width == map_data.width && map_data.para_height == map_data.height;
                for y in 0..rect.height {
                    for x in 0..rect.width {
                        let tile_index = ((rect.y + y) * map_width + rect.x + x) as usize;
                        map_data.fg_tiles[tile_index] = MapData::NO_TILE;
                        map_data.bg_tiles[tile_index] = fill_tile;
                        map_data.fx_tiles[tile_index] = MapData::NO_TILE;
                        if erase_para {
                            map_data.para_tiles[tile_index] = fill_tile;
                        }
                    }
                }
            }
        }
        frag
    }

    pub fn paste_in_map(&self, x: i32, y: i32, map_data: &mut MapData) {
        let map_width = map_data.width;
        let map_height = map_data.height;
        if (x > 0 &&   x  as u32 >= map_width) || (y > 0 &&   y  as u32 >= map_height) { return; }
        if (x < 0 && (-x) as u32 >= map_width) || (y < 0 && (-y) as u32 >= map_height) { return; }

        let mut src_x = 0;
        let mut src_y = 0;
        let mut width = self.width;
        let mut height = self.height;
        let mut x = x;
        let mut y = y;
        if x < 0 { src_x = (-x) as u32; width -= src_x; x = 0; }
        if y < 0 { src_y = (-y) as u32; height -= src_y; y = 0; }
        let x = x as u32;
        let y = y as u32;
        if width > map_width - x { width = map_width - x; }
        if height > map_height - y { height = map_height - y; }

        for iy in 0..height {
            let src = ((iy + src_y) * self.width + src_x) as usize;
            let dest = ((iy + y) * map_width + x) as usize;
            for ix in 0..width as usize {
                let fg_tile = self.fg_data[src+ix]; if fg_tile != MapData::NO_TILE { map_data.fg_tiles[dest+ix] = fg_tile; }
                let bg_tile = self.bg_data[src+ix]; if bg_tile != MapData::NO_TILE { map_data.bg_tiles[dest+ix] = bg_tile; }
                let fx_tile = self.fx_data[src+ix]; if fx_tile != MapData::NO_TILE { map_data.fx_tiles[dest+ix] = fx_tile; }
                if ! self.para_data.is_empty() {
                    map_data.para_tiles[dest+ix] = self.para_data[src+ix];
                }
            }
        }
    }

    pub fn get_layer_tile(&self, x: u32, y: u32, layer: MapLayer) -> u8 {
        if x > self.width || y > self.height { return MapData::NO_TILE; }
        match layer {
            MapLayer::Foreground => { self.fg_data[(y * self.width + x) as usize] }
            MapLayer::Background => { self.bg_data[(y * self.width + x) as usize] }
            MapLayer::Effects    => { self.fx_data[(y * self.width + x) as usize] }
            MapLayer::Parallax => {
                if self.para_data.is_empty() { return MapData::NO_TILE; }
                self.para_data[(y * self.width + x) as usize]
            }
            _ => MapData::NO_TILE
        }
    }
}

pub enum MapClipboardData {
    Empty,
    MapLayerFragment(MapLayerFragment),
    MapWholeFragment(MapWholeFragment),
}

impl MapClipboardData {
    pub fn is_none(&self) -> bool {
        matches!(self, MapClipboardData::Empty)
    }

    pub fn take(&mut self) -> MapClipboardData {
        std::mem::replace(self, MapClipboardData::Empty)
    }
}

pub trait MapTileFixer {
    fn get_tile_planes_mut(&mut self) -> Vec<&mut [u8]>;

    fn add_hole(&mut self, tile_index: u8, num_tiles: u8) {
        fn add_plane_hole(tiles: &mut [u8], tile_index: u8, num_tiles: u8) {
            for tile in tiles {
                if *tile >= tile_index {
                    *tile = (*tile).saturating_add(num_tiles);
                }
            }
        }
        for plane in self.get_tile_planes_mut() {
            add_plane_hole(plane, tile_index, num_tiles);
        }
    }

    fn remove_hole(&mut self, tile_index: u8, num_tiles: u8) {
        fn rm_plane_hole(tiles: &mut [u8], tile_index: u8, num_tiles: u8) {
            for tile in tiles {
                if *tile >= tile_index + num_tiles && *tile != MapData::NO_TILE {
                    *tile = (*tile).saturating_sub(num_tiles);
                }
            }
        }
        for plane in self.get_tile_planes_mut() {
            rm_plane_hole(plane, tile_index, num_tiles);
        }
    }
}

impl MapTileFixer for MapData {
    fn get_tile_planes_mut(&mut self) -> Vec<&mut [u8]> {
        vec![ &mut self.fg_tiles, &mut self.bg_tiles, &mut self.para_tiles ]
    }
}

pub fn fix_maps_after_tiles_added(maps: &mut AssetList<MapData>, tileset_id: DataAssetId, tile_index: u8, num_tiles: u8) {
    for map_data in maps.iter_mut() {
        if map_data.tileset_id == tileset_id {
            map_data.add_hole(tile_index, num_tiles);
        }
    }
}

pub fn fix_maps_after_tiles_removed(maps: &mut AssetList<MapData>, tileset_id: DataAssetId, tile_index: u8, num_tiles: u8) {
    for map_data in maps.iter_mut() {
        if map_data.tileset_id == tileset_id {
            map_data.remove_hole(tile_index, num_tiles);
        }
    }
}
