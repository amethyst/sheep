#[cfg(feature = "amethyst")]
pub mod amethyst;

use SpriteAnchor;

pub trait Format {
    type Data;

    fn encode(dimensions: (u32, u32), sprites: &[SpriteAnchor]) -> Self::Data;
}
