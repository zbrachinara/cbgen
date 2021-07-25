use std::io::Read;
use std::fs::File;
use std::path::Path;

use log::*;

#[derive(Debug)]
pub struct McaFile {
    file: File,
}

impl McaFile {

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {

        let mut buf: [u8; 4] = [0; 4];
        let mut file = File::open(path)?;

        for x in 0..32 {
            for z in 0..32 {

                file.read_exact(&mut buf)?;

                // combine first three bytes into u32 while
                // interpreting as big endian

                let offset = u32::from_be_bytes([0, buf[0], buf[1], buf[2]]) * 4;
                let sector_count = buf[3] * 4;
                debug!("Chunk ({}, {})   offset (KiB): {}, sector count (KiB): {}", x, z, offset, sector_count);

            }
        }

        Result::Ok(
            Self {
                file,
            }
        )

    }

}
