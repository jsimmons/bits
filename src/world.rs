use crate::{raw_table::RawTable, registry::Registry, virtual_vec::VirtualVec};

const MAX_ARCHTYPES_LOG2: usize = 14;
const MAX_THINGS_LOG2: usize = 20;
const MAX_CHUNKS_LOG2: usize = 17;

const CHUNK_BYTES: usize = 32 * 1024;
pub const MAX_PART_TYPES: usize = 128;

pub struct ArchtypeKey {
    scalar_parts: u128,
    vector_parts: u128,
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
    data: [u8; CHUNK_BYTES],
}

pub struct Query {}

pub struct World<'registry> {
    registry: &'registry Registry,
    thing_lookup: RawTable<MAX_THINGS_LOG2>,
    things: VirtualVec<Thing>,
    archtype_lookup: RawTable<MAX_ARCHTYPES_LOG2>,
    archtypes: VirtualVec<Archtype>,
    chunk_lookup: RawTable<MAX_CHUNKS_LOG2>,
    chunks: VirtualVec<Chunk>,
}

impl<'registry> World<'registry> {
    pub fn new(registry: &'registry Registry) -> Self {
        Self {
            registry,
            thing_lookup: RawTable::new(),
            things: VirtualVec::new(1 << MAX_THINGS_LOG2),
            archtype_lookup: RawTable::new(),
            archtypes: VirtualVec::new(1 << MAX_ARCHTYPES_LOG2),
            chunk_lookup: RawTable::new(),
            chunks: VirtualVec::new(1 << MAX_CHUNKS_LOG2),
        }
    }
}
