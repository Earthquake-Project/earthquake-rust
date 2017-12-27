extern crate byteorder;

// Chunk

pub struct Chunk {
    pub fourcc: String,
    pub variant: ChunkVariant,
}

// ChunkVariant

pub enum ChunkVariant {
    Meta(Meta),
    InitialMap(InitialMap),
    MemoryMap(MemoryMap),
    Unimplemented,
}

// Meta

#[derive(Debug)]
pub struct Meta {
    pub codec: String,
}

// InitialMap

#[derive(Debug)]
pub struct InitialMap {
    pub entry_count: u32,
    pub entries: Vec<u32>,
}

// MemoryMap

#[derive(Debug)]
pub struct MemoryMap {
    pub unknown0: u16,
    pub unknown1: u16,
    pub chunk_count_max: u32,
    pub chunk_count_used: u32,
    pub junk_pointer: i32,
    pub unknown2: i32,
    pub free_pointer: i32,
    pub entries: Vec<MemoryMapEntry>,
}

#[derive(Debug)]
pub struct MemoryMapEntry {
    pub fourcc: String,
    pub length: u32,
    pub offset: u32,
    pub padding: i16,
    pub unknown0: i16,
    pub link: i32,
}
