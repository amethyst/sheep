pub mod amethyst;

use SpriteAnchor;

pub trait Format {
    fn encode(dimensions: (u32, u32), anchors: &[SpriteAnchor]) -> Self;
}
