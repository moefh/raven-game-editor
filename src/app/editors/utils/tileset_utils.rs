use std::collections::HashSet;

use crate::image::colors;
use crate::data_asset::{
    DataAssetId,
    DataAssetStore,
    Tileset,
    MapData,
    TileAnimation,
};
use crate::image::{
    ImageCollection,
    ImagePixels,
    TextureNameId,
    TextureSlot,
};

use super::{
    MapTileFixer,
    AssetIdHolder,
};
use super::super::{
    resize_map_tiles,
    WindowContext,
    EditorStore,
};

pub struct TileGrid {
    pub width: u32,
    pub height: u32,
    fg_tiles: Vec<u8>,
    buf_tiles: Vec<u8>,
    buf_set: HashSet<u8>,
    tileset_hash: u64,
    image: TileGridImage,
}

impl TileGrid {
    pub fn new(tileset_id: DataAssetId) -> Self {
        let width = 2;
        let height = 2;
        TileGrid {
            width,
            height,
            fg_tiles: vec![MapData::NO_TILE; (width * height) as usize],
            buf_tiles: vec![0; (width * height) as usize],
            buf_set: HashSet::new(),
            tileset_hash: 0,
            image: TileGridImage::new(tileset_id, width, height),
        }
    }

    fn calc_tileset_hash(tileset: &Tileset) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::hash::DefaultHasher::new();
        tileset.hash(&mut hasher);
        hasher.finish()
    }

    pub fn get_image<'a>(&'a mut self, tileset: &Tileset) -> &'a TileGridImage {
        let tileset_hash = Self::calc_tileset_hash(tileset);
        if self.tileset_hash != tileset_hash {
            self.tileset_hash = tileset_hash;
            self.tileset_to_image(tileset);
        }
        &self.image
    }

    pub fn get_image_mut<'a>(&'a mut self, tileset: &Tileset) -> &'a mut TileGridImage {
        let tileset_hash = Self::calc_tileset_hash(tileset);
        if self.tileset_hash != tileset_hash {
            self.tileset_hash = tileset_hash;
            self.tileset_to_image(tileset);
        }
        &mut self.image
    }

    pub fn resize(&mut self, tileset: &Tileset, width: u32, height: u32) {
        resize_map_tiles(&mut self.fg_tiles, self.width, self.height, width, height, MapData::NO_TILE);
        resize_map_tiles(&mut self.buf_tiles, self.width, self.height, width, height, 0);
        self.width = width;
        self.height = height;
        self.tileset_to_image(tileset); // will resize image to match grid
    }

    pub fn set_tile(&mut self, x: u32, y: u32, tile: u8) {
        if x >= self.width || y >= self.height { return; }
        self.fg_tiles[((y * self.width) + x) as usize] = tile;
    }

    pub fn get_tile(&self, x: u32, y: u32) -> u8 {
        if x >= self.width || y >= self.height {
            MapData::NO_TILE
        } else {
            self.fg_tiles[((y * self.width) + x) as usize]
        }
    }

    fn copy_tile(dest: &mut [u8], dst_stride: usize, src: &[u8], src_stride: usize) {
        let size = Tileset::TILE_SIZE as usize;
        for y in 0..size {
            let dst_index = y * dst_stride;
            let src_index = y * src_stride;
            dest[dst_index..dst_index+size].copy_from_slice(&src[src_index..src_index+size]);
        }
    }

    fn tiles_are_different(tile1: &[u8], stride1: usize, tile2: &[u8], stride2: usize) -> bool {
        let size = Tileset::TILE_SIZE as usize;
        for y in 0..size {
            let index1 = y * stride1;
            let index2 = y * stride2;
            for x in 0..size {
                if tile1[index1 + x] != tile2[index2+x] {
                    return true;
                }
            }
        }
        false
    }

    fn clear_tile(tile: &mut [u8], stride: usize, color: u8) -> bool {
        let size = Tileset::TILE_SIZE as usize;
        for y in 0..size {
            let index = y * stride;
            tile[index..index+size].fill(color);
        }
        false
    }

    pub fn image_to_tileset(&mut self, tileset: &mut Tileset) {
        let size = Tileset::TILE_SIZE as usize;
        let num_tiles_x = self.width as usize;
        let num_tiles_y = self.height as usize;
        let img_width = self.image.pixels.width as usize;

        // calculate which tiles need copying
        let mut has_repeated_tiles = false;
        self.buf_tiles[..].fill(0);
        self.buf_set.clear();
        for y in 0..num_tiles_y {
            for x in 0..num_tiles_x {
                let fg_tile = self.fg_tiles[num_tiles_x * y + x];
                let img_pos = (img_width * y + x) * size;
                if fg_tile == MapData::NO_TILE {
                    // ensure image with no associted tile remains empty
                    Self::clear_tile(&mut self.image.pixels.data[img_pos..], img_width, colors::TRANSPARENT);
                } else {
                    if self.buf_set.contains(&fg_tile) { has_repeated_tiles = true; }
                    self.buf_set.insert(fg_tile);
                    if Self::tiles_are_different(&tileset.data[size*size * (fg_tile as usize)..], size,
                                                 &self.image.pixels.data[img_pos..], img_width) {
                        self.buf_tiles[num_tiles_x * y + x] = 1;
                    }
                }
            }
        }

        // copy tiles
        for y in 0..num_tiles_y {
            for x in 0..num_tiles_x {
                let fg_tile = self.fg_tiles[num_tiles_x * y + x];
                if fg_tile != MapData::NO_TILE && self.buf_tiles[num_tiles_x * y + x] != 0 {
                    let img_pos = (img_width * y + x) * size;
                    Self::copy_tile(&mut tileset.data[size*size * (fg_tile as usize)..], size,
                                    &self.image.pixels.data[img_pos..], img_width);
                }
            }
        }
        self.tileset_hash = Self::calc_tileset_hash(tileset);
        if has_repeated_tiles {
            self.tileset_to_image(tileset);
        }
    }

    pub fn tileset_to_image(&mut self, tileset: &Tileset) {
        let req_img_width = self.width * Tileset::TILE_SIZE;
        let req_img_height = self.height * Tileset::TILE_SIZE;
        if self.image.pixels.width != req_img_width ||
            self.image.pixels.height != req_img_height {
                self.image.resize(req_img_width, req_img_height, 1, colors::TRANSPARENT);
            }

        self.image.pixels.data[..].fill(colors::TRANSPARENT);

        let size = Tileset::TILE_SIZE as usize;
        let num_tiles_x = self.width as usize;
        let num_tiles_y = self.height as usize;
        let img_width = self.image.pixels.width as usize;
        for y in 0..num_tiles_y {
            for x in 0..num_tiles_x {
                let fg_tile = self.fg_tiles[num_tiles_x * y + x];
                if fg_tile != MapData::NO_TILE {
                    let img_pos = (img_width * y + x) * size;
                    Self::copy_tile(&mut self.image.pixels.data[img_pos..], img_width,
                                    &tileset.data[size*size * (fg_tile as usize)..], size);
                }
            }
        }
    }
}

