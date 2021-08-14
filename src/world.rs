use crate::{raw_table::RawTable, registry::Registry, ring_buf::RingBuf, virtual_vec::VirtualVec};

pub const MAX_PART_TYPES: usize = 256;

const MAX_ARCHTYPES: usize = 1 << 14;
const MAX_CHUNKS: usize = 1 << 17;
const MAX_THINGS: usize = 1 << 20;

const TABLE_CACHE_MIN_SIZE: usize = 128;
const TABLE_CACHE_FILL_SIZE: usize = 256;
const TABLE_CACHE_SIZE: usize = 512;

const CHUNK_SIZE_BYTES: usize = 16 * 1024;

#[derive(Clone)]
pub struct PartBitmap {
    parts: [u64; MAX_PART_TYPES / 64],
}

#[derive(Clone)]
pub struct ArchtypeKey {
    scalar_parts: PartBitmap,
    vector_parts: PartBitmap,
}

pub struct ArchtypeId(u32);

struct Archtype {
    key: ArchtypeKey,
    chunks: Vec<ChunkId>,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ThingId(u32);

struct Thing {
    archtype: ArchtypeId,
    chunk: ChunkId,
}

struct ChunkId(u32);

struct Chunk {
    len: u32,
    data: [u8; CHUNK_SIZE_BYTES],
}

pub struct Query {}

pub struct World<'registry> {
    registry: &'registry Registry,
    thing_cache: RingBuf<ThingId, TABLE_CACHE_SIZE>,
    thing_table: RawTable<MAX_THINGS>,
    things: VirtualVec<Thing>,
    archtype_cache: RingBuf<ArchtypeId, TABLE_CACHE_SIZE>,
    archtype_table: RawTable<MAX_ARCHTYPES>,
    archtypes: VirtualVec<Archtype>,
    chunk_cache: RingBuf<ChunkId, TABLE_CACHE_SIZE>,
    chunk_table: RawTable<MAX_CHUNKS>,
    chunks: VirtualVec<Chunk>,
}

impl<'registry> World<'registry> {
    pub fn new(registry: &'registry Registry) -> Self {
        Self {
            registry,
            thing_cache: RingBuf::new(),
            thing_table: RawTable::new(),
            things: VirtualVec::new(MAX_THINGS),
            archtype_cache: RingBuf::new(),
            archtype_table: RawTable::new(),
            archtypes: VirtualVec::new(MAX_ARCHTYPES),
            chunk_cache: RingBuf::new(),
            chunk_table: RawTable::new(),
            chunks: VirtualVec::new(MAX_CHUNKS),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_destroy() {
        let registry = Registry::new();
        let world = World::new(&registry);
    }
}
