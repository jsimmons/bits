use crate::ring_buf::RingBuf;

pub struct RawTable<const N: usize> {
    storage: Box<[u32]>,
    free_ids: RingBuf<u32, N>,
}

impl<const N: usize> RawTable<N> {
    const INDEX_MASK: u32 = (N - 1) as u32;
    const GENERATION_MASK: u32 = !Self::INDEX_MASK;

    #[inline(always)]
    fn pack(value: u32, generation: u32) -> u32 {
        debug_assert!(generation & Self::INDEX_MASK == 0);
        debug_assert!(value & Self::GENERATION_MASK == 0);
        // Invert the value so that all bits zero is the invalid handle.
        !(value | generation)
    }

    #[inline(always)]
    fn unpack_value(handle: u32) -> u32 {
        (!handle) & Self::INDEX_MASK
    }

    #[inline(always)]
    fn unpack_generation(handle: u32) -> u32 {
        (!handle) & Self::GENERATION_MASK
    }

    pub fn new() -> Self {
        // We want to reserve the largest value (all index bits set) for an invalid handle, so
        // reduce actual capacity by one.
        assert!(N.is_power_of_two(), "N must be a power of two");
        assert!(
            N < (u32::MAX / 2) as usize,
            "N must be less than `u32::MAX / 2`"
        );
        let len = (N - 1) as u32;
        let storage = (0..len).map(|_| 0).collect::<Box<[_]>>();
        let free_ids = (0..len).collect::<RingBuf<_, N>>();
        Self { storage, free_ids }
    }

    pub fn allocate_handle(&mut self) -> u32 {
        let index = self.free_ids.pop_front().expect("id pool exhausted");
        let store = *self
            .storage
            .get(index as usize)
            .expect("invalid entry in id free list");
        Self::pack(index, Self::unpack_generation(store))
    }

    pub fn release_handle(&mut self, handle: u32) {
        let index = Self::unpack_value(handle);
        self.free_ids.push_back(index)
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
            *store = store.wrapping_add(N as u32)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TABLE_SIZE: usize = 65536;

    #[test]
    fn allocate_deallocate() {
        let mut table = RawTable::<TABLE_SIZE>::new();
        let handles = (0..(TABLE_SIZE - 1))
            .map(|_| table.allocate_handle())
            .collect::<Vec<_>>();
        for handle in handles {
            table.invalidate(handle);
            table.release_handle(handle);
        }
    }

    #[test]
    fn default_invalid() {
        let mut table = RawTable::<TABLE_SIZE>::new();
        let valid_handle = table.allocate_handle();
        assert_eq!(table.get(0), None);
        assert!(table.get(valid_handle).is_some());
        table.invalidate(valid_handle);
        assert!(table.get(valid_handle).is_none());
    }
}
