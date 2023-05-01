use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
};

use texture::Texture;

use crate::{model::Models, texture::Textures};

mod model;
mod texture;
mod vec;

fn main() -> std::io::Result<()> {
    // //* TEST
    // //* Create test texture
    // let mut test_pal = texture::test::palette(texture::test::Test::_4096Colors64x64);
    // let mut test_tex = texture::test::texture(texture::test::Test::_4096Colors64x64);

    // //* Decode/Encode
    // // test_tex.0 = texture::convert_8bit::encode(&test_tex.0);
    // // test_tex.0 = texture::convert_8bit::decode(&test_tex.0);

    // //* Export
    // let test_tex = Texture::from_palette_and_palette_texture(&test_pal, &test_tex);
    // test_tex.write_to_image(Path::new("4-bit_64x64_decoded.png"), false);

    // return Ok(());
    // //* END TEST

    let args: Vec<String> = env::args().collect();

    match args.len() {
        2 => {
            let binary_path = Path::new(&args[1]);

            if binary_path.is_dir() {
                for entry in std::fs::read_dir(binary_path)? {
                    let file_path = entry?.path();
                    if file_path.extension().unwrap().to_str().unwrap() == "BIN" {
                        println!("path: {}", file_path.to_str().unwrap());
                        // read from bin file
                        let models = Models::read_from_bin(&file_path)?;
                        let textures = Textures::read_from_bin(&file_path)?;
                        // write to obj files
                        models.write_to_obj(&file_path)?;
                        if !models.0.is_empty() {
                            textures.write_to_mtl(&file_path)?;
                        }
                        // write to png files
                        textures.write_to_image(&file_path)?;
                    }
                }
            } else {
                // read from bin file
                let models = Models::read_from_bin(&binary_path)?;
                let textures = Textures::read_from_bin(&binary_path)?;
                // write to obj files
                models.write_to_obj(&binary_path)?;
                if !models.0.is_empty() {
                    textures.write_to_mtl(&binary_path)?;
                }
                // write to png files
                textures.write_to_image(&binary_path)?;
            }
        }
        3 => {
            let binary_path = Path::new(&args[1]);
            let replacement_path = Path::new(&args[2]);

            let mut replacement_textures = HashMap::<PathBuf, Textures>::new();
            for entry in walkdir::WalkDir::new(replacement_path) {
                let entry = entry?;
                let file_path = entry.path();
                match file_path.extension() {
                    Some(ext) => {
                        if ext.to_str().unwrap() == "png" {
                            // read replacement textures
                            println!("path: {}", file_path.to_str().unwrap());
                            let stem = file_path.file_stem().unwrap().to_str().unwrap();
                            let seperator = stem.find('-').unwrap();
                            let binary_file_stem = &stem[..seperator];
                            let texture_index = stem[seperator + 1..].parse().unwrap();
                            replacement_textures
                                .entry(PathBuf::from(binary_file_stem).with_extension("BIN"))
                                .or_default()
                                .0
                                .insert(texture_index, Texture::read_from_image(&file_path, true));
                        }
                    }
                    None => (),
                }
            }
            // write replacement texture
            for (binary_file_stem, textures) in replacement_textures {
                textures.write_to_bin(&binary_path.join(binary_file_stem))?;
            }
        }
        _ => {
            eprintln!("Error. Usage:\n  ./nights_into_bins <binary_file>\n    extracts textures and models from binary files in directory and exports into ./out/\n  ./nights_into_bins <binary_file_directory>\n    extracts textures and models from binary file and exports into ./out/\n  ./nights_into_bins <binary_file_directory> <texture_replacement_file_directory>\n    copies binary files into mod at ./in/nights.test.nightsintobins/ and replaces their textures");
            std::process::exit(1);
        }
    }

    Ok(())
}
