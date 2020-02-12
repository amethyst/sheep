# Sheep 🐑

[![Build Status](https://travis-ci.org/amethyst/sheep.svg?branch=master)](https://travis-ci.org/amethyst/sheep)
[![Crates.io](https://img.shields.io/crates/v/sheep)](https://crates.io/crates/sheep)

`sheep` (Sprite**shee**t **p**acker) is a lightweight and modular library used to create spritesheets. It aims to impose as little restrictions as possible on the usage of its API so that it can be used in asset pipelines.

The project is in heavy development and the API might change a few times until we reach a stable version, but the general flow of inputting bytes and receiving metadata and a byte vec will remain the same.

## Usage

To use the CLI, simply install it with cargo:

```
cargo install sheep_cli
```

Usagen hints are provided. To see all options, simply run the command with no arguments. Options can be passed to the packers using the `--options` flag, as space separated `key=value` pairs.
By default, the `maxrects` packer will be used, see [packers](#Packers) for more information.

**Example:**

```
sheep pack --options max_width=1024 max_height=1024 sprites/*.png
```

If you want to use the CLI from source, simple clone the repo and run `cargo run -- ...`. For an example on how to use the library directly, please see the `simple_pack` example in the `sheep/examples` directory.

## Implementing your own `Packer` and `Format`

Sheep achieves its modularity by letting you choose the implementation it will use for packing the sprites and encoding the metadata. Right now, two common packing algorithms are provided (`SimplePacker` and `MaxrectsPacker`, see [packers](#Packers)), as well as the data format used by the [amethyst engine](https://github.com/amethyst/amethyst) (`AmethystFormat`). There will be more in the future, however, you can also choose your own packing algorithm and format:

#### Implementing `Packer`

```rust
pub struct MyPacker;

impl Packer for MyPacker {
    fn pack(sprites: &[SpriteData]) -> PackerResult {
        // Spritedata contains an id for the sprite to reference back
        // to it, and the dimensions of the sprite.

        // The expected output is the dimensions of the resulting spritesheet,
        // as well as all the anchors for the sprites (i.e. their positions).
        PackerResult { dimensions, anchors }
    }
}
```

#### Implementing `Format`

```rust
pub struct MyFormat;

// This is the format that will be output by encode, and you'll probably want
// to serialize later.
#[derive(Serialize)]
pub struct Foo {}

impl Format for AmethystFormat {
    type Data = Foo;

    fn encode(dimensions: (u32, u32), sprites: &[SpriteAnchor]) -> Self::Data {
        // Encode the spritesheet dimensions and sprite positions into
        // your chosen data format here.

        Foo {}
    }
}
```

#### Using your custom `impl`s

To use custom packers or formatters, simply pass them as type parameters when calling the functions:

```rust
let sprite_sheet = sheep::pack::<MyPacker>(sprites, 4);
let meta = sheep::encode::<MyFormat>(&sprite_sheet);
```

## Packers

Right now, there are two implementations to choose from:

- MAXRECTS (**recommended**)

Implementation of the maxrects sprite packing algorithm. The paper and original implementation used as a reference for this can be found [here](https://github.com/juj/RectangleBinPack). This algorithm should yield optimal results in most scenarios.

- simple

A naive implementation that will sort the sprites by area and then pack them all into a single texture. This won't scale very well since you can't limit the maximum size of the resulting sprite sheet, but can be quicker than maxrects in simple scenarios.

## Roadmap

Here are the planned features for `sheep`:

- ~~Support for multiple output textures (bins)~~
- ~~Smart output texture sizing~~
- More packing algorithms
  - ~~MAXRECTS~~
  - Skyline
- More meta formats
- More image formats

## License

`sheep` is dual licensed under MIT and Apache, see `COPYING`.
