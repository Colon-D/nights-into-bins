use crate::texture::convert_8bit::swap_4x4_chunks;
use crate::texture::{convert_4bit, convert_8bit, texture_format::TextureFormat, CHUNK_SIZE};
use byteorder::*;
use ndarray::{s, Array2};
use std::fs::{self, File};
use std::io::{self, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::process::exit;

// texture before applying palette
pub struct PaletteTexture(
    // u16 used for testing
    pub Array2<u16>,
);

impl PaletteTexture {
    pub fn read_from_bin<T: Read + Seek>(reader: &mut T, tf: TextureFormat) -> io::Result<Self> {
        // read palette indexes texture
        reader.seek(SeekFrom::Start((tf.location) as _))?;
        let mut palette_tex: Array2<u16> = Array2::default((tf.size.y as _, tf.size.x as _));
        let mut nibble_leftover = None;

        for value in palette_tex.iter_mut() {
            match tf.pixel_encoding {
                8 => {
                    let palette_index = reader.read_u8().unwrap_or_default();
                    *value = palette_index as _;
                }
                4 => {
                    let palette_index = nibble_leftover.take().unwrap_or_else(|| {
                        let byte = reader.read_u8().unwrap_or_default();
                        nibble_leftover = Some(byte >> 4);
                        byte & 0x0F
                    });
                    *value = palette_index as _;
                }
                _ => unreachable!(),
            }
        }

        // decode
        match tf.pixel_encoding {
            8 => {
                palette_tex = convert_8bit::decode(&palette_tex);
            }
            4 => {
                // if width is 64, decode blah blah blah
                if tf.size.x == 64 && tf.size.y == 64 {
                    convert_4bit::convert64x64_4bit(palette_tex.as_slice_mut().unwrap());
                }
                // if width is 32, each 32x32 pixel chunk needs decoded
                if tf.size.x == 32 && tf.size.y >= 32 {
                    for h in (0..tf.size.y as usize).step_by(32) {
                        convert_4bit::convert32x32(
                            palette_tex
                                .slice_mut(s![h..h + 32, ..])
                                .as_slice_mut()
                                .unwrap(),
                        );
                    }
                }
            }
            _ => unreachable!(),
        }

        Ok(Self(palette_tex))
    }

    pub fn write_to_bin<T: Write + Seek>(
        &self,
        writer: &mut T,
        tf: TextureFormat,
    ) -> io::Result<()> {
        // encode
        let encoded = match tf.pixel_encoding {
            8 => convert_8bit::encode(&self.0),
            4 => {
                eprintln!("Error: Writing 4-bit images is currently unsupported");
                std::process::exit(1)
            }
            _ => unreachable!(),
        };

        // write palette indexes texture
        writer.seek(SeekFrom::Start((tf.location) as _))?;
        for value in encoded.iter() {
            writer.write_u8(*value as _)?;
        }

        Ok(())
    }
}
