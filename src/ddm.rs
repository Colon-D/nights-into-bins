use std::{
    fs::File,
    io::{self, Read, Write},
    path::Path,
};

pub struct DDM(pub Vec<DDS>);

impl DDM {
    pub fn read(path: &Path) -> io::Result<Self> {
        let mut reader = File::open(path)?;
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        // - split the data into DDS files
        let separator: &[u8] = &[0x44, 0x44, 0x53, 0x20, 0x7C];

        let mut slices = Vec::new();
        if let Some(start) = data
            .windows(separator.len())
            .position(|window| window == separator)
        {
            let mut start = start + separator.len();
            while let Some(end) = data[start..]
                .windows(separator.len())
                .position(|window| window == separator)
            {
                slices.push(DDS(data[start - separator.len()..start + end].to_vec()));
                start += end + separator.len();
            }
            slices.push(DDS(data[start - separator.len()..].to_vec()));
        }

        Ok(Self(slices))
    }
}

pub struct DDS(pub Vec<u8>);

impl DDS {
    pub fn write(&self, path: &Path, index: usize) -> io::Result<()> {
        // create dir if it does not exist
        let stem = path.file_stem().unwrap().to_str().unwrap();
        let dir_path = format!("out/{}", stem);
        let dir_path = Path::new(&dir_path);
        if !dir_path.exists() {
            std::fs::create_dir_all(dir_path)?;
        }

        let filename = format!("out/{}/{}-{}.dds", stem, stem, index);
        let mut file = File::create(filename)?;
        file.write_all(&self.0)?;
        Ok(())
    }
}
