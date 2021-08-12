use std::{
    alloc::{self, Layout},
    ptr::NonNull,
};

pub struct RingBuf<T> {
    head: u32,
    tail: u32,
    mask: u32,
    cap: u32,
    ptr: NonNull<T>,
}

impl<T> RingBuf<T> {
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

    #[inline]
    pub fn is_full(&self) -> bool {
        self.head - self.tail == self.cap
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.head == self.tail
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.cap as usize
    }

    #[inline]
    pub fn len(&self) -> usize {
        (self.head - self.tail) as usize
    }

    /// Turn ptr into a slice
    #[inline]
    unsafe fn buffer_as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.capacity()) }
    }

    /// Turn ptr into a mut slice
    #[inline]
    unsafe fn buffer_as_mut_slice(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.capacity()) }
    }

    pub fn push_back(&mut self, value: T) {
        assert!(!self.is_full());
        unsafe {
            let offset = (self.head & self.mask) as isize;
            std::ptr::write(self.ptr.as_ptr().offset(offset), value);
            self.head = self.head.wrapping_add(1);
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
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

    pub fn iter(&self) -> Iter<T> {
        let head = self.head;
        let tail = self.tail;
        Iter::<T> {
            head,
            tail,
            ring: unsafe { self.buffer_as_slice() },
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        let head = self.head;
        let tail = self.tail;
        IterMut::<T> {
            head,
            tail,
            ring: unsafe { self.buffer_as_mut_slice() },
        }
    }

    #[inline]
    pub fn as_slices(&self) -> (&[T], &[T]) {
        unsafe {
            let buf = self.buffer_as_slice();
            let head = (self.head & self.mask) as usize;
            let tail = (self.tail & self.mask) as usize;
            RingSlices::ring_slices(buf, head, tail)
        }
    }

    #[inline]
    pub fn as_mut_slices(&mut self) -> (&mut [T], &mut [T]) {
        unsafe {
            let head = (self.head & self.mask) as usize;
            let tail = (self.tail & self.mask) as usize;
            let buf = self.buffer_as_mut_slice();
            RingSlices::ring_slices(buf, head, tail)
        }
    }
}

impl<T> Drop for RingBuf<T> {
    fn drop(&mut self) {
        /// Runs the destructor for all items in the slice when it gets dropped (normally or
        /// during unwinding).
        struct Dropper<'a, T>(&'a mut [T]);

        impl<'a, T> Drop for Dropper<'a, T> {
            fn drop(&mut self) {
                unsafe {
                    std::ptr::drop_in_place(self.0);
                }
            }
        }

        let (front, back) = self.as_mut_slices();
        unsafe {
            let _back_dropper = Dropper(back);
            // use drop for [T]
            std::ptr::drop_in_place(front);
        }

        let layout = Layout::array::<T>(self.cap as usize).expect("capacity overflow");
        unsafe {
            let ptr = std::mem::transmute::<*mut T, *mut u8>(self.ptr.as_ptr());
            alloc::dealloc(ptr, layout)
        }
    }
}

/// Returns the two slices that cover the `VecDeque`'s valid range
pub trait RingSlices: Sized {
    fn slice(self, from: usize, to: usize) -> Self;
    fn split_at(self, i: usize) -> (Self, Self);

    fn ring_slices(buf: Self, head: usize, tail: usize) -> (Self, Self) {
        let contiguous = tail <= head;
        if contiguous {
            let (empty, buf) = buf.split_at(0);
            (buf.slice(tail, head), empty)
        } else {
            let (mid, right) = buf.split_at(tail);
            let (left, _) = mid.split_at(head);
            (right, left)
        }
    }
}

impl<T> RingSlices for &[T] {
    fn slice(self, from: usize, to: usize) -> Self {
        &self[from..to]
    }
    fn split_at(self, i: usize) -> (Self, Self) {
        (*self).split_at(i)
    }
}

impl<T> RingSlices for &mut [T] {
    fn slice(self, from: usize, to: usize) -> Self {
        &mut self[from..to]
    }
    fn split_at(self, i: usize) -> (Self, Self) {
        (*self).split_at_mut(i)
    }
}

pub struct Iter<'a, T: 'a> {
    ring: &'a [T],
    head: u32,
    tail: u32,
}

pub struct IterMut<'a, T: 'a> {
    ring: &'a mut [T],
    head: u32,
    tail: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut ring_buf = RingBuf::new(4);
        ring_buf.push_back(1);
        ring_buf.push_back(2);
        ring_buf.push_back(3);
        ring_buf.push_back(4);
        assert!(ring_buf.is_full());
        assert_eq!(ring_buf.pop_front(), Some(1));
        assert_eq!(ring_buf.pop_front(), Some(2));
        assert_eq!(ring_buf.pop_front(), Some(3));
        assert_eq!(ring_buf.pop_front(), Some(4));
        assert!(ring_buf.is_empty());
    }

    #[test]
    fn push_pop() {
        let mut ring_buf = RingBuf::new(256);
        for _ in 0..10 {
            for i in 0..200 {
                ring_buf.push_back(i);
            }
            assert_eq!(ring_buf.len(), 200);
            for i in 0..200 {
                assert_eq!(ring_buf.pop_front(), Some(i));
            }
        }
        assert_eq!(ring_buf.len(), 0);
    }
}
