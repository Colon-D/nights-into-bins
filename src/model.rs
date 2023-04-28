use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use itertools::Itertools;
use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom, Write},
    path::Path,
};
use triangle_strip::TriangleStrip;

pub mod triangle_strip;

pub struct Models(pub Vec<Model>);

impl Models {
    pub fn read_from_bin(path: &Path) -> io::Result<Self> {
        let mut reader = File::open(path)?;

        let mut models = Vec::new();

        loop {
            match Model::read_from_bin(&mut reader) {
                Ok(model) => {
                    if !model.triangle_strips.is_empty() {
                        models.push(model)
                    }
                }
                Err(e) => {
                    if e.kind() == io::ErrorKind::UnexpectedEof {
                        break;
                    }
                }
            }
        }

        Ok(Self(models))
    }

    pub fn write_to_obj(&self, file_path: &Path) -> io::Result<()> {
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
        // iterate through each model
        let obj_file_path = format!("out/{}/{}.obj", stem, stem);
        let mut writer = File::create(obj_file_path)?;
        let mut e_next = 0;
        for (i, model) in self.0.iter().enumerate() {
            model.write_to_obj(&mut writer, &format!("{}-{}", stem, i), stem, &mut e_next)?;
        }

        Ok(())
    }
}

pub struct Model {
    pub triangle_strips: Vec<TriangleStrip>,
    pub material: u32,
}

impl Model {
    pub fn read_from_bin<T: Read + Seek>(reader: &mut T) -> io::Result<Self> {
        // read model signature
        // println!("- Reading model signature");
        loop {
            let file_signature = reader.read_u32::<LE>()?;
            if file_signature != 0x00001000 {
                // discard next 12 bytes
                reader.seek(SeekFrom::Current(12))?;
            } else {
                break;
            }
        }

        // discard next 4 bytes (unknown)
        reader.seek(SeekFrom::Current(4))?;

        // read expected vertex count
        // println!("- Reading vertex count");
        let expected_vertex_count = reader.read_u32::<LE>()? as usize;

        let mut triangle_strips = Vec::new();
        let mut material = 0;
        if expected_vertex_count == 0 {
            // example: 0x26D70 in DATCLARIS.BIN

            // read until 0x10 aligned and return an empty model
            let reader_pos = reader.seek(SeekFrom::Current(0))?;
            let diff = 0x10 - (reader_pos % 0x10);
            if diff != 0x10 {
                reader.seek(SeekFrom::Current(diff as _))?;
            }

            return Ok(Self {
                triangle_strips,
                material,
            });
        }

        // find material
        loop {
            let material_signature = reader.read_u32::<LE>()?;
            if material_signature != 0xFFFFFFFE {
                // discard next 12 bytes (aligned to 0x0C column)
                reader.seek(SeekFrom::Current(12))?;
            } else {
                break;
            }
        }
        material = reader.read_u32::<LE>()? / 2;

        let mut vertex_count = 0;
        // println!("- Reading triangle strips:");

        // while not reached vertex count
        while vertex_count < expected_vertex_count {
            //println!("vertex_count: {}/{}: ", vertex_count, expected_vertex_count);
            // println!("  - Reading triangle strip:");
            // println!("    - Finding triangle strip");

            // read triangle strip and add to vector
            let triangle_strip = TriangleStrip::read(reader)?;
            vertex_count += triangle_strip.pos.len();
            triangle_strips.push(triangle_strip);
        }

        Ok(Self {
            triangle_strips,
            material,
        })
    }

