use super::Format;
use SpriteAnchor;

pub struct AmethystFormat;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SpritePosition {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub offsets: Option<[f32; 2]>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SerializedSpriteSheet {
    pub texture_width: f32,
    pub texture_height: f32,
    pub sprites: Vec<SpritePosition>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum Wrapper<T> {
    List(T),
}

impl Format for AmethystFormat {
    type Data = Wrapper<SerializedSpriteSheet>;
    type Options = ();

    fn encode(
        dimensions: (u32, u32),
        sprites: &[SpriteAnchor],
        _options: Self::Options,
    ) -> Self::Data {
        let sprite_positions = sprites
            .iter()
            .map(|it| SpritePosition {
                x: it.position.0 as f32,
                y: it.position.1 as f32,
                width: it.dimensions.0 as f32,
                height: it.dimensions.1 as f32,
                offsets: None,
            })
            .collect::<Vec<SpritePosition>>();

        Wrapper::List(SerializedSpriteSheet {
            texture_width: dimensions.0 as f32,
            texture_height: dimensions.1 as f32,
            sprites: sprite_positions,
        })
    }
}
