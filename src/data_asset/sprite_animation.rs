#[derive(Clone)]
pub struct SpriteAnimationFrame {
    pub head_index: Option<u8>,
    pub foot_index: Option<u8>,
}

#[allow(unused)]
#[derive(Clone)]
pub struct SpriteAnimationLoop {
    pub name: String,
    pub frame_indices: Vec<SpriteAnimationFrame>,
}

#[allow(unused)]
pub struct SpriteAnimation {
    pub asset: super::DataAsset,
    pub sprite_id: super::DataAssetId,
    pub clip_rect: super::Rect,
    pub foot_overlap: i8,
    pub loops: Vec<SpriteAnimationLoop>,
}

#[derive(Clone)]
pub struct LoopCreationData {
    pub offset: u16,
    pub len: u16,
}

pub struct CreationData<'a> {
    pub sprite_id: super::DataAssetId,
    pub clip_rect: super::Rect,
    pub use_foot_frames: bool,
    pub foot_overlap: i8,
    pub frame_indices: &'a [u8],
    pub loops: &'a [LoopCreationData],
}

impl SpriteAnimation {
    pub const NUM_LOOPS: usize = 20;

    pub fn new(asset: super::DataAsset, sprite_id: super::DataAssetId) -> Self {
        let mut loops = Vec::new();
        for loop_index in 0..Self::NUM_LOOPS {
            loops.push(SpriteAnimationLoop {
                name: format!("loop {}", loop_index),
                frame_indices: if loop_index == 0 {
                    vec![SpriteAnimationFrame { head_index: Some(0), foot_index: None }]
                } else {
                    Vec::new()
                },
            });
        }

        SpriteAnimation {
            asset,
            sprite_id,
            clip_rect: super::Rect::new(0, 0, 0, 0),
            foot_overlap: 0,
            loops,
        }
    }

    pub fn from_data(asset: super::DataAsset, data: CreationData) -> Self {
        let mut loops = Vec::new();
        for loop_data in data.loops {
            let mut frame_indices = Vec::new();
            for frame_index in 0..loop_data.len {
                let src_offset = (loop_data.offset + if data.use_foot_frames { 2*frame_index } else { frame_index }) as usize;
                let head_index = if data.frame_indices[src_offset] != 0xff {
                    Some(data.frame_indices[src_offset])
                } else { None };
                let foot_index = if data.use_foot_frames && data.frame_indices[src_offset+1] != 0xff {
                    Some(data.frame_indices[src_offset+1])
                } else { None };
                frame_indices.push(SpriteAnimationFrame { head_index, foot_index });
            }
            loops.push(SpriteAnimationLoop {
                name: String::new(),
                frame_indices,
            });
        }
        SpriteAnimation {
            asset,
            sprite_id: data.sprite_id,
            clip_rect: data.clip_rect,
            foot_overlap: data.foot_overlap,
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

impl super::GenericAsset for SpriteAnimation {
    //fn asset(&self) -> &super::DataAsset { &self.asset }
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
