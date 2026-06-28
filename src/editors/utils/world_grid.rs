use crate::data_asset::{
    World,
    WorldRegion,
};

pub struct Grid {
    pub width: i32,
    pub height: i32,
    borders: Vec<u8>,
}

impl Grid {
    pub const BORDER_TOP: u8 = 1 << 0;
    pub const BORDER_LEFT: u8 = 1 << 1;

    fn new() -> Self {
        Grid {
            width: 0,
            height: 0,
            borders: Vec::new()
        }
    }

    fn calc<T: std::cmp::PartialEq, F: Fn(i32, i32) -> Option<T>>(&mut self, width: i32, height: i32, get_block: F) {
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

    pub fn get_block_borders(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            0
        } else {
            self.borders[(y * self.width + x) as usize]
        }
    }
}

pub struct WorldGridStore {
    world_hash: u64,
    pub world_grid: Grid,
    pub region_grids: Vec<Grid>,
}

impl WorldGridStore {
    pub fn new() -> Self {
        WorldGridStore {
            world_hash: 0,
            world_grid: Grid::new(),
            region_grids: Vec::new(),
        }
    }

    pub fn update(&mut self, world: &World) {
        use std::hash::{Hash, Hasher};

        let mut hasher = std::hash::DefaultHasher::new();
        world.hash(&mut hasher);
        let hash = hasher.finish();
        if self.world_hash != hash {
            self.world_hash = hash;
            self.calc(world);
        }
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
            if bx >= 0 && by >= 0 && bx < rw && by < rh {
                return region.blocks[(by * WorldRegion::BLOCK_STRIDE as i32 + bx) as usize].map(|block| {
                    block as u32 | ((region_index as u32) << 8)
                });
            }
        }
        None
    }

    fn calc(&mut self, world: &World) {
        // world
        let (world_width, world_height) = world.regions.iter().fold((0, 0), |size, region| {
            (
                size.0.max(region.x as i32 + region.width as i32),
                size.1.max(region.y as i32 + region.height as i32),
            )
        });
        self.world_grid.calc(world_width + 1, world_height + 1, |x, y| { Self::get_world_block(x, y, world) });

        // regions
        if self.region_grids.len() < world.regions.len() {
            self.region_grids.resize_with(world.regions.len(), Grid::new);
        }
        for (grid, region) in self.region_grids.iter_mut().zip(world.regions.iter()) {
            let width = region.width as i32 + 1;
            let height = region.height as i32 + 1;
            grid.calc(width, height, |x, y| { Self::get_region_block(x, y, region) });
        }
    }
}
