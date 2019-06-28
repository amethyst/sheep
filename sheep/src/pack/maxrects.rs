use {Packer, PackerResult, SpriteAnchor, SpriteData};

pub struct MaxrectsPacker;

#[derive(Copy, Clone)]
pub struct MaxrectsOptions {
    preferred_width: u32,
    preferred_height: u32,
}

impl Default for MaxrectsOptions {
    fn default() -> Self {
        MaxrectsOptions {
            preferred_width: 4096,
            preferred_height: 4096,
        }
    }
}

impl MaxrectsOptions {
    pub fn preferred_width(mut self, width: u32) -> Self {
        self.preferred_width = width;
        self
    }

    pub fn preferred_height(mut self, height: u32) -> Self {
        self.preferred_height = height;
        self
    }
}

impl Packer for MaxrectsPacker {
    type Options = MaxrectsOptions;

    fn pack(sprites: &[SpriteData], options: MaxrectsOptions) -> Vec<PackerResult> {
        let mut bins = Vec::new();
        let mut oversized = Vec::new();

        // First, filter out all oversized sprites
        let mut sprites = sprites
            .iter()
            .enumerate()
            .filter(|(i, sprite)| {
                if sprite.dimensions.0 > options.preferred_width
                    || sprite.dimensions.1 > options.preferred_height
                {
                    oversized.push(MaxRectsBin::oversized(sprite.dimensions, *i));
                    false
                } else {
                    true
                }
            })
            .map(|(_, sprite)| *sprite)
            .collect::<Vec<_>>();

        // Now, keep inserting as many as possible into each bin until
        // all sprites have been placed. Since all oversized rects have
        // already been filtered out, this will always terminate.
        while !sprites.is_empty() {
            let mut bin = MaxRectsBin::new(options.preferred_width, options.preferred_height);
            sprites = bin.insert_sprites(&sprites);
            bins.push(bin);
        }

        bins.extend(oversized.into_iter());
        let result = bins
            .into_iter()
            .map(|bin| bin.to_result())
            .collect::<Vec<PackerResult>>();

        result
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Rect {
    pub min_x: u32,
    pub min_y: u32,
    pub max_x: u32,
    pub max_y: u32,
}

impl Rect {
    pub fn xywh(x: u32, y: u32, width: u32, height: u32) -> Self {
        Rect {
            min_x: x,
            min_y: y,
            max_x: x + width,
            max_y: y + height,
        }
    }

    pub fn new(min_x: u32, min_y: u32, max_x: u32, max_y: u32) -> Self {
        Rect {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    pub fn contains(&self, other: &Rect) -> bool {
        self.min_x <= other.min_x
            && self.min_y <= other.min_y
            && self.max_x >= other.max_x
            && self.max_y >= other.max_y
    }

    pub fn no_intersection(&self, other: &Rect) -> bool {
        self.min_x >= other.max_x
            || self.max_x <= other.min_x
            || self.min_y >= other.max_y
            || self.max_y <= other.min_y
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ScoreResult {
    NoFit,
    FitFound(RectScore),
}

// NOTE(happenslol): The score represents the leftover
// space in case of a placement, thus _lower is better_
#[derive(Debug, Clone, Copy, PartialEq)]
struct RectScore {
    placement: Rect,
    primary: i32,
    secondary: i32,
}

#[derive(Debug, Clone)]
struct MaxRectsBin {
    bin_width: u32,
    bin_height: u32,
    used: Vec<(Rect, usize)>,
    free: Vec<Rect>,
    oversized: bool,
}

impl MaxRectsBin {
    pub fn new(width: u32, height: u32) -> Self {
        MaxRectsBin {
            bin_width: width,
            bin_height: height,
            used: Vec::new(),
            free: vec![Rect::xywh(0, 0, width, height)],
            oversized: false,
        }
    }

    pub fn oversized(dimensions: (u32, u32), index: usize) -> Self {
        let used_rect = Rect::xywh(0, 0, dimensions.0, dimensions.1);

        MaxRectsBin {
            bin_width: dimensions.0,
            bin_height: dimensions.1,
            used: vec![(used_rect, index)],
            free: vec![],
            oversized: true,
        }
    }

    pub fn to_result(&self) -> PackerResult {
        let anchors = self
            .used
            .iter()
            .map(|(rect, id)| SpriteAnchor {
                id: *id,
                position: (rect.min_x, rect.min_y),
                dimensions: (rect.max_x - rect.min_x, rect.max_y - rect.min_y),
            })
            .collect::<Vec<SpriteAnchor>>();

        let w = anchors
            .iter()
            .map(|a| a.position.0 + a.dimensions.0)
            .max()
            .unwrap_or(0);

        let h = anchors
            .iter()
            .map(|a| a.position.1 + a.dimensions.1)
            .max()
            .unwrap_or(0);

        PackerResult {
            dimensions: (w, h),
            anchors,
        }
    }

    pub fn insert_sprites(&mut self, sprites: &[SpriteData]) -> Vec<SpriteData> {
        let mut sprites = sprites.iter().cloned().collect::<Vec<SpriteData>>();
        let mut placed = Vec::new();

        while !sprites.is_empty() {
            // Score all rects and sort them by their score, best score first
            let mut placeable = sprites
                .iter()
                .filter_map(|sprite| {
                    match self.score_rect(sprite.dimensions.0, sprite.dimensions.1) {
                        ScoreResult::NoFit => None,
                        ScoreResult::FitFound(score) => Some((score, *sprite)),
                    }
                })
                .collect::<Vec<(RectScore, SpriteData)>>();

            // If the placeable list is empty at this point, we can break out and
            // return all SpriteDatas we were not able to place
            if placeable.is_empty() {
                break;
            }

            placeable.sort_by_key(|(score, _)| score.primary);
            let (score, sprite) = {
                // Find out if there's multiple with the best score
                let best_scored = placeable
                    .iter()
                    .filter(|(score, _)| score.primary == placeable[0].0.primary)
                    .collect::<Vec<&(RectScore, SpriteData)>>();

                // If not, we have found the next best fit! Othweise, take the
                // next best by the secondary score
                if best_scored.len() == 1 {
                    best_scored[0]
                } else {
                    best_scored
                        .iter()
                        .min_by_key(|(score, _)| score.secondary)
                        .expect("Unreachable")
                }
            };

            self.place_rect(score.placement, sprite.id);
            sprites.retain(|s| s.id != sprite.id);
            placed.push(sprite.id);
        }

        sprites
    }

    pub fn score_rect(&self, width: u32, height: u32) -> ScoreResult {
        use std::cmp::{max, min};

        // We score by best short side fit, since it's the best performing
        // strategy according to the reference implementation
        let mut best_short = std::u32::MAX;
        let mut best_long = std::u32::MAX;
        let mut placement = Rect::new(0, 0, 0, 0);
        let mut fit_found = false;

        for rect in &self.free {
            let other_width = (rect.max_x - rect.min_x) as i32;
            let other_height = (rect.max_y - rect.min_y) as i32;

            let leftover_horiz = (other_width - width as i32).abs() as u32;
            let leftover_vert = (other_height - height as i32).abs() as u32;

            let short_side_fit = min(leftover_horiz, leftover_vert);
            let long_side_fit = max(leftover_horiz, leftover_vert);

            if short_side_fit < best_short
                || (short_side_fit == best_short && long_side_fit < best_long)
            {
                best_short = short_side_fit;
                best_long = long_side_fit;
                placement = Rect::xywh(rect.min_x, rect.min_y, width, height);
                fit_found = true;
            }
        }

        if !fit_found {
            ScoreResult::NoFit
        } else {
            ScoreResult::FitFound(RectScore {
                placement,
                primary: best_short as i32,
                secondary: best_long as i32,
            })
        }
    }

    fn place_rect(&mut self, rect: Rect, sprite_id: usize) {
        let mut to_process = self.free.len();
        let mut i = 0;

        while i < to_process {
            if self.free[i].no_intersection(&rect) {
                i += 1;
                continue;
            }

            let to_split = self.free.remove(i);
            self.split_rect(to_split, rect);
            to_process -= 1;
        }

        remove_redundant_rects(&mut self.free);
        self.used.push((rect, sprite_id));
    }

    fn split_rect(&mut self, split: Rect, place: Rect) {
        if place.min_x < split.max_x && place.max_x > split.min_x {
            // New node at the top side of the placed node.
            if place.min_y > split.min_y && place.min_y < split.max_y {
                let height = split.max_y - split.min_y;
                let new_min_y = place.min_y - split.min_y;

                self.free.push(Rect {
                    min_y: new_min_y,
                    max_y: new_min_y + height,
                    ..split
                })
            }

            if place.max_y < split.max_y {
                let new_min_y = place.max_y;
                let height = split.max_y - place.max_y;

                self.free.push(Rect {
                    min_y: new_min_y,
                    max_y: new_min_y + height,
                    ..split
                });
            }
        }

        if place.min_y < split.max_y && place.max_y > split.min_y {
            // New node at the left side of the placed node.
            if place.min_x > split.min_x && place.min_x < split.max_x {
                let width = place.min_x - split.min_x;

                self.free.push(Rect {
                    max_x: split.min_x + width,
                    ..split
                });
            }

            // New node at the right side of the placed node.
            if place.max_x < split.max_x {
                let new_min_x = place.max_x;
                let width = split.max_x - place.max_x;

                self.free.push(Rect {
                    min_x: new_min_x,
                    max_x: new_min_x + width,
                    ..split
                });
            }
        }
    }
}

fn remove_redundant_rects(rects: &mut Vec<Rect>) {
    let mut i = 0;
    while let Some(next) = rects.get(i).cloned() {
        // check if it's contained by any other rect
        if rects[i + 1..].iter().any(|s| s.contains(&next)) {
            // if so, discard it and keep going
            rects.swap_remove(i);
            continue;
        }

        // otherwise, prune all unprocessed rects that are
        // contained by our rect and accept it
        for j in ((i + 1)..rects.len()).rev() {
            if next.contains(&rects[j]) {
                rects.swap_remove(j);
            }
        }

        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::{self, Bencher};

    #[test]
    fn remove_redundant() {
        let mut rects = Vec::new();
        for i in 0..10 {
            rects.push(Rect::xywh(i * 10, 0, 10, 10));
            rects.push(Rect::xywh(i * 10 + 2, 2, 6, 6));
        }

        assert_eq!(rects.len(), 20);
        remove_redundant_rects(&mut rects);
        assert_eq!(rects.len(), 10);

        for rect in &rects {
            assert_eq!((rect.max_x - rect.min_x), 10);
            assert_eq!((rect.max_y - rect.min_y), 10);
        }
    }

    #[test]
    fn pack_regular() {
        let mut sprites = (0..10)
            .map(|i| SpriteData::new(i, (10, 10)))
            .collect::<Vec<SpriteData>>();

        let options = MaxrectsOptions::default()
            .preferred_width(10 * 10)
            .preferred_height(10 * 10);

        let result = MaxrectsPacker::pack(&sprites, options);
        let first = result.iter().next().expect("should have 1 result");

        assert_eq!(result.len(), 1);

        // They'll all be packed into 1 column in this example, so they output
        // will be shrunk to fit the entire width plus 1 column.
        assert_eq!(first.dimensions.0, 10);
        assert_eq!(first.dimensions.1, 10 * 10);

        // The new sprite will bu pushed to the next line
        sprites.push(SpriteData::new(11, (10, 20)));
        let result = MaxrectsPacker::pack(&sprites, options);
        let first = result.iter().next().expect("should have 1 result");

        assert_eq!(first.dimensions.0, 30);
        assert_eq!(first.dimensions.1, 100);
    }

    #[test]
    fn pack_oversized() {
        let oversized = (0..1000)
            .map(|i| SpriteData::new(i, (100, 100)))
            .collect::<Vec<SpriteData>>();

        let options = MaxrectsOptions::default()
            .preferred_width(50)
            .preferred_height(50);

        let result = MaxrectsPacker::pack(&oversized, options);

        assert_eq!(result.len(), oversized.len());
        for bin in result {
            assert_eq!(bin.dimensions.0, 100);
            assert_eq!(bin.dimensions.1, 100);
        }
    }

    #[bench]
    fn bench_pack(b: &mut Bencher) {
        let mut sprites = (0..1000)
            .map(|i| SpriteData::new(i, (100, 100)))
            .collect::<Vec<SpriteData>>();

        let smaller_sprites = (0..1000)
            .map(|i| SpriteData::new(i, (80, 80)))
            .collect::<Vec<SpriteData>>();

        sprites.extend(smaller_sprites.into_iter());

        let options = MaxrectsOptions::default()
            .preferred_width(1000 * 100)
            .preferred_height(1000 * 100);

        b.iter(|| {
            let result = MaxrectsPacker::pack(&sprites, options);
            let _ = test::black_box(&result);
        });
    }
}
