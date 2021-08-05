use std::{
    alloc::Layout,
    marker::PhantomData,
    mem::{align_of, size_of},
    ptr::{null_mut, NonNull},
};

use libc::{
    c_void, mmap, mprotect, munmap, MAP_ANONYMOUS, MAP_PRIVATE, PROT_NONE, PROT_READ, PROT_WRITE,
};

const PAGE_SIZE: usize = 4096;
const MAX_ELEMENTS: usize = usize::MAX / 2;

pub struct VirtualVec<T> {
    map: usize,
    cap: usize,
    len: usize,
    ptr: NonNull<T>,
    _marker: PhantomData<T>,
}

#[cold]
#[inline(never)]
fn bounds_check_failed(index: usize, len: usize) {
    panic!("index `{}` beyond VirtualVec length `{}`", index, len);
}

#[cold]
#[inline(never)]
fn capacity_overflow(len: usize) {
    panic!("capacity of `{}` too large for VirtualVec", len);
}

#[cold]
#[inline(never)]
fn mapping_reserve(layout: Layout) -> *mut c_void {
    unsafe {
        mmap(
            null_mut(),
            layout.size(),
            PROT_NONE,
            MAP_PRIVATE | MAP_ANONYMOUS,
            0,
            0,
        )
    }
}

#[cold]
#[inline(never)]
fn mapping_commit(addr: *mut c_void, len: usize) {
    unsafe {
        mprotect(addr, len, PROT_READ | PROT_WRITE);
    }
}

impl<T> VirtualVec<T> {
    #[cold]
    pub fn new(map: usize) -> Self {
        if size_of::<T>() == 0 {
            panic!("cannot create a VirtualVec containing a ZST")
        }
        if align_of::<T>() > PAGE_SIZE {
            panic!("alignment for type too large")
        }
        if map > MAX_ELEMENTS {
            panic!("too many elements in map")
        }
        if map == 0 {
            panic!("cannot create a VirtualVec without any backing storage")
        }

        let layout = Layout::array::<T>(map).expect("mapping too large");
        let ptr = unsafe {
            let mapping = mapping_reserve(layout);
            let ptr = std::mem::transmute::<*mut c_void, *mut T>(mapping);
            NonNull::new(ptr).expect("could not create memory mapping")
        };

        Self {
            map,
            cap: 0,
            len: 0,
            ptr,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.cap
    }

    #[inline]
    pub fn mapping(&self) -> usize {
        self.map
    }

    #[inline]
    pub fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr.as_ptr()
    }

    pub fn truncate(&mut self, len: usize) {
        unsafe {
            if len >= self.len {
                return;
            }
            let remaining_len = self.len - len;
            let s = std::ptr::slice_from_raw_parts_mut(self.as_mut_ptr().add(len), remaining_len);
            self.len = len;
            std::ptr::drop_in_place(s);
        }
    }

    pub fn clear(&mut self) {
        // SAFETY: We must adjust the length before dropping the slice in case the drop
        // operation panics and leaves us with a vector pointing to invalid objects.
        unsafe {
            if self.len == 0 {
                return;
            }
            let s = std::ptr::slice_from_raw_parts_mut(self.as_mut_ptr(), self.len());
            self.len = 0;
            std::ptr::drop_in_place(s);
        }
    }

    pub fn insert(&mut self, index: usize, element: T) {
        // SAFETY: ensure index is in bounds.
        let len = self.len;
        if index > len {
            bounds_check_failed(index, len);
        }

        // SAFETY: ensure capacity is sufficient.
        if len == self.cap {
            self.reserve(1);
        }

        unsafe {
            let ptr = self.ptr.as_ptr().add(index);
            std::ptr::copy(ptr, ptr.offset(1), len - index);
            std::ptr::write(ptr, element);
            self.len = len + 1;
        }
    }

    pub fn remove(&mut self, index: usize) -> T {
        // SAFETY: ensure index is in bounds.
        let len = self.len;
        if index > len {
            bounds_check_failed(index, len);
        }

        unsafe {
            let ptr = self.ptr.as_ptr().add(index);
            let ret = std::ptr::read(ptr);
            std::ptr::copy(ptr.offset(1), ptr, len - index - 1);
            self.len = len - 1;
            ret
        }
    }

