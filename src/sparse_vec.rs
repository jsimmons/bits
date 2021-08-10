use std::{ptr::NonNull, slice::SliceIndex};

pub struct SparseVec<T> {
    len: usize,
    cap: usize,
    pop: NonNull<u64>,
    ptr: NonNull<T>,
}

#[inline]
fn has_entry_at_index(pop: &[u64], index: usize) -> bool {
    if let Some(&word) = pop.get(index / 64) {
        let mask = 1 << (index % 64);
        word & mask != 0
    } else {
        false
    }
}

fn find_empty_slot() -> usize {
    todo!()
}

impl<T> SparseVec<T> {
    pub fn new() -> Self {
        Self {
            len: 0,
            cap: 0,
            pop: NonNull::dangling(),
            ptr: NonNull::dangling(),
        }
    }

    pub fn insert(&mut self, value: T) -> usize {
        todo!()
    }

    pub fn remove<I: SliceIndex<T>>(&mut self, index: I) -> T {
        todo!()
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }

        todo!()
    }

    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len {
            return None;
        }

        todo!()
    }
}
