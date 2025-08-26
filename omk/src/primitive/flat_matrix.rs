use super::{BinPackedArray, DataStorage, IndexByValue, IndexByValueMut};
use core::marker::PhantomData;
use core::ops::{Add, DerefMut, Mul};
use core::panic;

pub struct Array2D<const COL: u8, const ROW: u8, Idx, T: IndexByValueMut<Idx>> {
    backend: T,
    _phantom: PhantomData<Idx>,
}

pub struct Slice2D<Idx, T: IndexByValueMut<Idx>> {
    backend: T,
    col: u8,
    row: u8,
    _phantom: PhantomData<Idx>,
}

pub struct SizedView<
    const COL: u8,
    const ROW: u8,
    Idx: From<u16> + Copy,
    Backend: IndexByValueMut<Idx>,
    T: DerefMut<Target = Backend>,
> {
    backend: T,
    stride: u8,
    offset: Idx,
    _phantom: PhantomData<(Idx, Backend)>,
}

pub struct UnsizedView<
    Idx: core::convert::From<u16> + Copy,
    Backend: IndexByValueMut<Idx>,
    T: DerefMut<Target = Backend>,
> {
    backend: T,
    stride: u8,
    offset: Idx,
    row: u8,
    col: u8,
    _phantom: PhantomData<(Idx, Backend)>,
}

impl<const COL: u8, const ROW: u8, const N: usize> Default
    for Array2D<COL, ROW, u16, BinPackedArray<N>>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<const COL: u8, const ROW: u8, const N: usize> Array2D<COL, ROW, u16, BinPackedArray<N>> {
    pub const fn new() -> Self {
        if N != (COL as usize * ROW as usize).div_ceil(8) {
            panic!()
        } else {
            Self {
                backend: BinPackedArray { data: [0; N] },
                _phantom: PhantomData,
            }
        }
    }
    pub const fn from_existing(data: BinPackedArray<N>) -> Self {
        if N != (COL as usize * ROW as usize).div_ceil(8) {
            panic!()
        } else {
            Self {
                backend: data,
                _phantom: PhantomData,
            }
        }
    }
}

impl<const COL: u8, const ROW: u8, const N: usize, Data: Copy> Array2D<COL, ROW, usize, [Data; N]> {
    pub const fn new(start: Data) -> Self {
        if N != (COL as usize * ROW as usize) {
            panic!()
        } else {
            Self {
                backend: [start; N],
                _phantom: PhantomData,
            }
        }
    }
}

impl<'a, const COL: u8, const ROW: u8, const N: usize, Data: Copy>
    SizedView<COL, ROW, usize, [Data; N], &'a mut [Data; N]>
{
    pub const fn new(data: &'a mut [Data; N]) -> Self {
        SizedView {
            backend: data,
            stride: COL,
            offset: 0,
            _phantom: PhantomData,
        }
    }
}

pub trait Container2D<Idx, T>
where
    Idx: Copy + From<u8> + From<u16> + Add<Output = Idx> + Mul<Output = Idx>,
    T: IndexByValueMut<Idx>,
{
    type SliceContainer: IndexByValueMut<Idx>;
    fn backend_mut(&mut self) -> &mut Self::SliceContainer;
    fn backend(&self) -> &Self::SliceContainer;
    fn row(&self) -> u8;
    fn col(&self) -> u8;
    fn stride(&self) -> u8;
    fn offset(&self) -> Idx;
    #[inline(always)]
    fn set(
        &mut self,
        col: Idx,
        row: Idx,
        value: <<Self as Container2D<Idx, T>>::SliceContainer as IndexByValue<Idx>>::Data,
    ) {
        let index: Idx =
            Into::<Idx>::into(self.offset()) + col + Into::<Idx>::into(self.stride()) * row;
        self.backend_mut().set(index, value);
    }
    #[inline(always)]
    fn get(
        &self,
        col: Idx,
        row: Idx,
    ) -> <<Self as Container2D<Idx, T>>::SliceContainer as IndexByValue<Idx>>::Data {
        let index: Idx = self.offset() + col + Into::<Idx>::into(self.stride()) * row;
        self.backend().at(index)
    }

    ///# SAFETY
    ///the view must fit in self, aka startcol + COL >= self.col(), and same for rows
    #[inline(always)]
    unsafe fn extract_sized_view_unchecked<const COL: u8, const ROW: u8>(
        &mut self,
        startcol: u8,
        startrow: u8,
    ) -> SizedView<COL, ROW, Idx, Self::SliceContainer, &mut Self::SliceContainer> {
        let stride = self.stride();
        let offset = self.offset();
        SizedView {
            backend: self.backend_mut(),
            stride,
            offset: (offset + startcol.into() + <u8 as Into<Idx>>::into(stride) * startrow.into()),
            _phantom: PhantomData,
        }
    }

    ///# SAFETY
    ///the view must fit in self, aka startcol + numcol >= self.col(), and same for rows
    #[inline(always)]
    unsafe fn extract_unsized_view_unchecked(
        &mut self,
        startcol: u8,
        startrow: u8,
        numcol: u8,
        numrow: u8,
    ) -> UnsizedView<Idx, Self::SliceContainer, &mut Self::SliceContainer> {
        let stride = self.stride();
        let offset = self.offset();
        UnsizedView {
            backend: self.backend_mut(),
            stride,
            offset: (offset + startcol.into() + <u8 as Into<Idx>>::into(stride) * startrow.into()),
            row: numrow,
            col: numcol,
            _phantom: PhantomData,
        }
    }
}

