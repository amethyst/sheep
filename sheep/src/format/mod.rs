#[cfg(feature = "amethyst")]
pub mod amethyst;

pub mod named;

use SpriteAnchor;

pub trait Format {
    type Data;
    type Options;

    fn encode(
        dimensions: (u32, u32),
        sprites: &[SpriteAnchor],
        options: Self::Options,
    ) -> Self::Data;
}
