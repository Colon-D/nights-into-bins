use image::imageops;
use image::GenericImageView;
use image::ImageBuffer;
use image::Rgba;
use ndarray::Array2;

use self::palette::Palette;
use self::palette_texture::PaletteTexture;
use self::texture_format::TextureFormat;
use self::texture_format::TextureFormats;
use std::collections::HashMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{self, BufWriter, ErrorKind, Read, Seek, Write};
use std::path::Path;

pub mod convert_4bit;
pub mod convert_8bit;
mod palette;
mod palette_texture;
pub mod test;
pub mod texture_format;

// todo: flag
const VERBOSE: bool = false;

#[derive(Default)]
pub struct Textures(pub HashMap<usize, Texture>);

impl Textures {
    pub fn read_from_bin(path: &Path) -> io::Result<Self> {
        let mut reader = File::open(path)?;

        let tfs = match TextureFormats::read(&mut reader, VERBOSE) {
            Ok(tfs) => tfs,
            Err(e) => {
                if e.kind() == ErrorKind::UnexpectedEof {
                    return Ok(Self(HashMap::new()));
                } else {
                    return Err(e);
                }
            }
        };

        let mut textures = HashMap::new();
        for tf in tfs.0.iter() {
            let texture = Texture::read_from_bin(&mut reader, *tf)?;
            textures.insert(textures.len(), texture);
        }
        Ok(Self(textures))
    }

    pub fn write_to_bin(&self, original: &Path) -> io::Result<()> {
        // create dir if it does not exist
        let out_dir = Path::new("in/nights.test.nightsintobins/Redirector/afs/");
        if !out_dir.exists() {
            std::fs::create_dir_all(out_dir)?;
        }

        // copy original file
        let copy = out_dir.join(original.file_name().unwrap());
        std::fs::copy(original, &copy)?;

        // read texture formats
        let mut file = OpenOptions::new().read(true).write(true).open(copy)?;
        let tfs = TextureFormats::read(&mut file, false)?;

        // write textures
        for (i, t) in self.0.iter() {
            t.write_to_bin(&mut file, tfs.0[*i])?;
        }
        Ok(())
    }

    pub fn write_to_image(&self, file_path: &Path) -> io::Result<()> {
        if self.0.is_empty() {
            return Ok(());
        }
        // create dir if it does not exist
        let stem = file_path.file_stem().unwrap().to_str().unwrap();
        let dir_path = format!("out/{}", stem);
        let dir_path = Path::new(&dir_path);
        if !dir_path.exists() {
            std::fs::create_dir_all(dir_path)?;
        }
        // iterate through each texture
        for (i, texture) in self.0.iter() {
            // create file and write to png
            texture.write_to_image(Path::new(&format!("out/{}/{}-{}.png", stem, stem, i)), true);
        }
        Ok(())
    }

    pub fn write_to_mtl(&self, file_path: &Path) -> io::Result<()> {
        if self.0.is_empty() {
            return Ok(());
        }

        let stem = file_path.file_stem().unwrap().to_str().unwrap();
        let mtl_filename = format!("out/{}/{}.mtl", stem, stem);

        // write mtl file
        let mut mtl = File::create(mtl_filename)?;

        for i in 0..self.0.len() {
            writeln!(mtl, "newmtl {}-{}", &stem, i)?;
            writeln!(mtl, "map_Kd {}-{}.png", &stem, i)?;
        }

        Ok(())
    }
}
pub struct Texture(pub Array2<Color>);

impl Texture {
    pub fn read_from_bin<T: Read + Seek>(reader: &mut T, tf: TextureFormat) -> io::Result<Self> {
        let palette = Palette::read_from_bin(reader, tf)?;
        let palette_tex = PaletteTexture::read_from_bin(reader, tf)?;

        Ok(Self::from_palette_and_palette_texture(
            &palette,
            &palette_tex,
        ))
    }

    pub fn write_to_bin<T: Write + Seek>(
        &self,
        writer: &mut T,
        tf: TextureFormat,
    ) -> io::Result<()> {
        let (pal, pal_tex) = self.to_palette_and_palette_texture(tf);
        pal.write_to_bin(writer, tf)?;
        pal_tex.write_to_bin(writer, tf)
    }

    pub fn from_palette_and_palette_texture(
        palette: &Palette,
        palette_tex: &PaletteTexture,
    ) -> Self {
        Self(palette_tex.0.map(|i| palette.0[*i as usize]))
    }

    /// image should be flipped, unless you are testing something
    pub fn write_to_image(&self, path: &Path, flip: bool) {
        let (width, height) = (self.0.ncols(), self.0.nrows());
        let mut img = ImageBuffer::new(width as u32, height as u32);

        for y in 0..height {
            for x in 0..width {
                let col = &self.0[[y, x]];
                let pixel = Rgba([col.r, col.g, col.b, col.a]);
                img.put_pixel(x as u32, y as u32, pixel);
            }
        }
        if flip {
            imageops::flip_vertical_in_place(&mut img);
        }
        img.save(path).unwrap();
    }

    /// image should be flipped, unless you are testing something
    pub fn read_from_image(path: &Path, flip: bool) -> Texture {
        let mut img = image::open(path).unwrap();
        imageops::flip_vertical_in_place(&mut img);
        let (width, height) = img.dimensions();
        let mut data = Array2::default((height as usize, width as usize));

        for (x, y, pixel) in img.pixels() {
            let rgba = pixel.0;
            data[[y as usize, x as usize]] = Color {
                r: rgba[0],
                g: rgba[1],
                b: rgba[2],
                a: rgba[3],
            };
        }

        Texture(data)
    }

    pub fn to_palette_and_palette_texture(&self, tf: TextureFormat) -> (Palette, PaletteTexture) {
        let max_len = 2usize.pow(tf.pixel_encoding as _);
        let mut palette = Vec::with_capacity(max_len);
        let mut palette_map = HashMap::with_capacity(256);
        let mut palette_tex = Array2::default((self.0.nrows(), self.0.ncols()));

        // iterate through each color in texture, and index in palette texture
        for (tex_c, tex_i) in self.0.iter().zip(palette_tex.iter_mut()) {
            // set index in palette texture
            *tex_i = *palette_map.entry(tex_c).or_insert_with(|| {
                let i = palette.len();
                if i == max_len {
                    eprintln!("Error: Too many colors");
                    std::process::exit(1);
                }
                palette.push(*tex_c);
                i as _
            });
        }
        palette.resize(max_len, Color::default());
        (Palette(palette), PaletteTexture(palette_tex))
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

pub const CHUNK_SIZE: usize = 32;
