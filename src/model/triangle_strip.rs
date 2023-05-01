use crate::vec::{Vec2, Vec3};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::io::{self, ErrorKind, Read, Seek, SeekFrom, Write};

pub struct TriangleStrip {
    pub pos: Vec<Vec3<f32>>,
    pub norm: Vec<Vec3<i8>>,
    pub uv: Vec<Vec2<f32>>,
    pub material: u32,
}

impl TriangleStrip {
    pub fn read<T: Read + Seek>(reader: &mut T, material: &mut u32) -> io::Result<Self> {
        // read until 0x10 aligned
        let reader_pos = reader.seek(SeekFrom::Current(0))?;
        let diff = 0x10 - (reader_pos % 0x10);
        if diff != 0x10 {
            reader.seek(SeekFrom::Current(diff as _))?;
        }
        // discard next 8 bytes
        reader.seek(SeekFrom::Current(8))?;
        // discard until triangle strip begins (or found material)
        // (material code should probably be outside of this function, but it is not)
        loop {
            const FIND_TS_SIGNATURE: [u8; 8] = [0x00, 0x00, 0x00, 0x20, 0x40, 0x40, 0x40, 0x40];
            const FIND_MAT_SIGNATURE: [u8; 8] = [0x00, 0x00, 0x00, 0x00, 0xFE, 0xFF, 0xFF, 0xFF];
            let mut signature = [0; 8];
            let read = reader.read(&mut signature)?;
            if read == 0 {
                println!("Model EOF!");
                return Err(ErrorKind::UnexpectedEof.into());
            }
            match signature {
                FIND_TS_SIGNATURE => break,
                FIND_MAT_SIGNATURE => {
                    *material = reader.read_u32::<LE>()? / 2;
                    reader.seek(SeekFrom::Current(4))?;
                }
                _ => {
                    reader.seek(SeekFrom::Current(8))?;
                }
            }
        }

        // read vertex positions
        let pos = TriangleStrip::read_pos(reader)?;

        // read vertex normals
        let norm = TriangleStrip::read_norm(reader)?;

        // read unknown
        // println!("    - Reading vertex unknown");
        let unknown_head = reader.read_u32::<LE>()?;
        assert_eq!(unknown_head & 0xFF00FFFF, 0x6E00C006);
        let unknown_count = (unknown_head >> 16 & 0xFF) as _;
        let mut unknown = Vec::with_capacity(unknown_count);
        for _ in 0..unknown_count {
            unknown.push(reader.read_u32::<LE>()?);
        }
        if unknown != vec![0x80808080; unknown_count] {
            // print!("STRANGE UNKNOWN: [ ");
            // for unknown in unknown {
            //     print!("{:x} ", unknown);
            // }
            // println!("]");
        }

        // read texture coordinates
        let uv = TriangleStrip::read_uv(reader)?;

        // read until 0x10 aligned
        let reader_pos = reader.seek(SeekFrom::Current(0))?;
        let diff = 0x10 - (reader_pos % 0x10);
        if diff != 0x10 {
            reader.seek(SeekFrom::Current(diff as _))?;
        }

        Ok(Self {
            pos,
            norm,
            uv,
            material: *material,
        })
    }

    pub fn write<T: Write>(&self, writer: &mut T) -> io::Result<()> {
        self.write_pos(writer)?;
        self.write_norm(writer)?;

        // write unknown header
        let unknown_count = self.pos.len() as u32; // probably should be same as rest for length
        let unknown_head = 0x78008004 | (unknown_count << 16);
        writer.write_u32::<LE>(unknown_head)?;
        // write unknown
        for _ in 0..unknown_count {
            writer.write_u32::<LE>(0x80808080)?
        }

        self.write_uv(writer)?;

        Ok(())
    }

