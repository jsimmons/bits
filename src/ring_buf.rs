use std::{iter::FromIterator, mem::MaybeUninit};

use crate::helpers;

pub struct RingBuf<T, const N: usize> {
    head: u32,
    tail: u32,
    data: Box<[MaybeUninit<T>; N]>,
}

#[cold]
#[inline(never)]
fn capacity_assert_failed() {
    panic!("RingBuf is full")
}

impl<T, const N: usize> RingBuf<T, N> {
    const MASK: u32 = (N - 1) as u32;

    pub fn new() -> Self {
        assert!(N.is_power_of_two());
        assert!(N <= (1 << 31));
        assert!(N > 0);
        Self {
            head: 0,
            tail: 0,
            data: helpers::box_uninit_array(), //Box::new(helpers::uninit_array()),
        }
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.head.wrapping_sub(self.tail) == (N as u32)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.head == self.tail
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        N
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.head.wrapping_sub(self.tail) as usize
    }

    #[inline]
    unsafe fn buffer_as_slice(&self) -> &[T] {
        std::mem::transmute::<&[MaybeUninit<_>; N], &[_; N]>(&*self.data)
    }

    #[inline]
    unsafe fn buffer_as_mut_slice(&mut self) -> &mut [T] {
        std::mem::transmute::<&mut [MaybeUninit<_>; N], &mut [_; N]>(&mut *self.data)
    }

    #[inline]
    pub fn push_back(&mut self, value: T) {
        if self.is_full() {
            capacity_assert_failed()
        }
        unsafe {
            self.data
                .get_unchecked_mut((self.head & Self::MASK) as usize)
                .as_mut_ptr()
                .write(value);
        }
        self.head = self.head.wrapping_add(1);
    }

    #[inline]
    pub fn pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            unsafe {
                let value = std::ptr::read(
                    self.data
                        .get_unchecked((self.tail & Self::MASK) as usize)
                        .as_ptr(),
                );
                self.tail = self.tail.wrapping_add(1);
                Some(value)
            }
        }
    }

    pub fn iter(&self) -> Iter<T> {
        Iter::<T> {
            head: self.head,
            tail: self.tail,
            mask: Self::MASK,
            ring: unsafe { self.buffer_as_slice() },
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut::<T> {
            head: self.head,
            tail: self.tail,
            mask: Self::MASK,
            ring: unsafe { self.buffer_as_mut_slice() },
        }
    }

    #[inline]
    pub fn as_slices(&self) -> (&[T], &[T]) {
        unsafe {
            let buf = self.buffer_as_slice();
            let head = (self.head & Self::MASK) as usize;
            let tail = (self.tail & Self::MASK) as usize;
            RingSlices::ring_slices(buf, head, tail)
        }
    }

    #[inline]
    pub fn as_mut_slices(&mut self) -> (&mut [T], &mut [T]) {
        unsafe {
            let head = (self.head & Self::MASK) as usize;
            let tail = (self.tail & Self::MASK) as usize;
            let buf = self.buffer_as_mut_slice();
            RingSlices::ring_slices(buf, head, tail)
        }
    }
}

impl<T, const N: usize> Drop for RingBuf<T, N> {
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
    mask: u32,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.head == self.tail {
            None
        } else {
            unsafe {
                let value = self.ring.get_unchecked((self.tail & self.mask) as usize);
                self.tail = self.tail.wrapping_add(1);
                Some(value)
            }
        }
    }
}

pub struct IterMut<'a, T: 'a> {
    ring: &'a mut [T],
    head: u32,
    tail: u32,
    mask: u32,
}

impl<A, const N: usize> Extend<A> for RingBuf<A, N> {
    fn extend<T: IntoIterator<Item = A>>(&mut self, iter: T) {
        for value in iter.into_iter() {
            self.push_back(value)
        }
    }
}

impl<'a, T: 'a + Copy, const N: usize> Extend<&'a T> for RingBuf<T, N> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.extend(iter.into_iter().cloned());
    }
}

impl<A, const N: usize> FromIterator<A> for RingBuf<A, N> {
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> RingBuf<A, N> {
        let iterator = iter.into_iter();
        let mut deq = RingBuf::new();
        deq.extend(iterator);
        deq
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn large_table() {
        let mut ring_buf = RingBuf::<u32, 2147483648>::new();
        for i in 0..2147483648 {
            ring_buf.push_back(i);
        }
        assert!(ring_buf.is_full());
    }

    #[test]
    fn small_table() {
        let mut ring_buf = RingBuf::<u32, 1>::new();
        ring_buf.push_back(0);
        assert!(ring_buf.is_full());
    }

    #[test]
    fn push_pop_wrap() {
        let mut ring_buf = RingBuf::<_, 256>::new();
        // Adjust the zero position so we wrap half way through a push sequence.
        ring_buf.head = 0xffff_ffff - 100;
        ring_buf.tail = 0xffff_ffff - 100;
        for _ in 0..2 {
            for i in 0..200 {
                ring_buf.push_back(i);
                assert_eq!(ring_buf.len(), (i + 1) as usize);
            }
            assert_eq!(ring_buf.len(), 200);
            for i in 0..200 {
                assert_eq!(ring_buf.pop_front(), Some(i));
                assert_eq!(ring_buf.len(), (199 - i) as usize);
            }
        }
        assert_eq!(ring_buf.len(), 0);
    }

    #[test]
    fn empty_full() {
        let mut ring_buf = RingBuf::<_, 2>::new();
        assert!(ring_buf.is_empty());
        assert!(!ring_buf.is_full());

        ring_buf.push_back(1);
        assert!(!ring_buf.is_full());
        assert!(!ring_buf.is_empty());

        ring_buf.push_back(2);
        assert!(!ring_buf.is_empty());
        assert!(ring_buf.is_full());

        assert_eq!(ring_buf.pop_front(), Some(1));
        assert!(!ring_buf.is_full());
        assert!(!ring_buf.is_empty());

        assert_eq!(ring_buf.pop_front(), Some(2));
        assert!(ring_buf.is_empty());
        assert!(!ring_buf.is_full());

        ring_buf.push_back(1);
        assert!(!ring_buf.is_full());
        assert!(!ring_buf.is_empty());

        ring_buf.push_back(2);
        assert!(ring_buf.is_full());
        assert!(!ring_buf.is_empty());

        assert_eq!(ring_buf.pop_front(), Some(1));
        assert!(!ring_buf.is_full());
        assert!(!ring_buf.is_empty());

        assert_eq!(ring_buf.pop_front(), Some(2));
        assert!(ring_buf.is_empty());
        assert!(!ring_buf.is_full());
    }

    #[test]
    fn iterator() {
        let mut ring_buf = RingBuf::<_, 128>::new();
        assert!(ring_buf.iter().next().is_none());

        {
            for i in 0..100 {
                ring_buf.push_back(i);
                assert_eq!(ring_buf.len(), i + 1);
                assert_eq!(ring_buf.iter().count(), i + 1);
            }

            {
                let mut i = 0;
                for x in ring_buf.iter() {
                    assert_eq!(i, *x);
                    i += 1
                }
            }

            for i in 0..100 {
                assert_eq!(ring_buf.pop_front(), Some(i));
            }
        }

        {
            for i in 0..100 {
                ring_buf.push_back(i);
                assert_eq!(ring_buf.len(), i + 1);
                assert_eq!(ring_buf.iter().count(), i + 1);
            }

            {
                let mut i = 0;
                for x in ring_buf.iter() {
                    assert_eq!(i, *x);
                    i += 1
                }
            }
        }
    }
}
