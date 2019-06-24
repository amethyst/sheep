#[cfg(feature = "amethyst")]
extern crate serde;

#[cfg(feature = "amethyst")]
#[macro_use]
extern crate serde_derive;

mod format;
mod pack;
mod sprite;

pub use {
    format::Format,
    pack::{simple::SimplePacker, Packer, PackerResult},
    sprite::{InputSprite, Sprite, SpriteAnchor, SpriteData},
};

#[cfg(feature = "amethyst")]
pub use format::amethyst::{AmethystFormat, SerializedSpriteSheet, SpritePosition};
pub use format::named::AmethystNamedFormat;

use sprite::{create_pixel_buffer, write_sprite};

#[allow(dead_code)]
pub struct SpriteSheet {
    pub bytes: Vec<u8>,
    pub stride: usize,
    pub dimensions: (u32, u32),
    anchors: Vec<SpriteAnchor>,
}

pub fn pack<P: Packer>(input: Vec<InputSprite>, stride: usize) -> SpriteSheet {
    let sprites = input
        .into_iter()
        .enumerate()
        .map(|(idx, sprite)| Sprite::from_input(idx, sprite))
        .collect::<Vec<Sprite>>();

    let sprite_data = sprites
        .iter()
        .map(|it| it.data)
        .collect::<Vec<SpriteData>>();

    let packer_result = P::pack(&sprite_data);
    let mut buffer = create_pixel_buffer(packer_result.dimensions, stride);
    sprites.into_iter().for_each(|sprite| {
        let anchor = packer_result
            .anchors
            .iter()
            .find(|it| it.id == sprite.data.id)
            .expect("Should have found anchor for sprite");
        write_sprite(
            &mut buffer,
            packer_result.dimensions,
            stride,
            &sprite,
            &anchor,
        );
    });

    SpriteSheet {
        bytes: buffer,
        stride: stride,
        dimensions: packer_result.dimensions,
        anchors: packer_result.anchors,
    }
}

pub fn encode<F>(sprite_sheet: &SpriteSheet, options: F::Options) -> F::Data
where
    F: Format,
{
    F::encode(sprite_sheet.dimensions, &sprite_sheet.anchors, options)
}