    fn read_pos<T: Read + Seek>(reader: &mut T) -> io::Result<Vec<Vec3<f32>>> {
        // read vertex position header and assert it is correct
        // println!("    - Reading vertex positions");
        let pos_head = reader.read_u32::<LE>()?;
        assert_eq!(pos_head & 0xFF00FFFF, 0x78008004);

        // read vertex position count from header
        let pos_count = (pos_head >> 16 & 0xFF) as _;
        let mut pos = Vec::with_capacity(pos_count);

        // read vertex positions
        for _ in 0..pos_count {
            pos.push(Vec3::<f32> {
                x: reader.read_f32::<LE>()?,
                y: reader.read_f32::<LE>()?,
                z: reader.read_f32::<LE>()?,
            });
        }
        Ok(pos)
    }

    fn write_pos<T: Write>(&self, writer: &mut T) -> io::Result<()> {
        // write vertex positions header
        let pos_count = self.pos.len() as u32;
        let pos_head = 0x78008004 | (pos_count << 16);
        writer.write_u32::<LE>(pos_head)?;

        // write vertex positions
        for pos in &self.pos {
            writer.write_f32::<LE>(pos.x)?;
            writer.write_f32::<LE>(pos.y)?;
            writer.write_f32::<LE>(pos.z)?;
        }
        Ok(())
    }

    fn read_norm<T: Read + Seek>(reader: &mut T) -> io::Result<Vec<Vec3<i8>>> {
        // read vertex normal header and assert it is correct
        // println!("    - Reading vertex normals");
        let norm_head = reader.read_u32::<LE>()?;
        assert_eq!(norm_head & 0xFF00FFFF, 0x7E008005);

        // read vertex normal count from header
        let norm_count = (norm_head >> 16 & 0xFF) as _;
        let mut norm = Vec::with_capacity(norm_count);

        // read vertex normals
        for _ in 0..norm_count {
            norm.push(Vec3::<i8> {
                x: reader.read_i8()?,
                y: reader.read_i8()?,
                z: reader.read_i8()?,
            });
            // normal has byte at end for padding
            let vtx_norm_padding_byte = reader.read_i8()?;
            assert_eq!(vtx_norm_padding_byte, 0x00);
        }
        Ok(norm)
    }

    fn write_norm<T: Write>(&self, writer: &mut T) -> io::Result<()> {
        // write vertex normals header
        let norm_count = self.norm.len() as u32;
        let norm_head = 0x7E008005 | (norm_count << 16);
        writer.write_u32::<LE>(norm_head)?;

        // write vertex normals
        for norm in &self.norm {
            writer.write_i8(norm.x)?;
            writer.write_i8(norm.y)?;
            writer.write_i8(norm.z)?;
            writer.write_i8(0x00)?; // padding byte
        }
        Ok(())
    }

    fn read_uv<T: Read + Seek>(reader: &mut T) -> io::Result<Vec<Vec2<f32>>> {
        // read vertex texture coordinate header and assert it is correct
        // println!("    - Reading vertex texture coordinates");
        let uv_head = reader.read_u32::<LE>()?;
        assert_eq!(uv_head & 0xFF00FFFF, 0x64008007);

        // read texture coordinate count from header
        let uv_count = (uv_head >> 16 & 0xFF) as _;
        let mut uv = Vec::with_capacity(uv_count);

        // read texture coordinates
        for _ in 0..uv_count {
            uv.push(Vec2::<f32> {
                x: reader.read_f32::<LE>()?,
                y: reader.read_f32::<LE>()?,
            });
        }
        Ok(uv)
    }

    fn write_uv<T: Write>(&self, writer: &mut T) -> io::Result<()> {
        // write vertex positions header
        let uv_count = self.uv.len() as u32;
        let uv_head = 0x78008004 | (uv_count << 16);
        writer.write_u32::<LE>(uv_head)?;

        // write vertex positions
        for uv in &self.uv {
            writer.write_f32::<LE>(uv.x)?;
            writer.write_f32::<LE>(uv.y)?;
        }
        Ok(())
    }
}
