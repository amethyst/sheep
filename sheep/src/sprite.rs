#[derive(Debug, Clone)]
pub struct Sprite {
    data: SpriteData,
}

#[derive(Debug, Clone, Copy)]
pub struct SpriteData {
    pub id: usize,
    pub dimensions: (u32, u32),
}

#[derive(Debug, Clone, Copy)]
pub struct SpriteAnchor {
    pub id: usize,
    pub position: (u32, u32),
}