    pub fn write_to_obj<T: Write>(
        &self,
        writer: &mut T,
        model_name: &str,
        material_prefix: &str,
        e_next: &mut usize,
    ) -> io::Result<()> {
        // use obj_exporter::*;

        // create elements for triangles
        // this converts from triangle strips to triangles
        let mut elements = Vec::<Vec<usize>>::new();

        const CALC_WINDING_ORDER: bool = true;

        for s in self.triangle_strips.iter() {
            if CALC_WINDING_ORDER {
                // for each (iterator, triangle) in the triangle strip
                for (e_local, (a, b, c)) in
                    s.pos.iter().zip(s.norm.iter()).tuple_windows().enumerate()
                {
                    // calculate the average normal (doesn't need to be normalised)
                    let mean_norm = a.1.to() + b.1.to() + c.1.to();

                    // calculate the cross product of the vertex positions
                    let ab = b.0 - a.0;
                    let ac = c.0 - a.0;
                    let cross = ab.cross(&ac);

                    // check if cross and average normal are in the same direction
                    let dot = cross.dot(&mean_norm);

                    // determine winding order based on sign of dot product
                    let flip = dot > 0.;

                    // add triangle to element buffer in correct order
                    elements.push(if flip {
                        vec![
                            *e_next + e_local,
                            *e_next + e_local + 1,
                            *e_next + e_local + 2,
                        ]
                    } else {
                        vec![
                            *e_next + e_local + 2,
                            *e_next + e_local + 1,
                            *e_next + e_local,
                        ]
                    });
                }
            } else {
                // for each triangle iterator in the triangle strip
                for e_local in 0..s.pos.len() - 2 {
                    // do not flip flop winding order
                    elements.push(if e_local % 2 == 1 {
                        vec![
                            *e_next + e_local,
                            *e_next + e_local + 1,
                            *e_next + e_local + 2,
                        ]
                    } else {
                        vec![
                            *e_next + e_local + 2,
                            *e_next + e_local + 1,
                            *e_next + e_local,
                        ]
                    });
                }
            }
            *e_next += s.pos.len();
        }

        writeln!(writer, "o {}", model_name)?;
        writeln!(writer, "usemtl {}-{}", material_prefix, self.material)?;

        for ts in self.triangle_strips.iter() {
            for pos in &ts.pos {
                writeln!(writer, "v {} {} {}", pos.x, pos.y, pos.z)?;
            }
        }
        for ts in self.triangle_strips.iter() {
            for norm in &ts.norm {
                writeln!(
                    writer,
                    "vn {} {} {}",
                    norm.x as f32 / 255.,
                    norm.y as f32 / 255.,
                    norm.z as f32 / 255.
                )?;
            }
        }
        for ts in self.triangle_strips.iter() {
            for uv in &ts.uv {
                writeln!(writer, "vt {} {}", uv.x, uv.y)?;
            }
        }
        for tri in elements.iter() {
            writeln!(
                writer,
                "f {}/{}/{} {}/{}/{} {}/{}/{}",
                tri[0] + 1,
                tri[0] + 1,
                tri[0] + 1,
                tri[1] + 1,
                tri[1] + 1,
                tri[1] + 1,
                tri[2] + 1,
                tri[2] + 1,
                tri[2] + 1
            )?;
        }

        Ok(())
    }

    pub fn write_to_bin<T: Write + Seek>(&self, writer: &mut T) -> io::Result<()> {
        //! THIS WAS A TEST!
        //! THIS DOES NOT WORK

        // write model signature
        writer.write_u32::<LE>(0x00001000)?;

        // write unknown bytes
        writer.write_u32::<LE>(0)?;

        // write expected vertex count
        let mut expected_vertex_count = 0;
        for ts in self.triangle_strips.iter() {
            expected_vertex_count += ts.pos.len();
        }
        writer.write_u32::<LE>(expected_vertex_count as u32)?;

        // padding?
        writer.write_u32::<LE>(0x00000000)?;

        // write triangle strips
        for strip in &self.triangle_strips {
            // write triangle strip signature
            let index_count = 0x03; // should be 3 please keep it simple
            let signature = [
                0x00,
                0x80,
                0x04,
                0x60,
                index_count,
                0x00,
                0x00,
                0x00,
                0x41,
                0x00,
                0x00,
                0x00,
                0x00,
                0x40,
                0x1E,
                0x30,
                0x00,
                0xC0,
                0x1E,
                0x30,
                0x05,
                0x01,
                0x00,
                0x01,
                0x00,
                0x00,
                0x00,
                0x20,
                0x40,
                0x40,
                0x40,
                0x40,
            ];
            writer.write(&signature)?;

            // write triangle strip
            strip.write(writer)?;

            // write end 1
            writer.write_u32::<LE>(0x01000404)?;

            // write padding until 0x10 aligned
            while writer.seek(SeekFrom::Current(0))? % 0x10 != 0 {
                writer.write_u32::<LE>(0x00000000)?;
            }

            // write end 2
            let end = [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04,
                0x00, 0x00, 0x14,
            ];
            writer.write(&end)?;
        }

        Ok(())
    }
}
