use std::{
    alloc::{self, Layout},
    ptr::NonNull,
};

pub struct RingVec<T> {
    head: u32,
    tail: u32,
    mask: u32,
    cap: u32,
    ptr: NonNull<T>,
}

impl<T> RingVec<T> {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity.is_power_of_two());
        assert!(capacity <= u32::MAX as usize);
        let layout = Layout::array::<T>(capacity).expect("capacity overflow");
        let ptr = unsafe { std::mem::transmute::<*mut u8, *mut T>(alloc::alloc(layout)) };
        if ptr.is_null() {
            alloc::handle_alloc_error(layout);
        }
        let ptr = unsafe { NonNull::new_unchecked(ptr) };

        Self {
            head: 0,
            tail: 0,
            mask: (capacity - 1) as u32,
            cap: capacity as u32,
            ptr,
        }
    }

    pub fn is_full(&self) -> bool {
        self.head - self.tail == self.cap
    }

    pub fn is_empty(&self) -> bool {
        self.head == self.tail
    }

    pub fn push(&mut self, value: T) {
        assert!(!self.is_full());
        unsafe {
            let offset = (self.head & self.mask) as isize;
            std::ptr::write(self.ptr.as_ptr().offset(offset), value);
            self.head = self.head.wrapping_add(1);
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            unsafe {
                let offset = (self.tail & self.mask) as isize;
                let value = std::ptr::read(self.ptr.as_ptr().offset(offset));
                self.tail = self.tail.wrapping_add(1);
                Some(value)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut ring_vec = RingVec::new(4);
        ring_vec.push(1);
        ring_vec.push(2);
        ring_vec.push(3);
        ring_vec.push(4);
        assert!(ring_vec.is_full());
        assert_eq!(ring_vec.pop(), Some(1));
        assert_eq!(ring_vec.pop(), Some(2));
        assert_eq!(ring_vec.pop(), Some(3));
        assert_eq!(ring_vec.pop(), Some(4));
        assert!(ring_vec.is_empty());
    }
}
