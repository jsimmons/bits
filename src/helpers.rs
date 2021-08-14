use std::{
    alloc::{handle_alloc_error, Layout},
    mem::MaybeUninit,
};

pub fn box_uninit<T>() -> Box<MaybeUninit<T>> {
    let layout = Layout::new::<MaybeUninit<T>>();
    unsafe {
        let ptr: *mut MaybeUninit<T> = std::alloc::alloc(layout).cast();
        if ptr.is_null() {
            handle_alloc_error(layout)
        }
        Box::from_raw(ptr)
    }
}

pub fn box_uninit_array<T, const LEN: usize>() -> Box<[MaybeUninit<T>; LEN]> {
    let layout = Layout::new::<MaybeUninit<[MaybeUninit<T>; LEN]>>();
    unsafe {
        let ptr: *mut [MaybeUninit<T>; LEN] = std::alloc::alloc(layout).cast();
        if ptr.is_null() {
            handle_alloc_error(layout)
        }
        Box::from_raw(ptr)
    }
}
