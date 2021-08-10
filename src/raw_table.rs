use std::collections::VecDeque;

pub struct RawTable<const INDEX_BITS: usize> {
    storage: Box<[u32]>,
    free_ids: VecDeque<u32>,
}

impl<const INDEX_BITS: usize> RawTable<INDEX_BITS> {
    const INDEX_MASK: u32 = (1 << INDEX_BITS) - 1;
    const GENERATION_MASK: u32 = !Self::INDEX_MASK;
    const GENERATION_INCREMENT: u32 = 1 << INDEX_BITS;

    #[inline]
    fn pack(value: u32, generation: u32) -> u32 {
        debug_assert!(generation & Self::INDEX_MASK == 0);
        debug_assert!(value & Self::GENERATION_MASK == 0);
        // Invert the value so that all bits zero is the invalid handle.
        !(value | generation)
    }

    #[inline]
    fn unpack_value(handle: u32) -> u32 {
        (!handle) & Self::INDEX_MASK
    }

    #[inline]
    fn unpack_generation(handle: u32) -> u32 {
        (!handle) & Self::GENERATION_MASK
    }

    pub fn new() -> Self {
        // We want to reserve the largest value (all index bits set) for an invalid handle.
        let len = (1 << INDEX_BITS) - 1;
        let storage = (0..len).map(|_| 0).collect::<Box<[_]>>();
        let free_ids = (0..len).collect::<VecDeque<u32>>();
        Self { storage, free_ids }
    }

    pub fn allocate_handle(&mut self) -> u32 {
        self.free_ids.pop_back().unwrap()
    }

    pub fn release_handle(&mut self, handle: u32) {
        self.free_ids.push_front(handle)
    }

    pub fn get(&self, handle: u32) -> Option<u32> {
        if let Some(&store) = self.storage.get(Self::unpack_value(handle) as usize) {
            if Self::unpack_generation(store) == Self::unpack_generation(handle) {
                return Some(Self::unpack_value(store));
            }
        }
        None
    }

    pub fn set(&mut self, handle: u32, new_value: u32) -> bool {
        let index = Self::unpack_value(handle) as usize;
        if let Some(store) = self.storage.get_mut(index) {
            let store_generation = Self::unpack_generation(*store);
            if store_generation == Self::unpack_generation(handle) {
                *store = Self::pack(new_value, store_generation);
                return true;
            }
        }
        false
    }

    pub fn invalidate(&mut self, handle: u32) {
        let index = Self::unpack_value(handle) as usize;
        if let Some(store) = self.storage.get_mut(index) {
            *store = store.wrapping_add(Self::GENERATION_INCREMENT)
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn exploration() {
        assert_eq!(2 + 2, 4);
    }
}
