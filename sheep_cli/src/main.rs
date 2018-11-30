extern crate clap;
extern crate image;
extern crate ron;
extern crate sheep;

use clap::{App, AppSettings, Arg, SubCommand};
use sheep::{AmethystFormat, InputSprite, SimplePacker};
use std::{fs::File, io::prelude::*};

fn main() {
    let app = App::new("sheep")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("pack")
                .help("Packs supplied images into a spritesheet")
                .arg(Arg::with_name("INPUT").required(true).multiple(true))
                .arg(
                    Arg::with_name("output")
                        .help("Output filename without file extension")
                        .short("o")
                        .long("out")
                        .takes_value(true)
                        .required(false)
                        .default_value("out"),
                ),
        );

    let matches = app.get_matches();

    match matches.subcommand() {
        ("pack", Some(matches)) => {
            let input = matches
                .values_of("INPUT")
                .map(|values| values.map(|it| String::from(it)).collect::<Vec<String>>())
                .unwrap_or(Vec::new());

            let out = matches
                .value_of("output")
                .expect("Unreachable: param has default value");

            do_pack(input, out);
        }
        _ => {}
    }
}

fn do_pack(input: Vec<String>, output_path: &str) {
    let mut sprites = Vec::new();

    for path in input {
        let img = image::open(&path).expect("Failed to open image");
        let img = img.as_rgba8().expect("Failed to convert image to rgba8");

        let dimensions = img.dimensions();
        let bytes = img
            .pixels()
            .flat_map(|it| it.data.iter().map(|it| *it))
            .collect::<Vec<u8>>();

        let sprite = InputSprite { dimensions, bytes };
        sprites.push(sprite);
    }

    // NOTE(happenslol): By default, we're using rgba8 right now,
    // so the stride is always 4
    let sprite_sheet = sheep::pack::<SimplePacker>(sprites, 4);
    let meta = sheep::encode::<AmethystFormat>(&sprite_sheet);

    let outbuf = image::RgbaImage::from_vec(
        sprite_sheet.dimensions.0,
        sprite_sheet.dimensions.1,
        sprite_sheet.bytes,
    ).expect("Failed to construct image from sprite sheet bytes");

    outbuf
        .save(format!("{}.png", output_path))
        .expect("Failed to save image");

    let mut meta_file =
        File::create(format!("{}.ron", output_path)).expect("Failed to create meta file");

    let meta_str = ron::ser::to_string(&meta).expect("Failed to encode meta file");

    meta_file
        .write_all(meta_str.as_bytes())
        .expect("Failed to write meta file");
}
