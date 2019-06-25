use {Packer, PackerResult, SpriteAnchor, SpriteData};

pub struct MaxrectsPacker;

pub struct MaxrectsOptions {
    preferred_width: u32,
    preferred_height: u32,
}

impl Packer for MaxrectsPacker {
    type Options = MaxrectsOptions;

    fn pack(sprites: &[SpriteData], options: MaxrectsOptions) -> Vec<PackerResult> {
        let mut bins = vec![MaxRectsBin::new(
            options.preferred_width,
            options.preferred_height,
        )];

        let mut oversized = Vec::new();

        for (i, sprite) in sprites.iter().enumerate() {
            if sprite.dimensions.0 > options.preferred_width
                || sprite.dimensions.1 > options.preferred_height
            {
                oversized.push(MaxRectsBin::oversized(sprite.dimensions, i));
                continue;
            }

            for bin in bins.iter() {}
        }

        vec![PackerResult {
            dimensions: (0, 0),
            anchors: Vec::new(),
        }]
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains(&self, other: &Rect) -> bool {
        self.x >= other.x
            && self.y >= other.y
            && self.x + self.width <= other.x + other.width
            && self.y + self.height <= other.y + other.height
    }

    pub fn no_intersection(&self, other: &Rect) -> bool {
        self.x >= other.x + other.width
            || self.x + self.width <= other.x
            || self.y >= other.y + other.height
            || self.y + self.height <= other.y
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ScoreResult {
    NoFit,
    FitFound(RectScore),
}

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
            free: vec![Rect::new(0, 0, width, height)],
            oversized: false,
        }
    }

    pub fn oversized(dimensions: (u32, u32), index: usize) -> Self {
        let used_rect = Rect::new(0, 0, dimensions.0, dimensions.1);

        MaxRectsBin {
            bin_width: dimensions.0,
            bin_height: dimensions.1,
            used: vec![(used_rect, index)],
            free: vec![],
            oversized: true,
        }
    }

    pub fn insert_sprites(&mut self, sprites: &[SpriteData]) {
        let mut sprites = sprites.iter().cloned().collect::<Vec<SpriteData>>();

        while !sprites.is_empty() {
            // Score all rects and sort them by their score, best score first
            let sorted = sprites
                .iter()
                .filter_map(|sprite| {
                    match self.score_rect(sprite.dimensions.0, sprite.dimensions.1) {
                        ScoreResult::NoFit => None,
                        ScoreResult::FitFound(score) => Some((score, *sprite)),
                    }
                })
                .collect::<Vec<(RectScore, SpriteData)>>();

            let (score, sprite) = {
                // Find out if there's multiple with the best score
                let best_scored = sorted
                    .iter()
                    .filter(|(score, _)| score.primary == sorted[0].0.primary)
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

            self.place_rect(score.placement, *sprite, sprite.id);
            sprites.retain(|s| s.id != sprite.id);
        }
    }

    fn score_rect(&self, width: u32, height: u32) -> ScoreResult {
        use std::cmp::{max, min};

        // We score by best short side fit, since it's the best performing
        // strategy according to the reference implementation
        let mut best_short = std::u32::MAX;
        let mut best_long = std::u32::MAX;
        let mut placement = Rect::new(0, 0, 0, 0);

        self.free.iter().for_each(|it| {
            let leftover_horiz = (it.width as i32 - width as i32).abs() as u32;
            let leftover_vert = (it.height as i32 - height as i32).abs() as u32;

            let short_side_fit = min(leftover_horiz, leftover_vert);
            let long_side_fit = max(leftover_horiz, leftover_vert);

            if short_side_fit < best_short
                || (short_side_fit == best_short && long_side_fit < best_long)
            {
                best_short = short_side_fit;
                best_long = long_side_fit;
                placement = Rect::new(it.x, it.y, width, height);
            }
        });

        // TODO(happenslol): This is kind of a primitive way to check,
        // since it's directly translated from the reference implementation.
        // This function can probably be improved a lot, style-wise
        if placement.height == 0 {
            ScoreResult::NoFit
        } else {
            ScoreResult::FitFound(RectScore {
                placement,
                primary: best_short as i32,
                secondary: best_long as i32,
            })
        }
    }

    fn place_rect(&mut self, rect: Rect, sprite: SpriteData, sprite_id: usize) {
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

        self.prune_free_list();
        self.used.push((rect, sprite_id));
    }

    fn split_rect(&mut self, to_split: Rect, to_place: Rect) {
        if to_place.x < to_split.x + to_split.width && to_place.x + to_place.width > to_split.x {
            // New node at the top side of the placed node.
            if to_place.y > to_split.y && to_place.y < to_split.y + to_split.height {
                self.free.push(Rect {
                    y: to_place.y - to_split.y,
                    ..to_split
                })
            }

            if to_place.y + to_place.height < to_split.y + to_split.height {
                self.free.push(Rect {
                    y: to_place.y + to_place.height,
                    height: to_split.y + to_split.height - (to_place.y + to_place.height),
                    ..to_split
                });
            }
        }

        if to_place.y < to_split.y + to_split.height && to_place.y + to_place.height > to_split.y {
            // New node at the left side of the placed node.
            if to_place.x > to_split.x && to_place.x < to_split.x + to_split.width {
                self.free.push(Rect {
                    width: to_place.x - to_split.x,
                    ..to_split
                });
            }

            // New node at the right side of the placed node.
            if to_place.x + to_place.width < to_split.x + to_split.width {
                self.free.push(Rect {
                    x: to_place.x + to_place.width,
                    width: to_split.x + to_split.width - (to_place.x + to_place.width),
                    ..to_split
                });
            }
        }
    }

    fn prune_free_list(&mut self) {
        // TODO(happenslol): This is really ugly, since I haven't found
        // a way to either modify the array while iterating over it, or
        // modify the iterator variable while going over a range.
        // I'm 99% sure there's a better way to do this.
        let mut i = 0;
        'outer: while i < self.free.len() {
            let mut j = i + 1;

            'inner: while j < self.free.len() {
                if self.free[j].contains(&self.free[i]) {
                    self.free.remove(i);
                    i -= 1;
                    break 'inner;
                }

                if self.free[i].contains(&self.free[j]) {
                    self.free.remove(j);
                } else {
                    j += 1;
                }
            }

            i += 1;
        }
    }
}
