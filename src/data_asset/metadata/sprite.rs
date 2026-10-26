#[derive(Clone)]
pub struct SpriteFrameCollection {
    pub name: String,
    pub indices: Vec<u32>,
}

impl SpriteFrameCollection {
    pub fn new(name: impl Into<String>) -> Self {
        SpriteFrameCollection {
            name: name.into(),
            indices: Vec::new(),
        }
    }
}

pub struct SpriteMetadata {
    pub frames: Vec<SpriteFrameCollection>,
}

impl SpriteMetadata {
    pub fn new() -> Self {
        SpriteMetadata {
            frames: Vec::new(),
        }
    }
}
