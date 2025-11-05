#[allow(unused)]
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
    
}
