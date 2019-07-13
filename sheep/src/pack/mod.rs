pub mod maxrects;
pub mod simple;

use {SpriteAnchor, SpriteData};

#[derive(Debug, Clone)]
pub struct PackerResult {
    pub dimensions: (u32, u32),
    pub anchors: Vec<SpriteAnchor>,
}

pub trait Packer {
    type Options;

    fn pack(sprites: &[SpriteData], options: Self::Options) -> Vec<PackerResult>;
}
