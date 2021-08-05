pub unsafe trait Blit {}

unsafe impl Blit for u8 {}
unsafe impl Blit for u16 {}
unsafe impl Blit for u32 {}
unsafe impl Blit for u64 {}

unsafe impl Blit for i8 {}
unsafe impl Blit for i16 {}
unsafe impl Blit for i32 {}
unsafe impl Blit for i64 {}

unsafe impl Blit for f32 {}
unsafe impl Blit for f64 {}
