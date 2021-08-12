use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::{blit::Blit, world};

struct Part {
    code: [u8; 4],
    version: u32,
    mask: u128,
    align: usize,
    width: usize,
}

struct Blob {
    code: [u8; 4],
    version: u32,
    align: usize,
    width: usize,
}

#[derive(Default)]
pub struct Registry {
    part_map: HashMap<TypeId, usize>,
    parts: Vec<Part>,

    blob_map: HashMap<TypeId, usize>,
    blobs: Vec<Blob>,
}

impl Registry {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn register_part<T: Blit + Any>(&mut self) {
        let next_index = self.parts.len();
        assert!(next_index < world::MAX_PART_TYPES);
        self.part_map.insert(TypeId::of::<T>(), next_index);
        self.parts.push(Part {
            code: [0; 4],
            version: 0,
            mask: 1 << next_index,
            align: std::mem::align_of::<T>(),
            width: std::mem::size_of::<T>(),
        })
    }

    pub fn register_type<T: Blit + Any>(&mut self) {}
}
