use crate::texture::{texture_format::TextureFormat, Color};
use byteorder::*;
use std::fs::{self, File};
use std::io::{self, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

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
        let mut palette: Vec<Color>;
        palette = vec![
            Color {
                r: 0xFF,
                g: 0xFF,
                b: 0xFF,
                a: 0xFF,
            };
            256
        ];
        reader.seek(SeekFrom::Start((tf.location + palette_offset) as _))?;
        // unwrap_or_default is used as color palette might be smaller
        // than 256, and might be at the end of the file
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
            // for 8x8, for every 4 rows, swap the middle two rows
            for i in (8..255).step_by(32) {
                for j in 0..8 {
                    //switches every other pair of 8-color chunks in the palette
                    let temp = palette[i + j];
                    palette[i + j] = palette[i + j + 8];
                    palette[i + j + 8] = temp;
                }
            }
        }

        Ok(Self(palette))
    }
}
