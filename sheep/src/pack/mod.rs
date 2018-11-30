pub mod simple;

use {SpriteAnchor, SpriteData};

impl SpriteAnchor {
    pub fn new(id: usize, position: (u32, u32)) -> Self {
        SpriteAnchor { id, position }
    }
}

#[derive(Debug, Clone)]
pub struct PackerResult {
    pub dimensions: (u32, u32),
    pub anchors: Vec<SpriteAnchor>,
}

pub trait Packer {
    fn pack(sprites: &[SpriteData]) -> PackerResult;
}
