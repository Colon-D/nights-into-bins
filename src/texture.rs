use self::palette::Palette;
use self::palette_texture::PaletteTexture;
use self::texture_format::TextureFormat;
use self::texture_format::TextureFormats;
use std::fs::File;
use std::io::{self, BufWriter, ErrorKind, Read, Seek, Write};
use std::path::Path;

pub mod convert_4bit;
pub mod convert_8bit;
mod palette;
mod palette_texture;
pub mod test;
mod texture_format;

// todo: flag
const VERBOSE: bool = false;

pub struct Textures(pub Vec<Texture>);

impl Textures {
    pub fn read_from_bin(path: &Path) -> io::Result<Self> {
        let mut reader = File::open(path)?;

        let tfs = match TextureFormats::read(&mut reader, VERBOSE) {
            Ok(tfs) => tfs,
            Err(e) => {
                if e.kind() == ErrorKind::UnexpectedEof {
                    return Ok(Self(Vec::new()));
                } else {
                    return Err(e);
                }
            }
        };

        let mut textures = Vec::new();
        for tf in tfs.0.iter() {
            let texture = Texture::read_from_bin(&mut reader, *tf)?;
            textures.push(texture);
        }
        Ok(Self(textures))
    }

    pub fn write_to_png(&self, file_path: &Path) -> io::Result<()> {
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
        for (i, texture) in self.0.iter().enumerate() {
            // create file and write to png
            let mut file = File::create(format!("out/{}/{}-{}.png", stem, stem, i)).unwrap();
            texture.write_to_png(&mut file);
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
pub struct Texture {
    pub data: Vec<Color>,
    pub width: usize,
}

impl Texture {
    pub fn read_from_bin<T: Read + Seek>(reader: &mut T, tf: TextureFormat) -> io::Result<Self> {
        let palette = Palette::read_from_bin(reader, tf)?;
        let palette_tex = PaletteTexture::read_from_bin(reader, tf)?;

        Ok(Self::from_palette_and_palette_texture(
            &palette,
            &palette_tex,
        ))
    }

    pub fn from_palette_and_palette_texture(
        palette: &Palette,
        palette_tex: &PaletteTexture,
    ) -> Self {
        let mut tex: Vec<Color> = Vec::with_capacity(palette_tex.data.len());
        for i in palette_tex.data.iter() {
            let color = palette.0[*i as usize];
            tex.push(color);
        }

        Self {
            data: tex,
            width: palette_tex.width,
        }
    }

    pub fn write_to_png<T: Write>(&self, writer: &mut T) {
        let ref mut w = BufWriter::new(writer);

        let mut encoder =
            png::Encoder::new(w, self.width as _, (self.data.len() / self.width) as _);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();

        let tex_flat: Vec<u8> = self
            .data
            .iter()
            .flat_map(|p| [p.r, p.g, p.b, p.a])
            .collect();
        writer.write_image_data(&tex_flat).unwrap();
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

pub const CHUNK_SIZE: usize = 32;
