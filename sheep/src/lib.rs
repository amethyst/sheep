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

use sprite::{create_pixel_buffer, write_sprite};

#[allow(dead_code)]
pub struct SpriteSheet<F: Format> {
    bytes: Vec<u8>,
    stride: usize,
    dimensions: (u32, u32),
    data: F,
}

pub fn process<P, F>(input: Vec<InputSprite>, stride: usize) -> SpriteSheet<F>
where
    P: Packer,
    F: Format,
{
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
    let sprites_with_anchors = sprites
        .into_iter()
        .map(|sprite| {
            let anchor_idx = packer_result
                .anchors
                .binary_search_by_key(&sprite.data.id, |it| it.id)
                .expect("Should have found anchor for sprite");

            (sprite, packer_result.anchors[anchor_idx])
        }).collect::<Vec<(Sprite, SpriteAnchor)>>();

    let format_result = F::encode(packer_result.dimensions, &sprites_with_anchors);

    let mut buffer = create_pixel_buffer(packer_result.dimensions, stride);
    for (sprite, anchor) in sprites_with_anchors {
        write_sprite(
            &mut buffer,
            packer_result.dimensions,
            stride,
            &sprite,
            &anchor,
        );
    }

    SpriteSheet {
        bytes: buffer,
        stride: stride,
        dimensions: packer_result.dimensions,
        data: format_result,
    }
}
