extern crate byteorder;

use byteorder::ReadBytesExt;
use io::value::*;
use chunk::*;

use std::{self, fmt};
use std::io::{Cursor, SeekFrom};
use std::io::prelude::*;

// ReadChunks

pub trait ReadChunks {
    fn read_chunk<B: ByteOrder>(&mut self, fourcc: String, length: Option<u32>) -> Result<Chunk, ReadChunkError>;
    fn read_rifx<B: ByteOrder>(&mut self) -> Result<ChunkVariant, ReadChunkError>;
    fn read_imap<B: ByteOrder>(&mut self) -> Result<ChunkVariant, ReadChunkError>;
    fn read_mmap<B: ByteOrder>(&mut self) -> Result<ChunkVariant, ReadChunkError>;
}

impl<'a> ReadChunks for Cursor<&'a [u8]> {
    fn read_chunk<B: ByteOrder>(&mut self, expected_fourcc: String, expected_length: Option<u32>) -> Result<Chunk, ReadChunkError> {
        let offset = self.position();
        let fourcc = self.read_fourcc::<B>()?;
        let mut length = self.read_u32::<B>()?;

        // Validate chunk
        if let Some(n) = expected_length {
            if n != length || expected_fourcc != fourcc {
                return Err(ReadChunkError::UnexpectedChunk(
                    UnexpectedChunkError::KnownLength(offset, expected_fourcc, n, fourcc, length)
                ));
            }
        } else {
            if expected_fourcc != fourcc {
                return Err(ReadChunkError::UnexpectedChunk(
                    UnexpectedChunkError::UnknownLength(offset, expected_fourcc, fourcc, length)
                ));
            }
        }

        if fourcc == "RIFX" {
            // Pretend RIFX is only 12 bytes long
            // This is because offsets are relative to the beginning of the file
            // Whereas everywhere else they're relative to chunk start
            length = 4;
        }

        let data_offset = self.position() as usize;
        let slice = &self.get_ref()[data_offset..data_offset+(length as usize)];
        self.seek(SeekFrom::Current(length as i64))?;
        let mut rdr = Cursor::new(slice.clone());

        let variant = match fourcc.as_ref() {
            "RIFX" => rdr.read_rifx::<B>()?,
            "imap" => rdr.read_imap::<B>()?,
            "mmap" => rdr.read_mmap::<B>()?,
            _ => ChunkVariant::Unimplemented,
        };

        Ok(Chunk {
            fourcc: fourcc,
            variant: variant,
        })
    }

    fn read_rifx<B: ByteOrder>(&mut self) -> Result<ChunkVariant, ReadChunkError> {
        let codec = self.read_fourcc::<B>()?;
        Ok(ChunkVariant::Meta(Meta {
            codec: codec,
        }))
    }

    fn read_imap<B: ByteOrder>(&mut self) -> Result<ChunkVariant, ReadChunkError> {
        let entry_count = self.read_u32::<B>()?;
        let mut entries = vec![0u32; entry_count as usize];
        for i in 0..(entry_count as usize) {
            entries[i] = self.read_u32::<B>()?;
        }
        Ok(ChunkVariant::InitialMap(InitialMap {
            entry_count: entry_count,
            entries: entries,
        }))
    }

    fn read_mmap<B: ByteOrder>(&mut self) -> Result<ChunkVariant, ReadChunkError> {
        let unknown0 = self.read_u16::<B>()?;
        let unknown1 = self.read_u16::<B>()?;
        let chunk_count_max = self.read_u32::<B>()?;
        let chunk_count_used = self.read_u32::<B>()?;
        let junk_pointer = self.read_i32::<B>()?;
        let unknown2 = self.read_i32::<B>()?;
        let free_pointer = self.read_i32::<B>()?;
        let mut entries = Vec::new();
        for _i in 0..(chunk_count_used as usize) {
            let fourcc = self.read_fourcc::<B>()?;
            let length = self.read_u32::<B>()?;
            let offset = self.read_u32::<B>()?;
            let padding = self.read_i16::<B>()?;
            let unknown0 = self.read_i16::<B>()?;
            let link = self.read_i32::<B>()?;
            let entry = MemoryMapEntry {
                fourcc: fourcc,
                length: length,
                offset: offset,
                padding: padding,
                unknown0: unknown0,
                link: link,
            };
            entries.push(entry);
        }
        Ok(ChunkVariant::MemoryMap(MemoryMap {
            unknown0: unknown0,
            unknown1: unknown1,
            chunk_count_max: chunk_count_max,
            chunk_count_used: chunk_count_used,
            junk_pointer: junk_pointer,
            unknown2: unknown2,
            free_pointer: free_pointer,
            entries: entries,
        }))
    }
}

// UnexpectedChunkError

pub enum UnexpectedChunkError {
    KnownLength(u64, String, u32, String, u32),
    UnknownLength(u64, String, String, u32)
}

impl fmt::Debug for UnexpectedChunkError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UnexpectedChunkError::KnownLength(offset, ref expected_fourcc, expected_length, ref fourcc, length) => {
                write!(
                    f, "at offset {} expected a {} chunk with length {}, but got a {} chunk with length {}",
                    offset, expected_fourcc, expected_length, fourcc, length
                )
            },
            UnexpectedChunkError::UnknownLength(offset, ref expected_fourcc, ref fourcc, length) => {
                write!(
                    f, "at offset {} expected a {} chunk of unknown length, but got a {} chunk with length {}",
                    offset, expected_fourcc, fourcc, length
                )
            },
        }
    }
}

// ReadChunkError

pub enum ReadChunkError {
    IOError(std::io::Error),
    ReadStringError(ReadStringError),
    UnexpectedChunk(UnexpectedChunkError),
}

impl From<std::io::Error> for ReadChunkError {
    fn from(e: std::io::Error) -> ReadChunkError {
        ReadChunkError::IOError(e)
    }
}

impl From<ReadStringError> for ReadChunkError {
    fn from(e: ReadStringError) -> ReadChunkError {
        ReadChunkError::ReadStringError(e)
    }
}

impl From<UnexpectedChunkError> for ReadChunkError {
    fn from(e: UnexpectedChunkError) -> ReadChunkError {
        ReadChunkError::UnexpectedChunk(e)
    }
}

impl fmt::Debug for ReadChunkError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ReadChunkError::IOError(ref e) => e.fmt(f),
            ReadChunkError::ReadStringError(ref e) => e.fmt(f),
            ReadChunkError::UnexpectedChunk(ref e) => e.fmt(f),
        }
    }
}
