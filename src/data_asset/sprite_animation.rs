#[derive(Clone, std::hash::Hash)]
pub struct SpriteAnimationFrame {
    pub head_index: Option<u8>,
    pub foot_index: Option<u8>,
}

#[derive(Clone, std::hash::Hash)]
pub struct SpriteAnimationLoop {
    pub name_id: String,
    pub frame_indices: Vec<SpriteAnimationFrame>,
}

#[derive(std::hash::Hash)]
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

        // loop: offset(2) + length(2)
        let loop_size = 2usize + 2usize;

        // header: frame_indices<ptr>(4) + sprite<ptr>(4) + collision +
        //         use_foot_frames(1) + foot_overlap(1) + pad(2) + 20*loop
        let header =  4usize + 4usize + collision + 1usize + 1usize + 2usize + 20usize * loop_size;

        // frames:
        let frames = self.loops.iter().fold(0, |acc, l| acc + l.frame_indices.len());

        header + frames * if self.use_foot_frames() { 2usize } else { 1usize }
    }
}
