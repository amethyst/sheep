use super::Format;
use SpriteAnchor;

pub struct AmethystNamedFormat;

/// `NameSprite` represents a field in a `SerializedNamedSpriteSheet`.
/// All of the fields, except `name`, mimic the `SpritePosition` struct.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct NamedSpritePosition {
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub offsets: Option<[f32; 2]>,
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
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct SerializedNamedSpriteSheet {
    pub texture_width: f32,
    pub texture_height: f32,
    pub sprites: Vec<NamedSpritePosition>,
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
            .map(|anchor| (anchor, options[anchor.id].clone()))
            .map(Into::into)
            .collect::<Vec<NamedSpritePosition>>();

        SerializedNamedSpriteSheet {
            texture_width: dimensions.0 as f32,
            texture_height: dimensions.1 as f32,
            sprites: sprite_positions,
        }
    }
}
