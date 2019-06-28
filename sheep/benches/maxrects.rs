#![feature(test)]
extern crate test;
extern crate sheep;

use test::Bencher;
use sheep::{Packer, SpriteData, MaxrectsPacker, MaxrectsOptions};

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