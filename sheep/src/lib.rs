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
    pack::{
        maxrects::{MaxrectsOptions, MaxrectsPacker},
        simple::SimplePacker,
        Packer, PackerResult,
    },
    sprite::{InputSprite, Sprite, SpriteAnchor, SpriteData},
};

#[cfg(feature = "amethyst")]
pub use format::amethyst::{AmethystFormat, SerializedSpriteSheet, SpritePosition};
#[cfg(feature = "amethyst")]
pub use format::named::AmethystNamedFormat;

use sprite::{create_pixel_buffer, write_sprite};

#[derive(Debug, Clone)]
pub struct SpriteSheet {
    pub bytes: Vec<u8>,
    pub stride: usize,
    pub dimensions: (u32, u32),
    anchors: Vec<SpriteAnchor>,
}

pub fn pack<P: Packer>(
    input: Vec<InputSprite>,
    stride: usize,
    options: P::Options,
) -> Vec<SpriteSheet> {
    let sprites = input
        .into_iter()
        .enumerate()
        .map(|(idx, sprite)| Sprite::from_input(idx, sprite))
        .collect::<Vec<Sprite>>();

    let sprite_data = sprites
        .iter()
        .map(|it| it.data)
        .collect::<Vec<SpriteData>>();

    let packer_result = P::pack(&sprite_data, options);

    packer_result
        .into_iter()
        .map(|sheet| {
            let mut buffer = create_pixel_buffer(sheet.dimensions, stride);
            sprites
                .iter()
                .filter_map(|sprite| {
                    sheet
                        .anchors
                        .iter()
                        .find(|anchor| anchor.id == sprite.data.id)
                        .map(|anchor| (sprite, anchor))
                })
                .for_each(|(sprite, anchor)| {
                    write_sprite(&mut buffer, sheet.dimensions, stride, &sprite, &anchor);
                });

            SpriteSheet {
                bytes: buffer,
                stride: stride,
                dimensions: sheet.dimensions,
                anchors: sheet.anchors,
            }
        })
        .collect()
}

pub fn encode<F>(sprite_sheet: &SpriteSheet, options: F::Options) -> F::Data
where
    F: Format,
{
    F::encode(sprite_sheet.dimensions, &sprite_sheet.anchors, options)
}