    pub fn swap_remove(&mut self, index: usize) -> T {
        // SAFETY: ensure index is in bounds.
        let len = self.len;
        if index > len {
            bounds_check_failed(index, len);
        }

        // SAFETY: in the degenerate case where `len == 1` we swap with ourselves, which is fine.
        unsafe {
            let last = std::ptr::read(self.ptr.as_ptr().add(len - 1));
            let hole = self.ptr.as_ptr().add(index);
            self.len = len - 1;
            std::ptr::replace(hole, last)
        }
    }

    pub fn push(&mut self, element: T) {
        if self.len == self.cap {
            self.reserve(1);
        }

        unsafe {
            let ptr = self.as_mut_ptr().add(self.len);
            std::ptr::write(ptr, element);
            self.len += 1;
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        unsafe {
            self.len -= 1;
            Some(std::ptr::read(self.ptr.as_ptr().add(self.len)))
        }
    }

    #[inline]
    fn capacity_sufficient_for(&self, additional: usize) -> bool {
        // INVARIANT: `cap <= len` means subtraction can't underflow but use `wrapping_sub` to avoid
        // needing to *check* for underflow in debug builds.
        additional <= self.cap.wrapping_sub(self.len)
    }

    fn grow(&mut self, additional: usize) {
        if additional > MAX_ELEMENTS {
            capacity_overflow(additional)
        }

        // SAFETY: can't wrap, but we use wrapping add so we don't need to check.
        let min_capacity = self.cap.wrapping_add(additional);
        if min_capacity > self.map {
            capacity_overflow(additional)
        }

        let growth_amount = self.map / 16;
        let new_capacity = usize::max(self.cap.wrapping_add(growth_amount), min_capacity);
        let new_capacity = usize::min(new_capacity, self.map);

        let layout = Layout::array::<T>(new_capacity).expect("mapping too large");
        mapping_commit(
            unsafe { std::mem::transmute::<*mut T, *mut c_void>(self.ptr.as_ptr()) },
            layout.size(),
        );
        self.cap = new_capacity;
    }

    pub fn reserve(&mut self, additional: usize) {
        if self.capacity_sufficient_for(additional) {
            return;
        }

        self.grow(additional);
    }

    pub fn as_slice(&self) -> &[T] {
        self
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self
    }
}

impl<T> Drop for VirtualVec<T> {
    fn drop(&mut self) {
        let layout = Layout::array::<T>(self.map).unwrap();
        unsafe {
            let ptr = std::mem::transmute::<*mut T, *mut c_void>(self.ptr.as_ptr());
            let ret = munmap(ptr, layout.size());
            assert!(ret == 0);
        }
    }
}

impl<T> std::ops::Deref for VirtualVec<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.as_ptr(), self.len) }
    }
}

impl<T> std::ops::DerefMut for VirtualVec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) }
    }
}

impl<'a, T> IntoIterator for &'a VirtualVec<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> std::slice::Iter<'a, T> {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut VirtualVec<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> std::slice::IterMut<'a, T> {
        self.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_destroy() {
        let vec = VirtualVec::<i32>::new(4096);
        assert_eq!(vec.is_empty(), true);
        assert_eq!(vec.mapping(), 4096);
        assert_eq!(vec.capacity(), 0);
        assert_eq!(vec.len(), 0);
        drop(vec);
    }

    #[test]
    fn push() {
        let mut vec = VirtualVec::<i32>::new(1 << 16);

        for i in 0..1 << 16 {
            vec.push(i);
        }

        assert_eq!(vec.len(), 1 << 16);
        assert_eq!(vec.capacity(), 1 << 16);

        for (i, v) in vec.iter().enumerate() {
            assert_eq!(i as i32, *v);
        }

        vec.clear();

        assert_eq!(vec.len(), 0);
        assert_eq!(vec.capacity(), 65536);

        drop(vec);
    }
}
