use chunk::*;

use byteorder::{BigEndian, LittleEndian};
use chunk::Chunk;
use io::value::*;
use io::chunk::*;

use std::collections::HashMap;
use std::fmt;
use std::io::Cursor;

// Movie

pub struct Movie {
    chunks: HashMap<u32, Chunk>,
}

impl Movie {
    pub fn read(buf: &[u8]) -> Result<Movie, ReadMovieError> {
        let mut rdr = Cursor::new(buf);
        let header = match rdr.read_fourcc::<BigEndian>() {
            Ok(v) => v,
            Err(_e) => return Err(ReadMovieError::InvalidHeader),
        };
        rdr.set_position(0);
        let chunks = match header.as_ref() {
            "RIFX" => read_chunks::<BigEndian>(&mut rdr)?,
            "XFIR" => read_chunks::<LittleEndian>(&mut rdr)?,
            _ => return Err(ReadMovieError::InvalidHeader),
        };
        Ok(Movie {
            chunks: chunks
        })
    }
}

fn read_chunks<B: ByteOrder>(mut rdr: &mut Cursor<&[u8]>) -> Result<HashMap<u32, Chunk>, ReadMovieError> {
    let mut chunks = HashMap::new();

    let mmap = lookup_mmap::<B>(&mut rdr)?;
    if let ChunkVariant::MemoryMap(ref mmap_struct) = mmap.variant {
        for i in 0..mmap_struct.chunk_count_used {
            // Note: RIFX, imap, and mmap are re-parsed for simplicity's sake
            let mmap_entry = &mmap_struct.entries[i as usize];
            rdr.set_position(mmap_entry.offset as u64);
            if mmap_entry.fourcc != "free" && mmap_entry.fourcc != "junk" {
                let chunk = rdr.read_chunk::<B>(mmap_entry.fourcc.clone(), Some(mmap_entry.length))?;
                chunks.insert(i, chunk);
            }
        }
    }

    Ok(chunks)
}

fn lookup_mmap<B: ByteOrder>(rdr: &mut Cursor<&[u8]>) -> Result<Chunk, ReadMovieError> {
    let meta = rdr.read_chunk::<B>("RIFX".to_string(), None)?;
    if let ChunkVariant::Meta(ref meta_struct) = meta.variant {
        if meta_struct.codec != "MV93" {
            return Err(ReadMovieError::UnsupportedCodec(meta_struct.codec.clone()));
        }
    }

    let imap = rdr.read_chunk::<B>("imap".to_string(), None)?;
    let mmap_offset = if let ChunkVariant::InitialMap(ref imap_struct) = imap.variant {
        imap_struct.entries[0]
    } else {unreachable!()};

    rdr.set_position(mmap_offset as u64);
    let mmap = rdr.read_chunk::<B>("mmap".to_string(), None)?;

    Ok(mmap)
}

// ReadMovieError

pub enum ReadMovieError {
    InvalidHeader,
    UnsupportedCodec(String),
    ReadChunkError(ReadChunkError),
}

impl From<ReadChunkError> for ReadMovieError {
    fn from(e: ReadChunkError) -> ReadMovieError {
        ReadMovieError::ReadChunkError(e)
    }
}

impl fmt::Debug for ReadMovieError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ReadMovieError::InvalidHeader => write!(f, "invalid header"),
            ReadMovieError::UnsupportedCodec(ref codec) => write!(f, "unsupported codec: {}", codec),
            ReadMovieError::ReadChunkError(ref e) => e.fmt(f),
        }
    }
}

