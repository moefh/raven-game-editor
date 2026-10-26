#[derive(Clone)]
pub struct SpriteAnimationFrame {
    pub head_index: Option<u8>,
    pub foot_index: Option<u8>,
}

#[derive(Clone)]
pub struct SpriteAnimationLoop {
    pub name_id: String,
    pub frame_indices: Vec<SpriteAnimationFrame>,
    pub dont_loop: bool,
    pub frame_speed: u16, // 1-256
}

pub struct SpriteAnimation {
    pub asset: super::DataAsset,
    pub sprite_id: super::DataAssetId,
    pub clip_rect: super::Rect,
    pub foot_overlap: i8,
    pub loops: Vec<SpriteAnimationLoop>,
}

impl SpriteAnimation {
    pub const NUM_LOOPS: usize = 20;

    pub fn new(id: super::DataAssetId, name: String, sprite_id: super::DataAssetId) -> Self {
        let mut loops = Vec::new();
        for loop_index in 0..Self::NUM_LOOPS {
            loops.push(SpriteAnimationLoop {
                name_id: format!("loop {}", loop_index),
                frame_indices: if loop_index == 0 {
                    vec![SpriteAnimationFrame { head_index: Some(0), foot_index: None }]
                } else {
                    Vec::new()
                },
                dont_loop: false,
                frame_speed: 64,
            });
        }

        SpriteAnimation {
            asset: super::DataAsset::new(super::DataAssetType::SpriteAnimation, id, name),
            sprite_id,
            clip_rect: super::Rect::new(0, 0, 0, 0),
            foot_overlap: 0,
            loops,
        }
    }

    pub fn use_foot_frames(&self) -> bool {
        for aloop in &self.loops {
            for frame in &aloop.frame_indices {
                if frame.foot_index.is_some() {
                    return true;
                }
            }
        }
        false
    }
}

impl super::HashedData for SpriteAnimation {
    fn data_hash(&self, hasher: &mut super::DataHasher) {
        hasher.add_data_asset(&self.asset);
        hasher.add_asset_id(self.sprite_id);
        hasher.add_i32(self.clip_rect.x);
        hasher.add_i32(self.clip_rect.y);
        hasher.add_i32(self.clip_rect.w);
        hasher.add_i32(self.clip_rect.h);
        hasher.add_i8(self.foot_overlap);
        for aloop in &self.loops {
            hasher.add_str(&aloop.name_id);
            for frame in &aloop.frame_indices {
                match frame.head_index {
                    Some(n) => { hasher.add_u16(n as u16); }
                    None    => { hasher.add_u16(0xff00); }
                }
                match frame.foot_index {
                    Some(n) => { hasher.add_u16(n as u16); }
                    None    => { hasher.add_u16(0xff00); }
                }
            }
            hasher.add_bool(aloop.dont_loop);
            hasher.add_u16(aloop.frame_speed);
        }
    }
}

impl super::DuplicableAsset<SpriteAnimation> for SpriteAnimation {
    fn duplicate(&self, dup_id: super::DataAssetId, dup_name: String) -> Self {
        SpriteAnimation {
            asset: self.asset.duplicate(dup_id, dup_name),
            sprite_id: self.sprite_id,
            clip_rect: self.clip_rect,
            foot_overlap: self.foot_overlap,
            loops: self.loops.clone(),
        }
    }
}

impl super::GenericAsset for SpriteAnimation {
    fn asset(&self) -> &super::DataAsset { &self.asset }

    fn data_size(&self) -> usize {
        // collision: x(2) + y(2) + w(2) + h(2)
        let collision = 4usize * 2usize;

        // loop: offset(2) + length(2) + dont_loop(1) + frame_speed(1)
        let loop_size = 2usize + 2usize + 1usize + 1usize;

        // header: frame_indices<ptr>(4) + sprite<ptr>(4) + collision +
        //         use_foot_frames(1) + foot_overlap(1) + pad(2) + 20*loop
        let header =  4usize + 4usize + collision + 1usize + 1usize + 2usize + 20usize * loop_size;

        // frames:
        let frames = self.loops.iter().fold(0, |acc, l| acc + l.frame_indices.len());

        header + frames * if self.use_foot_frames() { 2usize } else { 1usize }
    }
}
