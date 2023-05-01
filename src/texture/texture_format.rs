use crate::vec::Vec2;
use byteorder::*;
use std::io::{self, ErrorKind, Read, Seek, SeekFrom};

pub struct TextureFormats(pub Vec<TextureFormat>);

impl TextureFormats {
    pub fn read<T: Read + Seek>(reader: &mut T, verbose: bool) -> io::Result<Self> {
        let mut texture_formats = Vec::new();

        // discard until 06 00 00 10 00 00 00 00 00 00 00 00 00 00 00 00
        loop {
            const FIND_SIGNATURE: [u8; 16] = [
                0x06, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00,
            ];
            let mut signature = [0; 16];
            let read = reader.read(&mut signature)?;
            if read == 0 {
                println!("Texture EOF!");
                return Err(ErrorKind::UnexpectedEof.into());
            }
            if signature == FIND_SIGNATURE {
                break;
            }
        }

        loop {
            // skip
            reader.seek(SeekFrom::Current(0x17))?;

            // read double size flag
            let double_size = reader.read_u8()? == 0;

            // skip
            reader.seek(SeekFrom::Current(0x18))?;

            // read size
            let mut size = Vec2 {
                x: reader.read_u32::<LE>()?,
                y: reader.read_u32::<LE>()?,
            };
            if size.x > 512 || size.y > 512 {
                println!(
                    "size [{}, {}] is way too big. I'm outta here!",
                    size.x, size.y
                );
                break;
            }
            if double_size {
                size = size * 2;
            }

            // skip
            reader.seek(SeekFrom::Current(0x5F))?;

            // read color depth (bits)
            let color_depth = if reader.read_u8()? == 0 { 32 } else { 16 };

            // skip
            reader.seek(SeekFrom::Current(0x48))?;

            // read pixel encoding (bits)
            let pixel_encoding = if reader.read_u8()? == 2 { 4 } else { 8 };
            // for some reason height might need double again?
            if pixel_encoding == 4 && double_size {
                size.y *= 2;
            }

            texture_formats.push(TextureFormat {
                size,
                color_depth,
                pixel_encoding,
                location: 0, // not set yet
            });

            // skip
            reader.seek(SeekFrom::Current(0x0F))?;

            // read end
            const FIND_END: [u8; 16] = [
                0x00, 0x00, 0x00, 0x60, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00,
            ];
            let mut end = [0; 16];
            reader.read(&mut end)?;
            if end == FIND_END {
                break;
            }
        }

        let counter = reader.seek(SeekFrom::Current(0))? as u32;

        // align to 0x100
        let mut tex_location = (0x100 - (counter % 0x100)) + counter;

        // calculate texture locations
        for tf in texture_formats.iter_mut() {
            tf.location = tex_location;
            match tf.color_depth {
                16 => match tf.pixel_encoding {
                    4 => tex_location += tf.size.x * tf.size.y / 2 + 0x20,
                    8 => tex_location += tf.size.x * tf.size.y + 0x200,
                    _ => unreachable!(),
                },
                32 => tex_location += tf.size.x * tf.size.y + 0x400,
                _ => unreachable!(),
            }
        }

        // output information
        if verbose {
            println!("Texture formats:");
            for (i, tf) in texture_formats.iter().enumerate() {
                println!("- Texture format:");
                println!(
                    "    size: [{}, {}], color_depth: {}, pixel_encoding: {}, location: {:#x}, index: {}",
                    tf.size.x, tf.size.y, tf.color_depth, tf.pixel_encoding, tf.location, i
                );
            }
        }

        Ok(Self(texture_formats))
    }
}

#[derive(Clone, Copy)]
pub struct TextureFormat {
    // pixels?
    pub size: Vec2<u32>,
    // bits
    pub color_depth: u8,
    // bits
    pub pixel_encoding: u8,
    // bytes
    pub location: u32,
}
