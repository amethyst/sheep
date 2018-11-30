mod format;
mod pack;
mod sprite;

pub use {
    format::Format,
    pack::{simple::SimplePacker, Packer, PackerResult},
    sprite::{InputSprite, Sprite, SpriteAnchor, SpriteData},
};

pub struct SpriteSheet<F: Format> {
    _dimensions: (u32, u32),
    _data: F,
}

pub fn process<P, F>(input: Vec<InputSprite>) -> SpriteSheet<F>
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
    let format_result = F::encode(packer_result.dimensions, &packer_result.anchors);

    SpriteSheet {
        _dimensions: packer_result.dimensions,
        _data: format_result,
    }
}
