#[derive(Clone)]
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
}
