use std::{env, fs::File, path::Path};

use texture::Texture;

use crate::{model::Models, texture::Textures};

mod model;
mod texture;
mod vec;

fn main() -> std::io::Result<()> {
    //* TEST
    let test_pal = texture::test::palette(texture::test::Test::_8Bit16384Colors128x128);
    let test_tex_encoded = texture::test::texture(texture::test::Test::_8Bit16384Colors128x128);
    let mut test_tex_decoded =
        texture::test::texture(texture::test::Test::_8Bit16384Colors128x128);

    let width = 128usize;
    let height = 128usize;

    let num_rows = height / 2;
    let num_chunks = width / 16;
    texture::convert_8bit::convert_array(
        num_rows as _,
        num_chunks as _,
        &mut test_tex_decoded.data,
    );

    let test_tex_encoded = Texture::from_palette_and_palette_texture(&test_pal, &test_tex_encoded);
    let mut test_tex_decoded =
        Texture::from_palette_and_palette_texture(&test_pal, &test_tex_decoded);
    let mut file = File::create("test_encoded_3.png").unwrap();
    test_tex_encoded.write_to_png(&mut file);
    let mut file = File::create("test_decoded_3.png").unwrap();
    test_tex_decoded.write_to_png(&mut file);

    // for every eight rows, starting at 2
    for rows in (2..height).step_by(8) {
        // for every eight columns
        for col in (0..width).step_by(8) {
            // swap each 4x4 chunk with the 4x4 chunk next to it horizontally
            for sub_row in 0..4 {
                for sub_col in 0..4 {
                    let temp = test_tex_decoded.data[(rows + sub_row) * width + col + sub_col];
                    test_tex_decoded.data[(rows + sub_row) * width + col + sub_col] =
                        test_tex_decoded.data[(rows + sub_row) * width + col + sub_col + 4];
                    test_tex_decoded.data[(rows + sub_row) * width + col + sub_col + 4] = temp;
                }
            }
        }
    }
    let mut file = File::create("test_decoded_3_4x4.png").unwrap();
    test_tex_decoded.write_to_png(&mut file);

    return Ok(());

    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: program_name <input_file>");
        std::process::exit(1);
    }

    let in_path = Path::new(&args[1]);

    // //! THIS DOES NOT WORK
    // //! THIS WAS A TEST
    // //! USES ASSIMP
    // let suzanne_mesh = &Scene::from_file("suzanne.obj", vec![]).unwrap().meshes[0];
    // let mut suzanne_native = Model {
    //     triangle_strips: Vec::new(),
    // };
    // for face in suzanne_mesh.faces.iter() {
    //     let mut pos = Vec::new();
    //     let mut norm = Vec::new();
    //     let mut uv = Vec::new();
    //     for idx in face.0.iter() {
    //         let idx = *idx as usize;
    //         pos.push(
    //             suzanne_mesh
    //                 .vertices
    //                 .get(idx)
    //                 .map(|v| Vec3 {
    //                     x: v.x,
    //                     y: v.y,
    //                     z: v.z,
    //                 })
    //                 .unwrap(),
    //         );
    //         norm.push(
    //             suzanne_mesh
    //                 .normals
    //                 .get(idx)
    //                 .map(|n| Vec3 {
    //                     x: (n.x * 255.0) as _,
    //                     y: (n.y * 255.0) as _,
    //                     z: (n.z * 255.0) as _,
    //                 })
    //                 .unwrap(),
    //         );
    //         let suzanne_uv = suzanne_mesh.texture_coords[0].as_ref().unwrap();
    //         uv.push(
    //             suzanne_uv
    //                 .get(idx)
    //                 .map(|uv| Vec2 { x: uv.x, y: uv.y })
    //                 .unwrap(),
    //         );
    //     }
    //     suzanne_native
    //         .triangle_strips
    //         .push(TriangleStrip { pos, norm, uv });
    // }
    // write_model_to_bin(Path::new("test.bin"), &suzanne_native)?;

    if in_path.is_dir() {
        for entry in std::fs::read_dir(in_path)? {
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
                textures.write_to_png(&file_path)?;
            }
        }
    } else {
        // read from bin file
        let models = Models::read_from_bin(&in_path)?;
        let textures = Textures::read_from_bin(&in_path)?;
        // write to obj files
        models.write_to_obj(&in_path)?;
        if !models.0.is_empty() {
            textures.write_to_mtl(&in_path)?;
        }
        // write to png files
        textures.write_to_png(&in_path)?;
    }

    Ok(())
}