impl MapTileFixer for TileGrid {
    fn get_tile_planes_mut(&mut self) -> Vec<&mut [u8]> {
        self.tileset_hash = 0;  // force re-creation of image next time it's used
        vec![ &mut self.fg_tiles ]
    }
}

pub struct TileGridImage {
    tileset_id: DataAssetId,
    pixels: ImagePixels,
}

impl TileGridImage {
    fn new(tileset_id: DataAssetId, width: u32, height: u32) -> Self {
        let data = vec![0; (width * height * Tileset::TILE_SIZE*Tileset::TILE_SIZE) as usize];
        TileGridImage {
            tileset_id,
            pixels: ImagePixels::new(width, height, data),
        }
    }
}

impl AssetIdHolder for TileGridImage {
    fn get_asset_id(&self) -> DataAssetId { self.tileset_id }
}

impl ImageCollection for TileGridImage {
    fn texture_name_id(&self) -> TextureNameId { TextureNameId::Asset(self.tileset_id) }
    fn texture_slot(&self, transparent: bool, float: bool) -> TextureSlot {
        let num = if float { 1 } else { 0 };
        if transparent {
            TextureSlot::CustomTransparent(num)
        } else {
            TextureSlot::CustomOpaque(num)
        }
    }

    fn width(&self) -> u32 { self.pixels.width }
    fn height(&self) -> u32 { self.pixels.height }
    fn num_items(&self) -> u32 { 1 }
    fn set_width(&mut self, width: u32) { self.pixels.width = width }
    fn set_height(&mut self, height: u32) { self.pixels.height = height; }
    fn set_num_items(&mut self, _num_items: u32) { }
    fn data(&self) -> &Vec<u8> { &self.pixels.data }
    fn data_mut(&mut self) -> &mut Vec<u8> { &mut self.pixels.data }
}

pub trait TilesetTileFixer {
    fn move_tile(&mut self, tileset_id: DataAssetId, old_index: u8, new_index: u8);

