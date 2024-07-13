use std::{
    collections::HashMap,
    env,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use texture::Texture;

use crate::{ddm::DDM, model::Models, texture::Textures};

mod ddm;
mod model;
mod texture;
mod vec;

fn process_dir(file_path: &Path) -> std::io::Result<()> {
    for entry in std::fs::read_dir(file_path)? {
        let file_path = entry?.path();
        process_file(&file_path)?;
    }
    Ok(())
}

fn process_file(file_path: &Path) -> std::io::Result<()> {
    if file_path.is_dir() {
        process_dir(file_path)?;
    } else if let Some(ext) = file_path.extension() {
        let ext = ext.to_str().unwrap();
        if ext == "BIN" {
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
        } else if ext == "ddm" {
            println!("path: {}", file_path.to_str().unwrap());
            // read from ddm file
            let ddm = DDM::read(&file_path)?;
            // write to dds files
            for (i, dds) in ddm.0.iter().enumerate() {
                dds.write(&file_path, i)?;
            }
        }
    }
    Ok(())
}

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
            let file_path = Path::new(&args[1]);
            if file_path.is_dir() {
                process_dir(file_path)?;
            } else {
                process_file(file_path)?;
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