impl<Idx, T, const COL: u8, const ROW: u8> Container2D<Idx, T> for Array2D<COL, ROW, Idx, T>
where
    T: IndexByValueMut<Idx> + DataStorage,
    Idx: Copy + From<u16> + Mul<Output = Idx> + Add<Output = Idx> + From<u8>,
{
    type SliceContainer = T;
    #[inline(always)]
    fn backend_mut(&mut self) -> &mut T {
        &mut self.backend
    }

    #[inline(always)]
    fn backend(&self) -> &T {
        &self.backend
    }

    #[inline(always)]
    fn row(&self) -> u8 {
        ROW
    }

    #[inline(always)]
    fn col(&self) -> u8 {
        COL
    }

    #[inline(always)]
    fn stride(&self) -> u8 {
        COL
    }

    #[inline(always)]
    fn offset(&self) -> Idx {
        0u8.into()
    }
}

impl<Idx, T> Container2D<Idx, T> for Slice2D<Idx, T>
where
    T: IndexByValueMut<Idx> + DataStorage,
    Idx: Copy + From<u16> + Mul<Output = Idx> + Add<Output = Idx> + From<u8>,
{
    type SliceContainer = T;
    #[inline(always)]
    fn backend_mut(&mut self) -> &mut T {
        &mut self.backend
    }

    #[inline(always)]
    fn backend(&self) -> &T {
        &self.backend
    }

    #[inline(always)]
    fn row(&self) -> u8 {
        self.row
    }

    #[inline(always)]
    fn col(&self) -> u8 {
        self.col
    }

    #[inline(always)]
    fn stride(&self) -> u8 {
        self.col
    }

    #[inline(always)]
    fn offset(&self) -> Idx {
        0u8.into()
    }
}

impl<Idx, Backend, T, const COL: u8, const ROW: u8> Container2D<Idx, Backend>
    for SizedView<COL, ROW, Idx, Backend, T>
where
    T: DerefMut<Target = Backend>,
    Backend: IndexByValueMut<Idx> + DataStorage,
    Idx: Copy + Mul<Output = Idx> + Add<Output = Idx> + From<u8> + From<u16>,
{
    type SliceContainer = Backend;
    #[inline(always)]
    fn backend_mut(&mut self) -> &mut Backend {
        self.backend.deref_mut()
    }

    #[inline(always)]
    fn backend(&self) -> &Backend {
        self.backend.deref()
    }

    #[inline(always)]
    fn row(&self) -> u8 {
        ROW
    }

    #[inline(always)]
    fn col(&self) -> u8 {
        COL
    }

    #[inline(always)]
    fn stride(&self) -> u8 {
        self.stride
    }

    #[inline(always)]
    fn offset(&self) -> Idx {
        self.offset
    }
}

impl<Idx, Backend, T> Container2D<Idx, Backend> for UnsizedView<Idx, Backend, T>
where
    T: DerefMut<Target = Backend>,
    Backend: IndexByValueMut<Idx> + DataStorage,
    Idx: Copy + Mul<Output = Idx> + Add<Output = Idx> + From<u8> + From<u16>,
{
    type SliceContainer = Backend;
    #[inline(always)]
    fn backend_mut(&mut self) -> &mut Backend {
        self.backend.deref_mut()
    }

    #[inline(always)]
    fn backend(&self) -> &Backend {
        self.backend.deref()
    }

    #[inline(always)]
    fn row(&self) -> u8 {
        self.row
    }

    #[inline(always)]
    fn col(&self) -> u8 {
        self.col
    }

    #[inline(always)]
    fn stride(&self) -> u8 {
        self.stride
    }

    #[inline(always)]
    fn offset(&self) -> Idx {
        self.offset
    }
}
