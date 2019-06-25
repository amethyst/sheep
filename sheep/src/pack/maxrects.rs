use {Packer, PackerResult, SpriteAnchor, SpriteData};

pub struct MaxrectsPacker;

pub struct MaxrectsOptions;

impl Packer for MaxrectsPacker {
    type Options = MaxrectsOptions;

    fn pack(sprites: &[SpriteData], options: MaxrectsOptions) -> Vec<PackerResult> {
        vec![PackerResult {
            dimensions: (0, 0),
            anchors: Vec::new(),
        }]
    }
}

static DEFAULT_BIN_SIZE: u32 = 4096;
static ALLOW_FLIP: bool = false;

#[derive(Debug, Clone, Copy)]
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
}

#[derive(Debug, Clone, Copy)]
enum Strategy {
    BestShortSideFit,
    BestLongSideFit,
    BestAreaFit,
    BottomLeftRule,
    ContactPointRule,
}

#[derive(Debug, Clone, Copy)]
struct RectScore {
    placement: Rect,
    primary: i32,
    secondary: i32,
}

#[derive(Debug, Clone)]
struct MaxRectsBins {
    bin_width: u32,
    bin_height: u32,
    allow_flip: bool,
    used: Vec<(Rect, usize)>,
    free: Vec<Rect>,
}

impl MaxRectsBins {
    pub fn new() -> Self {
        MaxRectsBins {
            bin_width: DEFAULT_BIN_SIZE,
            bin_height: DEFAULT_BIN_SIZE,
            allow_flip: ALLOW_FLIP,
            used: Vec::new(),
            free: vec![Rect::new(0, 0, DEFAULT_BIN_SIZE, DEFAULT_BIN_SIZE)],
        }
    }

    pub fn insert_sprites(&mut self, sprites: &[SpriteData]) {
        let mut sprites = sprites.iter().cloned().collect::<Vec<SpriteData>>();

        while !sprites.is_empty() {
            // Score all rects and sort them by their score, best score first
            let sorted = sprites
                .iter()
                .map(|it| (self.score_rect(it.dimensions.0, it.dimensions.1), *it))
                .collect::<Vec<(RectScore, SpriteData)>>();

            let next = {
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

            self.place_rect(&next.0.placement, &next.1);
            sprites.retain(|it| it.id != next.1.id);
        }
    }

    fn score_rect(&self, width: u32, height: u32) -> RectScore {
        use std::cmp::{min, max};

        // For now, we always use bssf since it is the most efficient by itself
        // TODO: Other strategies + global best choice
        let mut best_short = std::u32::MAX;
        let mut best_long = std::u32::MAX;
        let mut placement = Rect::new(0, 0, 0, 0);

        self.free.iter().for_each(|it| {
            let leftover_horiz = (it.width as i32 - width as i32).abs() as u32;
            let leftover_vert = (it.height as i32 - height as i32).abs() as u32;

            let short_side_fit = min(leftover_horiz, leftover_vert);
            let long_side_fit = max(leftover_horiz, leftover_vert);

            if short_side_fit < best_short ||
                (short_side_fit == best_short && long_side_fit < best_long) {
                best_short = short_side_fit;
                best_long = long_side_fit;
                placement = Rect::new(it.x, it.y, width, height);
            }
        });

        RectScore {
            placement,
            primary: best_short as i32,
            secondary: best_long as i32,
        }
    }

    fn find_position() {}

    fn place_rect(&mut self, rect: &Rect, sprite: &SpriteData) {

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
