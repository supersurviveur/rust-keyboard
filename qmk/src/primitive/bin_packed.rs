use core::iter::Iterator;
use core::ops::{Index, IndexMut};
use super::integral::Integral;

#[derive(Copy, Clone)]
pub struct BinPackedArray<const N: usize> {
    pub data: [u8; N],
}

#[const_trait]
pub trait IndexByValue<Idx: const Integral> {
    type Data;
    fn at(&self, index: Idx) -> Self::Data;
}

///marker trait for what is a proper data storage, to restrict later impls
pub(crate) trait DataStorage {}

#[const_trait]
pub trait IndexByValueMut<Idx: const Integral>: IndexByValue<Idx> {
    fn set(&mut self, index: Idx, value: <Self as IndexByValue<Idx>>::Data);
}

impl<const N: usize> DataStorage for BinPackedArray<N> {}
impl<const N: usize, T> DataStorage for [T; N] {}
impl<T> DataStorage for [T] {}

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
            for i in 0..(8_u8) {
                result <<= 1;
                result += fun(loctet & 1 != 0, index as u16 + i as u16) as u8;
                loctet >>= 1;
            }
            *octet = result;
        });
    }
}

impl<const N: usize> const IndexByValue<usize> for BinPackedArray<N> {
    type Data = bool;
    #[inline(always)]
    fn at(&self, index: usize) -> bool {
        let data_idx = index / 8;
        let char_idx = index % 8;
        let char = self.data[data_idx as usize];
        (char & (1 << char_idx)) != 0
    }
}
impl<const N: usize> const IndexByValueMut<usize> for BinPackedArray<N> {
    #[inline(always)]
    fn set(&mut self, index: usize, value: bool) {
        let data_idx = index / 8;
        let char_idx = index % 8;
        let mut char = self.data[data_idx as usize];
        let mask = 1 << char_idx;

        if value {
            char |= mask;
        } else {
            char &= !mask;
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
