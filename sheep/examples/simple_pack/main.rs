extern crate image;
extern crate sheep;

use sheep::{AmethystFormat, InputSprite, SimplePacker};
use std::{fs::File, io::prelude::*};

fn main() {
    // First, we get the raw, uncompressed pixel data of the image. These
    // can come from any sources and even be generated in code, but for
    // simplicity's sake we're using the image crate here.
    let img =
        image::open("sheep/examples/simple_pack/resources/logo.png").expect("Failed to open image");
    let img = img.as_rgba8().expect("Failed to convert image to rgba8");

    let dimensions = img.dimensions();
    let bytes = img
        .pixels()
        .flat_map(|it| it.data.iter().map(|it| *it))
        .collect::<Vec<u8>>();

    // We'll just repeat the same sprite 16 times and pack it into a texture.
    let sprites = (0..16)
        .map(|_| InputSprite {
            dimensions,
            bytes: bytes.clone(),
        }).collect::<Vec<InputSprite>>();

    // Do the actual packing! 4 defines the stride, since we're using rgba8 we
    // have 4 bytes per pixel.
    let sprite_sheet = sheep::pack::<SimplePacker>(sprites, 4);

    // Now, we can encode the sprite sheet in a format of our choosing to
    // save things such as offsets, positions of the sprites and so on.
    let meta = sheep::encode::<AmethystFormat>(&sprite_sheet);

    // Next, we save the output to a file using the image crate again.
    let outbuf = image::RgbaImage::from_vec(
        sprite_sheet.dimensions.0,
        sprite_sheet.dimensions.1,
        sprite_sheet.bytes,
    ).expect("Failed to construct image from sprite sheet bytes");

    outbuf.save("out.png").expect("Failed to save image");

    // Lastly, we serialize the meta info using serde. This can be any format
    // you want, just implement the trait and pass it to encode.
    let mut meta_file = File::create("out.ron").expect("Failed to create meta file");
    let meta_str = ron::ser::to_string(&meta).expect("Failed to encode meta file");

    meta_file
        .write_all(meta_str.as_bytes())
        .expect("Failed to write meta file");
}
