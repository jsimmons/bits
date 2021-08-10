use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::{blit::Blit, part::Part, world};

struct PartInfo {
    index: usize,
}

pub struct Registry {
    next_index: usize,
    map: HashMap<TypeId, usize>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            next_index: 0,
            map: HashMap::new(),
        }
    }

    pub fn register_part<T: Part + Blit + Any>(&mut self) {
        let next_index = self.next_index;
        assert!(next_index < world::MAX_PART_TYPES);
        self.next_index += 1;
        self.map.insert(TypeId::of::<T>(), next_index);
    }
}
