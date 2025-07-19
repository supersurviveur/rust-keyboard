use core::ops::{Mul,Add};

#[const_trait]
pub trait Integral: Sized + const Mul<Output=Self> + const Add<Output=Self> + Copy {
    fn fromu8(v: u8) -> Self;
    fn fromu16(v: u16) -> Self;
    fn fromusize(v: usize) -> Self;

}

impl const Integral for u8 {
    fn fromu8(v: u8) -> Self {
        v as Self
    }

    fn fromu16(v: u16) -> Self {
        v as Self
    }

    fn fromusize(v: usize) -> Self {
        v as Self
    }
}

impl const Integral for u16 {
    fn fromu8(v: u8) -> Self {
        v as Self
    }

    fn fromu16(v: u16) -> Self {
        v as Self
    }

    fn fromusize(v: usize) -> Self {
        v as Self
    }
    
}

impl const Integral for usize {
    fn fromu8(v: u8) -> Self {
        v as Self
    }

    fn fromu16(v: u16) -> Self {
        v as Self
    }

    fn fromusize(v: usize) -> Self {
        v as Self
    }
}
