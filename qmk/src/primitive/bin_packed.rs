use core::iter::Iterator;
use core::ops::{Index, IndexMut};

#[derive(Copy, Clone)]
pub struct BinPackedArray<const N: usize> {
    pub data: [u8; N],
}

pub trait IndexByValue<Idx> {
    type Data;
    fn at(&self, index: Idx) -> Self::Data;
}

///marker trait for what is a proper data storage, to restrict later impls
pub(super) trait DataStorage {}

pub trait IndexByValueMut<Idx>: IndexByValue<Idx> {
    fn set(&mut self, index: Idx, value: <Self as IndexByValue<Idx>>::Data);
}

impl<const N: usize> DataStorage for BinPackedArray<N> {}
impl<const N: usize, T> DataStorage for [T; N] {}
impl<T> DataStorage for [T] {}

impl<const N: usize> Default for BinPackedArray<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> BinPackedArray<N> {
    #[inline(always)]
    pub const fn new() -> Self {
        Self { data: [0; N] }
    }
    #[inline(always)]
    pub fn iter_enumerate<T: FnMut(bool, u16) -> bool>(&mut self, mut fun: T) {
        self.data.iter_mut().enumerate().for_each(|(i, octet)| {
            let mut loctet = *octet;
            let mut result: u8 = 0;
            let index = 8 * i;
            for i in 0..8_u8 {
                result <<= 1;
                result += fun(loctet & 1 != 0, index as u16 + i as u16) as u8;
                loctet >>= 1;
            }
            *octet = result;
        });
    }
}

impl<const N: usize> IndexByValue<u16> for BinPackedArray<N> {
    type Data = bool;
    #[inline(always)]
    fn at(&self, index: u16) -> bool {
        let data_idx = index / 8;
        let char_idx = index % 8;
        let char = self.data[data_idx as usize];
        (char & (1 << char_idx)) != 0
    }
}
impl<const N: usize> IndexByValueMut<u16> for BinPackedArray<N> {
    #[inline(always)]
    fn set(&mut self, index: u16, value: bool) {
        let data_idx = index / 8;
        let char_idx = index % 8;
        let mut char = self.data[data_idx as usize];
        let mask = 1 << char_idx;

        if value {
            char |= mask;
        } else {
            char ^= char & mask;
        }

        self.data[data_idx as usize] = char;
    }
}

impl<T> IndexByValue<usize> for T
where
    T: Index<usize> + DataStorage,
    <T as Index<usize>>::Output: Copy,
{
    type Data = <Self as Index<usize>>::Output;

    fn at(&self, index: usize) -> Self::Data {
        self[index]
    }
}
impl<T> IndexByValueMut<usize> for T
where
    T: IndexMut<usize> + DataStorage,
    <T as Index<usize>>::Output: Copy,
{
    fn set(&mut self, index: usize, value: <Self as IndexByValue<usize>>::Data) {
        (*self)[index] = value;
    }
}
