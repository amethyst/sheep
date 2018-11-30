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
    let sprites_with_anchors = sprites
        .into_iter()
        .map(|sprite| {
            let anchor_idx = packer_result
                .anchors
                .binary_search_by_key(&sprite.data.id, |it| it.id)
                .expect("Should have found anchor for sprite");

            (sprite, packer_result.anchors[anchor_idx])
        }).collect::<Vec<(Sprite, SpriteAnchor)>>();

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
        anchors: packer_result.anchors,
    }
}

pub fn encode<F>(sprite_sheet: &SpriteSheet) -> F
where
    F: Format,
{
    F::encode(sprite_sheet.dimensions, &sprite_sheet.anchors)
}
