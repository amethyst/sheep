# Sheep

`sheep` (Sprite**sheet** **packer**) is a lightweight and modular library used to create spritesheets. It aims to impose as little restrictions as possible on the usage of its API so that it can be used in asset pipelines.

The project is in heavy development and the API might change a few times until we reach a stable version, but the general flow of inputting bytes and receiving metadata and a byte vec will remain the same.

## Usage

To use the CLI, simple use `cargo run -- ...`. For an example on how to use the library, please see the `simple_pack` example in the `sheep/examples` directory.

## Implementing your own `Packer` and `Format`

Sheep achieves it's modularity by letting you choose the implementation it will use for packing the sprites and encoding the metadata. Right now, only a very naive packing algorithm is provided (`SimplePacker`), as well as the data format used by the [amethyst engine](https://github.com/amethyst/amethyst) (`AmethystFormat`). There will be more in the future, but for now, you can choose your own packing algorithm and format:

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

impl Format for AmethystFOrmat {
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

## Roadmap

Here are the planned features for `sheep`:

* Support for multiple output textures (bins)
* Smart output texture sizing
* More packing algorithms
    * MAXRECTS
    * Skyline
* More meta formats

## License

`sheep` is dual licensed under MIT and Apache, see `COPYING`.
