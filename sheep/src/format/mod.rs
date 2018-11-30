#[cfg(feature = "amethyst")]
pub mod amethyst;

use {Sprite, SpriteAnchor};

pub trait Format {
    fn encode(dimensions: (u32, u32), sprites: &[(Sprite, SpriteAnchor)]) -> Self;
}
