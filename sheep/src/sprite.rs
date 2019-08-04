#[derive(Debug, Clone)]
pub struct InputSprite {
    pub bytes: Vec<u8>,
    pub dimensions: (u32, u32),
}

#[derive(Debug, Clone)]
pub struct Sprite {
    pub bytes: Vec<u8>,
    pub data: SpriteData,
}

impl Sprite {
    pub fn from_input(index: usize, input: InputSprite) -> Sprite {
        Sprite {
            bytes: input.bytes,
            data: SpriteData {
                id: index,
                dimensions: input.dimensions,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SpriteData {
    pub id: usize,
    pub dimensions: (u32, u32),
}

impl SpriteData {
    pub fn new(id: usize, dimensions: (u32, u32)) -> Self {
        SpriteData { id, dimensions }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SpriteAnchor {
    pub id: usize,
    pub position: (u32, u32),
    pub dimensions: (u32, u32),
}

impl SpriteAnchor {
    pub fn new(id: usize, position: (u32, u32), dimensions: (u32, u32)) -> Self {
        SpriteAnchor {
            id,
            position,
            dimensions,
        }
    }
}

pub fn create_pixel_buffer(dimensions: (u32, u32), stride: usize) -> Vec<u8> {
    let length = (dimensions.0 as usize) * (dimensions.1 as usize) * stride;
    (0..length).map(|_| 0).collect::<Vec<u8>>()
}

pub fn write_sprite(
    buffer: &mut Vec<u8>,
    dimensions: (u32, u32),
    stride: usize,
    sprite: &Sprite,
    anchor: &SpriteAnchor,
) {
    let stride = stride as u32;

    for y in 0..sprite.data.dimensions.1 {
        let sprite_y = y * sprite.data.dimensions.0 * stride;
        let buffer_y = (y + anchor.position.1) * dimensions.0 * stride;

        for x in 0..sprite.data.dimensions.0 {
            let sprite_x = x * stride;
            let buffer_x = (x + anchor.position.0) * stride;

            for i in 0..stride {
                let sprite_idx = (sprite_y + sprite_x + i) as usize;
                let buffer_idx = (buffer_y + buffer_x + i) as usize;

                buffer[buffer_idx] = sprite.bytes[sprite_idx];
            }
        }
    }
}
