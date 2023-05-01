use crate::texture::{texture_format::TextureFormat, Color};
use byteorder::*;
use std::fs::{self, File};
use std::io::{self, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

// 4-bit encoded textures should only need 16 colors
// 8-bit encoded textures can have up to 256 colors
// textures I use for testing can have any amount of colors
pub struct Palette(pub Vec<Color>);

impl Palette {
    pub fn read_from_bin<T: Read + Seek>(reader: &mut T, tf: TextureFormat) -> io::Result<Self> {
        // find palette
        let mut palette_offset = tf.size.x * tf.size.y;
        if tf.pixel_encoding == 4 {
            palette_offset /= 2;
        }

        // read palette from file
        // normalised to 8 bits per channel
        let mut palette = vec![Color::default(); 2usize.pow(tf.pixel_encoding as _)];
        reader.seek(SeekFrom::Start((tf.location + palette_offset) as _))?;
        // todo: is unwrap_or_default still needed now that 4-bit only has 16 colors instead of 256?
        for color in palette.iter_mut() {
            *color = match tf.color_depth {
                16 => {
                    let bytes = reader.read_u16::<LE>().unwrap_or_default();
                    Color {
                        r: ((bytes & 0b00011111) as f32 * 255.0 / 31.0).round() as _,
                        g: ((bytes >> 5 & 0b00011111) as f32 * 255.0 / 31.0).round() as _,
                        b: ((bytes >> 10 & 0b00011111) as f32 * 255.0 / 31.0).round() as _,
                        a: if bytes >> 15 == 1 { 0xFF } else { 0x00 },
                    }
                }
                32 => Color {
                    r: reader.read_u8().unwrap_or_default(),
                    g: reader.read_u8().unwrap_or_default(),
                    b: reader.read_u8().unwrap_or_default(),
                    a: (reader.read_u8().unwrap_or_default() as f32 * 255.0 / 127.0).round() as _,
                },
                _ => unreachable!(),
            }
        }

        // if necessary (8-bit images) unscramble the palette
        if tf.pixel_encoding == 8 {
            for row in (8..255).step_by(32) {
                for row in row..row + 8 {
                    //switches every other pair of 8-color chunks in the palette
                    let temp = palette[row];
                    palette[row] = palette[row + 8];
                    palette[row + 8] = temp;
                }
            }
        }

        Ok(Self(palette))
    }

    pub fn write_to_bin<T: Write + Seek>(
        &self,
        writer: &mut T,
        tf: TextureFormat,
    ) -> io::Result<()> {
        // find palette
        let mut palette_offset = tf.size.x * tf.size.y;
        if tf.pixel_encoding == 4 {
            palette_offset /= 2;
        }

        // if necessary (8-bit images) scramble the palette
        let mut palette = self.0.clone();
        if tf.pixel_encoding == 8 {
            // for 8x8, for every 4 rows, swap the middle two rows
            for row in (8..255).step_by(32) {
                for row in row..row + 8 {
                    //switches every other pair of 8-color chunks in the palette
                    let temp = self.0[row];
                    palette[row] = self.0[row + 8];
                    palette[row + 8] = temp;
                }
            }
        }

        // write palette to file
        // normalised to 8 bits per channel
        writer.seek(SeekFrom::Start((tf.location + palette_offset) as _))?;
        for color in palette.iter() {
            match tf.color_depth {
                16 => {
                    let r = (color.r as f32 / 255.0 * 31.0).round() as u16;
                    let g = (color.g as f32 / 255.0 * 31.0).round() as u16;
                    let b = (color.b as f32 / 255.0 * 31.0).round() as u16;
                    let a = if color.a == 0xFF { 1 } else { 0 };
                    let bytes = (r & 0b00011111)
                        | ((g & 0b00011111) << 5)
                        | ((b & 0b00011111) << 10)
                        | ((a & 0b00000001) << 15);
                    writer.write_u16::<LE>(bytes)?;
                }
                32 => {
                    writer.write_u8(color.r)?;
                    writer.write_u8(color.g)?;
                    writer.write_u8(color.b)?;
                    writer.write_u8((color.a as f32 / 255.0 * 127.0).round() as u8)?;
                }
                _ => unreachable!(),
            }
        }

        Ok(())
    }
}
