use super::Format;
use SpriteAnchor;

pub struct AmethystNamedFormat;

/// `NameSprite` represents a field in a `SerializedNamedSpriteSheet`.
/// All of the fields, except `name`, mimic the `SpritePosition` struct.
#[derive(Clone, Debug, PartialEq, Serialize)]
struct NamedSpritePosition {
    name: String,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    offsets: Option<[f32; 2]>,
}

impl From<(&SpriteAnchor, String)> for NamedSpritePosition {
    fn from(anchor: (&SpriteAnchor, String)) -> NamedSpritePosition {
        return NamedSpritePosition {
            name: anchor.1,
            x: anchor.0.position.0 as f32,
            y: anchor.0.position.1 as f32,
            width: anchor.0.dimensions.0 as f32,
            height: anchor.0.dimensions.1 as f32,
            offsets: None,
        };
    }
}

/// `SerializedNamedSpriteSheet` is the serializable representation of the sprite sheet.
/// It is similar to `SerializedSpriteSheet`, except that it has a `Vec` of `NamedSpritePosition`s
/// instead of `SpriteAnchor`s
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SerializedNamedSpriteSheet {
    spritesheet_width: f32,
    spritesheet_height: f32,
    sprites: Vec<NamedSpritePosition>,
}

impl Format for AmethystNamedFormat {
    type Data = SerializedNamedSpriteSheet;
    type Options = Vec<String>;

    fn encode(
        dimensions: (u32, u32),
        sprites: &[SpriteAnchor],
        options: Self::Options,
    ) -> Self::Data {
        let sprite_positions = sprites
            .iter()
            .zip(options.into_iter())
            .map(Into::into)
            .collect::<Vec<NamedSpritePosition>>();

        SerializedNamedSpriteSheet {
            spritesheet_width: dimensions.0 as f32,
            spritesheet_height: dimensions.1 as f32,
            sprites: sprite_positions,
        }
    }
}
