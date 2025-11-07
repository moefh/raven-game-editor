#[allow(unused)]
#[derive(Clone)]
pub struct SpriteAnimationLoop {
    pub name: String,
    pub offset: u16,
    pub len: u16,
}

#[allow(unused)]
pub struct SpriteAnimation {
    pub asset: super::DataAsset,
    pub sprite_id: super::DataAssetId,
    pub clip_rect: super::Rect,
    pub use_foot_frames: bool,
    pub foot_overlap: i8,
    pub frame_indices: Vec<u8>,
    pub loops: Vec<SpriteAnimationLoop>,
}

pub struct CreationData<'a> {
    pub sprite_id: super::DataAssetId,
    pub clip_rect: super::Rect,
    pub use_foot_frames: bool,
    pub foot_overlap: i8,
    pub frame_indices: &'a [u8],
    pub loops: &'a [SpriteAnimationLoop],
}

impl SpriteAnimation {
    pub fn new(asset: super::DataAsset, sprite_id: super::DataAssetId) -> Self {
        SpriteAnimation {
            asset,
            sprite_id,
            clip_rect: super::Rect::new(0, 0, 0, 0),
            use_foot_frames: false,
            foot_overlap: 0,
            frame_indices: Vec::new(),
            loops: Vec::new(),
        }
    }

    pub fn from_data(asset: super::DataAsset, data: CreationData) -> Self {
        SpriteAnimation {
            asset,
            sprite_id: data.sprite_id,
            clip_rect: data.clip_rect,
            use_foot_frames: data.use_foot_frames,
            foot_overlap: data.foot_overlap,
            frame_indices: Vec::from(data.frame_indices),
            loops: Vec::from(data.loops),
        }
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
        let frames = self.loops.iter().fold(0, |max, l| max.max(l.offset + l.len)) as usize;

        header + frames * if self.use_foot_frames { 2usize } else { 1usize }
    }
}
