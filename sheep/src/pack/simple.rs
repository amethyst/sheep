use std::cmp::{min, Ordering};
use {Packer, PackerResult, SpriteAnchor, SpriteData};

pub struct SimplePacker;

impl Packer for SimplePacker {
    type Options = ();

    fn pack(sprites: &[SpriteData], _options: ()) -> Vec<PackerResult> {
        let mut sprites = sprites.iter().cloned().collect::<Vec<SpriteData>>();

        let mut free = Vec::new();
        let mut absolute = Vec::new();

        sprites.sort_by(compare_area);
        sprites.reverse();
        free.push((0, 0));

        for sprite in sprites.iter() {
            // Push the sprite to the next free anchor
            let next_free = *free.first().expect("No free anchor");
            absolute.push(SpriteAnchor::new(sprite.id, next_free, sprite.dimensions));

            // find new anchors
            let mut new_right = (next_free.0 + sprite.dimensions.0, next_free.1);
            let mut new_bottom = (next_free.0, next_free.1 + sprite.dimensions.1);

            // still finding new anchors
            for i in 1..(free.len() - 1) {
                // If we removed an anchor after the first round,
                // we might be out of bounds at this point
                if i > 1 && i >= free.len() {
                    break;
                }

                if free[i].0 >= free[0].0 && free[i].0 <= new_right.0 {
                    new_right.1 = min(new_right.1, free[i].1);
                    free.remove(i);
                    continue;
                }

                if free[i].1 >= free[0].1 && free[i].1 <= new_bottom.1 {
                    new_bottom.0 = min(new_bottom.0, free[i].0);
                    free.remove(i);
                    continue;
                }
            }

            // remove first, push new anchors
            free.remove(0);

            if !free.contains(&new_right) {
                free.push(new_right);
            }

            if !free.contains(&new_bottom) {
                free.push(new_bottom);
            }

            free.sort_by(compare_pos);
        }

        let width = free
            .iter()
            .max_by(|a, b| a.0.cmp(&b.0))
            .expect("Invalid: No free anchors")
            .0;

        let height = free
            .iter()
            .max_by(|a, b| a.1.cmp(&b.1))
            .expect("Invalid: No free anchors")
            .1;

        // Finally sort the anchors so that they are in the same order as the
        // input sprites
        absolute.sort_by_key(|s| s.id);

        let result = PackerResult {
            dimensions: (width, height),
            anchors: absolute,
        };

        vec![result]
    }
}

fn compare_area(a: &SpriteData, b: &SpriteData) -> Ordering {
    (a.dimensions.0 * a.dimensions.1).cmp(&(b.dimensions.0 * b.dimensions.1))
}

fn compare_pos(a: &(u32, u32), b: &(u32, u32)) -> Ordering {
    // NOTE(happenslol): We might overflow here quickly if the output
    // sprite becomes too big, so we use u64s. This is why this algorithm
    // doesn't scale very well...
    let a = (a.0 as u64, a.1 as u64);
    let b = (b.0 as u64, b.1 as u64);

    (a.0.pow(4) + a.1.pow(4)).cmp(&(b.0.pow(4) + b.1.pow(4)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_square() {
        let sprites = (0..16)
            .map(|i| SpriteData::new(i, (20, 20)))
            .collect::<Vec<SpriteData>>();

        let result = SimplePacker::pack(&sprites, ());

        assert_eq!(result[0].dimensions.0, 20 * 4);
        assert_eq!(result[0].dimensions.1, 20 * 4);
    }
}
