#[cfg(feature = "amethyst")]
extern crate serde;

#[cfg(feature = "amethyst")]
#[macro_use]
extern crate serde_derive;

extern crate smallvec;
extern crate twox_hash;

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
pub use format::named::{AmethystNamedFormat, SerializedNamedSpriteSheet};

use sprite::{create_pixel_buffer, write_sprite};

use smallvec::SmallVec;
use std::collections::hash_map::HashMap;
use std::hash::BuildHasherDefault;
use twox_hash::XxHash64;

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
    let mut hashes: HashMap<&[u8], usize, BuildHasherDefault<XxHash64>> = Default::default();
    let mut aliases: HashMap<usize, SmallVec<[usize; 1]>> = HashMap::with_capacity(input.len());
    for (id, sprite) in input.iter().enumerate() {
        let alias_id = hashes.entry(sprite.bytes.as_slice()).or_insert(id);
        aliases.entry(*alias_id).or_default().push(id);
    }

    let sprites = input
        .into_iter()
        .enumerate()
        .map(|(id, sprite)| Sprite::from_input(id, sprite))
        .collect::<Vec<Sprite>>();
    let sprite_data = sprites
        .iter()
        .enumerate()
        .filter(|(id, _)| aliases.contains_key(id))
        .map(|(_, it)| it.data)
        .collect::<Vec<SpriteData>>();

    let packer_result = P::pack(&sprite_data, options);

    packer_result
        .into_iter()
        .map(|mut sheet| {
            let mut buffer = create_pixel_buffer(sheet.dimensions, stride);
            let mut aliased_anchors = Vec::<SpriteAnchor>::new();
            for anchor in &sheet.anchors {
                write_sprite(
                    &mut buffer,
                    sheet.dimensions,
                    stride,
                    &sprites[anchor.id],
                    &anchor,
                );
                aliased_anchors.extend(
                    aliases[&anchor.id]
                        .iter()
                        .skip(1)
                        .map(|id| SpriteAnchor { id: *id, ..*anchor }),
                );
            }
            sheet.anchors.extend(aliased_anchors);

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

pub fn trim(input: &[InputSprite], stride: usize, alpha_channel_index: usize) -> Vec<InputSprite> {
    input
        .iter()
        .map(|sprite| sprite.trimmed(stride, alpha_channel_index))
        .collect()
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    #[test]
    fn alias_test() {
        let bytes1 = vec![0, 0, 0, 0];
        let bytes2 = vec![1, 1, 1, 1];
        let dimensions = (1, 1);
        let sprite1 = InputSprite {
            bytes: bytes1,
            dimensions,
        };
        let sprite2 = InputSprite {
            bytes: bytes2,
            dimensions,
        };

        let input = vec![sprite1.clone(), sprite1, sprite2];
        let sheets = pack::<SimplePacker>(input, 4, ());

        assert_eq!(sheets[0].anchors.len(), 3);
        assert_eq!(sheets[0].bytes.len(), 8);
    }

    #[test]
    fn alias_with_trimming_test() {
        let bytes1 = vec![1, 1, 1, 1];
        let bytes2 = vec![1, 1, 1, 1, 1, 1, 1, 0];
        let sprite1 = InputSprite {
            bytes: bytes1,
            dimensions: (1, 1),
        };
        let sprite2 = InputSprite {
            bytes: bytes2,
            dimensions: (2, 1),
        };

        let input = vec![sprite2.clone(), sprite1.clone(), sprite1, sprite2];
        let input = trim(input.as_slice(), 4, 3);
        let sheets = pack::<SimplePacker>(input, 4, ());

        assert_eq!(sheets[0].anchors.len(), 4);
        assert_eq!(sheets[0].bytes.len(), 4);
    }
}
