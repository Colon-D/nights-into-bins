use crate::texture::{convert_4bit, convert_8bit, texture_format::TextureFormat, CHUNK_SIZE};
use byteorder::*;
use std::fs::{self, File};
use std::io::{self, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

// texture before applying palette
pub struct PaletteTexture {
    // u16 used for testing
    pub data: Vec<u16>,
    pub width: usize,
}

impl PaletteTexture {
    pub fn read_from_bin<T: Read + Seek>(reader: &mut T, tf: TextureFormat) -> io::Result<Self> {
        // u16 used incase of testing
        let mut palette_tex: Vec<u16>;

        // read palette indexes texture
        reader.seek(SeekFrom::Start((tf.location) as _))?;
        palette_tex = Vec::with_capacity((tf.size.x * tf.size.y) as _);
        let mut nibble_leftover = None;
        for _ in 0..(tf.size.x * tf.size.y) {
            match tf.pixel_encoding {
                8 => {
                    let palette_index = reader.read_u8().unwrap_or_default();
                    palette_tex.push(palette_index as _);
                }
                4 => {
                    let palette_index = nibble_leftover.take().unwrap_or_else(|| {
                        let byte = reader.read_u8().unwrap_or_default();
                        nibble_leftover = Some(byte >> 4);
                        byte & 0x0F
                    });
                    palette_tex.push(palette_index as _);
                }
                _ => unreachable!(),
            }
        }

        // decode
        let mut flip = true;
        match tf.pixel_encoding {
            8 => {
                if tf.size.x >= CHUNK_SIZE as _ {
                    let num_rows = tf.size.y / 2;
                    let num_chunks = tf.size.x / 16;
                    convert_8bit::convert_array(num_rows as _, num_chunks as _, &mut palette_tex);
                }
            }
            4 => {
                // if width is 64, decode blah blah blah
                if tf.size.x == 64 && tf.size.y == 64 {
                    flip = false;
                    convert_4bit::convert64x64_4bit(&mut palette_tex);
                }
                // if width is 32, each 32x32 pixel chunk needs decoded
                if tf.size.x == 32 && tf.size.y >= 32 {
                    for h in (0..tf.size.y).step_by(32) {
                        convert_4bit::convert32x32(
                            &mut palette_tex[32 * h as usize..32 * (h + 32) as usize],
                        );
                    }
                }
            }
            _ => unreachable!(),
        }

        // flip vertical for png output
        if flip {
            for row in 0..tf.size.y / 2 {
                for col in 0..tf.size.x {
                    let index_a = row * tf.size.x + col;
                    let index_b = (tf.size.y - 1 - row) * tf.size.x + col;
                    palette_tex.swap(index_a as _, index_b as _);
                }
            }
        }

        Ok(Self {
            data: palette_tex,
            width: tf.size.x as _,
        })
    }
}
