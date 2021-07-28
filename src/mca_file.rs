use std::convert::TryInto;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use nbt::{from_gzip_reader, from_reader, from_zlib_reader, Blob, Value};

use log::*;

type StdResult<U, V> = std::result::Result<U, V>;
type Result<T> = StdResult<T, Box<dyn std::error::Error>>;
const KILOBYTE: u32 = 1024;
use CompressionMode::*;

#[derive(Debug)]
enum CompressionMode {
    GZip,
    ZLib,
    Raw,
}

#[derive(Debug)]
struct Chunk {
    offset: u32,
    size: u32,
    mode: CompressionMode,
    nbt: Blob,
}

#[derive(Debug)]
pub struct McaFile {
    file: File,
    // chunks: [[Option<Chunk>; 32]; 32],
}

impl McaFile {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut buf: [u8; 4] = [0; 4];
        let mut file = File::open(path)?;
        // init chunk array

        for x in 0..32 {
            for z in 0..32 {
                file.read_exact(&mut buf)?;

                // combine first three bytes into u32 while
                // interpreting as big endian
                let offset = u32::from_be_bytes([0, buf[0], buf[1], buf[2]]) * 4 * KILOBYTE;
                let size = buf[3];
                // debug!("Chunk ({}, {})   offset (bytes): {:x?}, sector count (4 KiB each): {}", x, z, offset, size);

                McaFile::read_chunk_at(&mut file, offset);
            }
        }

        Result::Ok(Self {
            file,
            // chunks: chunks.into(),
        })
    }

    fn read_chunk_at(file: &mut File, offset: u32) -> Result<Option<Chunk>> {
        if offset == 0 {
            return Result::Ok(None);
        }

        let start_offset = file.stream_position()?;
        file.seek(SeekFrom::Start(offset as u64))?;

        let mut buf: [u8; 5] = [0; 5];
        file.read_exact(&mut buf)?;

        let size = u32::from_be_bytes(buf[0..4].try_into().unwrap()); // should never fail, since arg expects array of 4 elements
        let mode = match buf[4] {
            1 => GZip,
            2 => ZLib,
            3 => Raw,
            _ => unreachable!(),
        };
        let data_section = file.take(size as u64);
        let nbt: Blob = match match mode {
            GZip => from_gzip_reader(data_section),
            ZLib => from_zlib_reader(data_section),
            Raw => from_reader(data_section),
        } {
            Err(x) => {
                debug!("hematite-nbt failed: {}", x);
                let pos = file.seek(SeekFrom::Start(start_offset))?;
                return Err(Box::new(x));
            }
            Ok(x) => x,
        };

        let pos = file.seek(SeekFrom::Start(start_offset))?;
        debug!("Returning to: {}", pos);

        Result::Ok(Option::Some(Chunk{
            offset,
            size,
            mode,
            nbt,
        }))

    }

}
