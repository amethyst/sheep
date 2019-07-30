#[derive(Clone)]
pub struct InputSprite {
    pub bytes: Vec<u8>,
    pub dimensions: (u32, u32),
}

impl InputSprite {
    pub fn trimmed(&self, stride: usize, alpha_channel_index: usize) -> InputSprite {
        let stride = stride as u32;
        let alpha_channel_index = alpha_channel_index as u32;

        let mut top_ident = self.dimensions.1;
        let mut left_ident = self.dimensions.0;

        // not including right and bottom
        let mut right_ident = 0;
        let mut bottom_ident = 0;

        'outer_top: for y in 0..self.dimensions.1 {
            let row_offset = y * self.dimensions.0 * stride;

            for x in 0..self.dimensions.0 {
                let index = row_offset + x * stride + alpha_channel_index;
                if self.bytes[index as usize] != 0 {
                    left_ident = left_ident.min(x);
                    right_ident = right_ident.max(x + 1);
                    top_ident = y;
                    break 'outer_top;
                }
            }
        }

        // if whole image is transparent, this loop won't run
        'outer_bottom: for y in (top_ident..self.dimensions.1).rev() {
            let row_offset = y * self.dimensions.0 * stride;

            // reverse for cache friendliness
            for x in (0..self.dimensions.0).rev() {
                let index = row_offset + x * stride + alpha_channel_index;
                if self.bytes[index as usize] != 0 {
                    left_ident = left_ident.min(x);
                    right_ident = right_ident.max(x + 1);
                    bottom_ident = y + 1;
                    break 'outer_bottom;
                }
            }
        }

        // image is completly transparent
        if top_ident == self.dimensions.1 {
            // NOTE: not sure, maybe return non empty image
            // in order not to break something unexpectedly?
            return InputSprite {
                bytes: vec![],
                dimensions: (0, 0),
            };
        }

        // left ident can only decrease and right ident can only increase,
        // so we only look at pixels that can change them
        for y in top_ident..bottom_ident {
            let row_offset = y * self.dimensions.0 * stride;
            for x in 0..left_ident {
                let index = row_offset + x * stride + alpha_channel_index;
                if self.bytes[index as usize] != 0 {
                    left_ident = x;
                    break;
                }
            }

            for x in (right_ident..self.dimensions.0).rev() {
                let index = row_offset + x * stride + alpha_channel_index;
                if self.bytes[index as usize] != 0 {
                    right_ident = x + 1;
                    break;
                }
            }
        }

        let trimmed_dimensions = (right_ident - left_ident, bottom_ident - top_ident);
        let mut trimmed_buffer = create_pixel_buffer(trimmed_dimensions, stride as usize);
        for y in 0..trimmed_dimensions.1 {
            let sprite_y = (y + top_ident) * self.dimensions.0 * stride;
            let trimmed_y = y * trimmed_dimensions.0 * stride;

            for x in 0..trimmed_dimensions.0 {
                let sprite_x = (x + left_ident) * stride;
                let trimmed_x = x * stride;

                for i in 0..stride {
                    let sprite_idx = (sprite_y + sprite_x + i) as usize;
                    let trimmed_idx = (trimmed_y + trimmed_x + i) as usize;

                    trimmed_buffer[trimmed_idx] = self.bytes[sprite_idx];
                }
            }
        }

        InputSprite {
            bytes: trimmed_buffer,
            dimensions: trimmed_dimensions,
        }
    }
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
