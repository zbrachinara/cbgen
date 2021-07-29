use std::convert::TryInto;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use nbt::{from_gzip_reader, from_reader, from_zlib_reader, Blob, Value};

use log::*;

type StdResult<U, V> = std::result::Result<U, V>;
type Result<T> = StdResult<T, Box<dyn std::error::Error>>;
const KILOBYTE: usize = 1024;
use CompressionMode::*;

#[derive(Debug)]
enum CompressionMode {
    GZip,
    ZLib,
    Raw,
}

impl Default for CompressionMode {
    fn default() -> Self {
        Self::Raw
    }
}

#[derive(Debug, Default)]
pub struct Chunk {
    offset: u32,
    size: u32,
    timestamp: u32,
    mode: CompressionMode,
    nbt: Blob,
}

#[derive(Debug)]
pub struct McaFile {
    file: File,
    chunks: [[Option<Chunk>; 32]; 32],
}

impl McaFile {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        // init chunk array
        let mut chunks = vec![];
        chunks.reserve_exact(32);
        for _ in 0..32 {
            let mut row = vec![];
            row.reserve_exact(32);
            for _ in 0..32 {
                row.push(Option::<Chunk>::None);
            }
            chunks.push(row.try_into().unwrap());
        }

        Result::Ok(Self {
            file: File::open(path)?,
            chunks: chunks.try_into().unwrap(),
        })
    }

    pub fn read_chunk(&mut self, posx: usize, posz: usize) -> Result<&Option<Chunk>> {
        if let None = &self.chunks[posx][posz] {
            // retry to see if there is a chunk in this location
            let mut buf: [u8; 4] = [0; 4];
            let loc_offset = (posx * 32 + posz) * 4;
            let timestamp_offset = loc_offset + 4 * KILOBYTE;

            self.file.read_exact(&mut buf)?;
            let dat_offset = u32::from_be_bytes([0, buf[0], buf[1], buf[2]]) * 4 * KILOBYTE as u32;

            self.file.seek(SeekFrom::Start(timestamp_offset as u64))?;
            self.file.read_exact(&mut buf)?;
            let timestamp = u32::from_be_bytes(buf);

            if let Some((nbt, mode, size)) = McaFile::read_chunk_at(&mut self.file, dat_offset)? {
                self.chunks[posx][posz] = Some(Chunk {
                    nbt,
                    mode,
                    size,
                    timestamp,
                    offset: dat_offset,
                })
            };
        }

        Result::Ok(&self.chunks[posx][posz])
    }

    fn read_chunk_at(file: &mut File, offset: u32) -> Result<Option<(Blob, CompressionMode, u32)>> {
        if offset == 0 {
            return Result::Ok(None);
        }

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
        let nbt: Blob = match mode {
            GZip => from_gzip_reader(data_section),
            ZLib => from_zlib_reader(data_section),
            Raw => from_reader(data_section),
        }?;

        Ok(Some((nbt, mode, size)))
    }
}