    fn add_tileset_hole(&mut self, tileset_id: DataAssetId, hole_start: u8, hole_size: u8, num_tiles_after_hole: u8) {
        for index in (0..num_tiles_after_hole).rev() {
            self.move_tile(tileset_id, hole_start + index, hole_start + hole_size + index);
        }
    }

    fn remove_tileset_hole(&mut self, tileset_id: DataAssetId, hole_start: u8, hole_size: u8, num_tiles_after_hole: u8) {
        for index in 0..num_tiles_after_hole {
            self.move_tile(tileset_id, hole_start + hole_size + index, hole_start + index);
        }
    }
}

impl TilesetTileFixer for TileAnimation {
    fn move_tile(&mut self, tileset_id: DataAssetId, old_index: u8, new_index: u8) {
        if self.parent_tileset_id == tileset_id {
            self.loops[new_index as usize] = self.loops[old_index as usize];
        }
        if self.anim_tileset_id == tileset_id {
            for tloop in self.loops.iter_mut() {
                if tloop.start == old_index {
                    tloop.start = new_index;
                }
            }
        }
    }
}

pub fn fix_after_tileset_tiles_added(
    wc: &mut WindowContext,
    store: &mut DataAssetStore,
    editors: &mut EditorStore,
    tileset_id: DataAssetId,
    hole_start: u8,
    hole_size: u8,
    num_tiles_after_hole: u8
) {
    if let Some(tileset_editor) = editors.tilesets.get_mut(&tileset_id) {
        TilesetTileFixer::add_tileset_hole(tileset_editor, tileset_id, hole_start, hole_size, num_tiles_after_hole);
        MapTileFixer::add_tileset_hole(tileset_editor, hole_start, hole_size, num_tiles_after_hole);
    }
    for map_id in store.asset_ids.maps.iter() {
        if let Some(map_data) = store.assets.maps.get_mut(map_id) && map_data.tileset_id == tileset_id {
            map_data.add_tileset_hole(hole_start, hole_size, num_tiles_after_hole);
            if let Some(map_editor) = editors.maps.get_mut(map_id) {
                map_editor.add_tileset_hole(hole_start, hole_size, num_tiles_after_hole);
            }
        }
    }
    for tile_anim_id in store.asset_ids.tile_anims.iter() {
        if let Some(tile_anim) = store.assets.tile_anims.get_mut(tile_anim_id) &&
            (tile_anim.parent_tileset_id == tileset_id || tile_anim.anim_tileset_id == tileset_id) {
                tile_anim.add_tileset_hole(tileset_id, hole_start, hole_size, num_tiles_after_hole);
                if let Some(tile_anim_editor) = editors.tile_anims.get_mut(tile_anim_id) {
                    tile_anim_editor.add_tileset_hole(tileset_id, hole_start, hole_size, num_tiles_after_hole);
                }
            }
    }
    if let Some(tileset) = store.assets.tilesets.get_mut(&tileset_id) {
        tileset.load_texture(wc.tex_man, wc.egui.ctx, tileset.texture_slot(false, false), true);
        tileset.load_texture(wc.tex_man, wc.egui.ctx, tileset.texture_slot(true, false), true);
    }
}

pub fn fix_after_tileset_tiles_removed(
    wc: &mut WindowContext,
    store: &mut DataAssetStore,
    editors: &mut EditorStore,
    tileset_id: DataAssetId,
    hole_start: u8,
    hole_size: u8,
    num_tiles_after_hole: u8
) {
    if let Some(tileset_editor) = editors.tilesets.get_mut(&tileset_id) {
        TilesetTileFixer::remove_tileset_hole(tileset_editor, tileset_id, hole_start, hole_size, num_tiles_after_hole);
        MapTileFixer::remove_tileset_hole(tileset_editor, hole_start, hole_size, num_tiles_after_hole);
    }
    for map_id in store.asset_ids.maps.iter() {
        if let Some(map_data) = store.assets.maps.get_mut(map_id) && map_data.tileset_id == tileset_id {
            map_data.remove_tileset_hole(hole_start, hole_size, num_tiles_after_hole);
            if let Some(map_editor) = editors.maps.get_mut(map_id) {
                map_editor.remove_tileset_hole(hole_start, hole_size, num_tiles_after_hole);
            }
        }
    }
    if let Some(tileset) = store.assets.tilesets.get_mut(&tileset_id) {
        tileset.load_texture(wc.tex_man, wc.egui.ctx, tileset.texture_slot(false, false), true);
        tileset.load_texture(wc.tex_man, wc.egui.ctx, tileset.texture_slot(true, false), true);
    }
}
