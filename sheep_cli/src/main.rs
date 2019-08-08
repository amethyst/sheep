extern crate clap;
extern crate image;
extern crate ron;
extern crate serde;
extern crate sheep;

use clap::{App, AppSettings, Arg, SubCommand};
use image::RgbaImage;
use serde::Serialize;
use sheep::{
    AmethystFormat, AmethystNamedFormat, InputSprite, MaxrectsOptions, MaxrectsPacker, SimplePacker,
};
use std::str::FromStr;
use std::{fs::File, io::prelude::*};

const DEFAULT_FORMAT: &'static str = "amethyst";
const DEFAULT_PACKER: &'static str = "maxrects";

const AVAILABLE_FORMATS: [&'static str; 2] = ["amethyst", "amethyst_named"];
const AVAILABLE_PACKERS: [&'static str; 2] = ["simple", "maxrects"];

fn main() {
    let app = App::new("sheep")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("pack")
                .about("Packs supplied images into a spritesheet")
                .arg(Arg::with_name("INPUT").required(true).multiple(true))
                .arg(
                    Arg::with_name("output")
                        .help("Output filename without file extension")
                        .short("o")
                        .long("out")
                        .takes_value(true)
                        .required(false)
                        .default_value("out"),
                )
                .arg(
                    Arg::with_name("packer")
                        .help("Packing algorithm to use")
                        .possible_values(&AVAILABLE_PACKERS)
                        .short("p")
                        .long("packer")
                        .takes_value(true)
                        .required(false)
                        .default_value(DEFAULT_PACKER),
                )
                .arg(
                    Arg::with_name("format")
                        .help("Determines the fields present in the serialized output")
                        .possible_values(&AVAILABLE_FORMATS)
                        .short("f")
                        .long("format")
                        .takes_value(true)
                        .required(false)
                        .default_value(DEFAULT_FORMAT),
                )
                .arg(
                    Arg::with_name("options")
                        .help("Settings that will be passed to the selected packer")
                        .short("s")
                        .long("options")
                        .takes_value(true)
                        .multiple(true)
                        .required(false),
                )
                .arg(
                    Arg::with_name("pretty")
                        .help("The resulting .ron-file is formatted")
                        .long("pretty")
                        .required(false),
                )
                .arg(
                    Arg::with_name("trim")
                        .help("Trim transparent sprite sides")
                        .short("t")
                        .long("trim"),
                )
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

            let mut sprites = load_images(&input);

            if matches.is_present("trim") {
                // stride is 4 and alpha index is 3 because rgba8 is used by default
                sprites = sheep::trim(sprites.as_slice(), 4, 3);
            }

            // NOTE(happenslol): By default, we're using rgba8 right now,
            // so the stride is always 4
            let results = match matches.value_of("packer") {
                Some("maxrects") => {
                    let max_width = matches
                        .values_of("options")
                        .and_then(|mut options| options.find(|o| o.starts_with("max_width")))
                        .and_then(|found| found.split("=").nth(1))
                        .and_then(|value| value.parse::<u32>().ok())
                        .unwrap_or(4096);

                    let max_height = matches
                        .values_of("options")
                        .and_then(|mut options| options.find(|o| o.starts_with("max_height")))
                        .and_then(|found| found.split("=").nth(1))
                        .and_then(|value| value.parse::<u32>().ok())
                        .unwrap_or(4096);

                    let options = MaxrectsOptions::default()
                        .max_width(max_width)
                        .max_height(max_height);

                    sheep::pack::<MaxrectsPacker>(sprites, 4, options)
                }
                Some("simple") => sheep::pack::<SimplePacker>(sprites, 4, ()),
                _ => panic!("Unknown packer"),
            };

            if results.is_empty() {
                panic!("No output was produced");
            }

            let is_single_sheet = results.len() == 1;

            for (i, sheet) in results.iter().enumerate() {
                let filename = if i == 0 && is_single_sheet {
                    String::from(out)
                } else {
                    format!("{}-{:02}", out, i)
                };

                let outbuf = RgbaImage::from_vec(
                    sheet.dimensions.0,
                    sheet.dimensions.1,
                    sheet.bytes.clone(),
                )
                .expect("Failed to create image from spritesheet");

                let pretty = matches.is_present("pretty");

                match matches.value_of("format") {
                    Some("amethyst_named") => {
                        let names = get_filenames(&input);
                        let meta = sheep::encode::<AmethystNamedFormat>(&sheet, names);
                        write_files(&filename, outbuf, meta, pretty);
                    }
                    Some("amethyst") => {
                        let meta = sheep::encode::<AmethystFormat>(&sheet, ());
                        write_files(&filename, outbuf, meta, pretty);
                    }
                    _ => panic!("Unknown format"),
                };
            }
        }
        _ => {}
    }
}

fn get_filenames(input: &[String]) -> Vec<String> {
    input
        .iter()
        .map(|path| {
            std::path::PathBuf::from(&path)
                .file_stem()
                .and_then(|name| name.to_str())
                .map(|name| String::from_str(name).expect("could not parse string from file name"))
                .expect("Failed to extract file name")
        })
        .collect()
}

fn load_images(input: &[String]) -> Vec<InputSprite> {
    input
        .iter()
        .map(|path| {
            let img = image::open(&path).expect("Failed to open image");
            let img_owned;
            let img = {
                if let Some(img) = img.as_rgba8() {
                    img
                } else {
                    img_owned = img.to_rgba();
                    &img_owned
                }
            };

            let dimensions = img.dimensions();
            let bytes = img
                .pixels()
                .flat_map(|it| it.data.iter().map(|it| *it))
                .collect::<Vec<u8>>();

            InputSprite { dimensions, bytes }
        })
        .collect()
}

fn write_files<S: Serialize>(output_path: &str, outbuf: RgbaImage, meta: S, pretty: bool) {
    outbuf
        .save(format!("{}.png", output_path))
        .expect("Failed to save image");

    let mut meta_file =
        File::create(format!("{}.ron", output_path)).expect("Failed to create meta file");

    let meta_str = if pretty {
        ron::ser::to_string_pretty(&meta, ron::ser::PrettyConfig::default())
    } else {
        ron::ser::to_string(&meta)
    }
    .expect("Failed to encode meta file");

    meta_file
        .write_all(meta_str.as_bytes())
        .expect("Failed to write meta file");
}
