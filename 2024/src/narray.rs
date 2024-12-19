use std::{
    hint::black_box,
    mem::MaybeUninit,
    ops::{Deref, DerefMut, Index, IndexMut, Range, RangeInclusive},
};

use crate::RangeAny;

const fn any<const N: usize>(vec: &[usize; N], val: usize) -> bool {
    let mut i = 0;
    while i < N {
        if vec[i] == val {
            return true;
        }
        i += 1;
    }
    false
}
const fn prod<const N: usize>(vec: &[usize; N]) -> usize {
    let mut result = 1;
    let mut i = 0;
    while i < N {
        result *= vec[i];
        i += 1;
    }
    result
}
const fn default_stride<const N: usize>(size: &[usize; N]) -> [usize; N] {
    let mut result = [1; N];
    let mut i = 1;
    while i < N {
        result[i] = result[i - 1] * size[i - 1];
        i += 1;
    }
    result
}
const fn linear<const N: usize>(index: [usize; N], stride: &[usize; N]) -> usize {
    let mut result = 0;
    let mut i = 0;
    while i < N {
        result += index[i] * stride[i];
        i += 1;
    }
    result
}
const fn last<const N: usize>(size: &[usize; N], stride: &[usize; N]) -> Option<usize> {
    let mut index = [0; N];
    let mut i = 0;
    while i < N {
        let Some(j) = size[i].checked_sub(1) else {
            return None;
        };
        index[i] = j;
        i += 1;
    }
    Some(linear(index, stride))
}
const fn next<const N: usize>(index: &mut [usize; N], size: &[usize; N]) {
    if N == 0 {
        return;
    }
    index[0] += 1;
    let mut i = 0;
    while i + 1 < N && index[i] >= size[i] {
        index[i] = 0;
        i += 1;
        index[i] += 1;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NArray<const N: usize, D> {
    size: [usize; N],
    stride: [usize; N],
    offset: usize,
    data: D,
}
impl<const N: usize, D> NArray<N, D> {
    /// # Safety
    /// Does not check if container is big enough to hold all elements required by size.
    pub unsafe fn from_buffer_unchecked(data: D, size: [usize; N]) -> Self {
        Self {
            size,
            stride: default_stride(&size),
            offset: 0,
            data,
        }
    }
    /// # Safety
    /// Does not check if container is big enough to hold all elements required by size and stride.
    pub unsafe fn from_buffer_with_stride_unchecked(
        data: D,
        size: [usize; N],
        stride: [usize; N],
    ) -> Self {
        Self {
            size,
            stride,
            offset: 0,
            data,
        }
    }
    /// # Safety
    /// Does not check if container is big enough to hold all elements required by size and stride.
    pub unsafe fn from_buffer_with_stride_and_offset_unchecked(
        data: D,
        size: [usize; N],
        stride: [usize; N],
        offset: usize,
    ) -> Self {
        Self {
            size,
            stride,
            offset,
            data,
        }
    }
    pub const fn size(&self) -> &[usize; N] {
        &self.size
    }
    pub const fn stride(&self) -> &[usize; N] {
        &self.stride
    }
    pub const fn offset(&self) -> usize {
        self.offset
    }
    pub const fn len(&self) -> usize {
        prod(&self.size)
    }
    pub fn is_empty(&self) -> bool {
        any(&self.size, 0)
    }
}
impl<const N: usize, D: Deref<Target: Index<usize>>> NArray<N, D> {
    pub fn from_buffer(data: D, size: [usize; N]) -> Self {
        let stride = default_stride(&size);
        if let Some(last) = last(&size, &stride) {
            black_box(&data[last]);
        }
        Self {
            size,
            stride,
            offset: 0,
            data,
        }
    }
    pub fn from_buffer_with_stride(data: D, size: [usize; N], stride: [usize; N]) -> Self {
        if let Some(last) = last(&size, &stride) {
            black_box(&data[last]);
        }
        Self {
            size,
            stride,
            offset: 0,
            data,
        }
    }
    pub fn from_buffer_with_stride_and_offset(
        data: D,
        size: [usize; N],
        stride: [usize; N],
        offset: usize,
    ) -> Self {
        if let Some(last) = last(&size, &stride) {
            black_box(&data[offset + last]);
        }
        Self {
            size,
            stride,
            offset: 0,
            data,
        }
    }
}
impl<D: Deref<Target: Index<usize, Output = u8>>> NArray<2, D> {
    pub fn from_ascii(data: D, range: Range<usize>) -> Result<Self, Self> {
        let offset = range.start;
        let len = range.end - offset;
        let until_eol = |start| {
            let mut index = start;
            while index < len {
                let char = data[offset + index];
                if char == b'\r' || char == b'\n' {
                    break;
                }
                index += 1;
            }
            index - start
        };
        let width = until_eol(offset);
        if width == 0 {
            let result = Self {
                data,
                size: [0, 0],
                stride: [0, 0],
                offset,
            };
            return if len > 0 { Err(result) } else { Ok(result) };
        }
        if width + 1 == len || width + 2 == len {
            return Ok(Self {
                data,
                size: [width, 1],
                stride: [1, width],
                offset,
            });
        }
        let next = data[offset + width + 2];
        let line_ending = if next == b'\r' || next == b'\n' { 2 } else { 1 };
        let stride = width + line_ending;
        let mut height = 1;
        let mut index = stride;
        while until_eol(index) == width {
            height += 1;
            index += stride;
        }
        let result = Self {
            data,
            size: [width, height],
            stride: [1, stride],
            offset,
        };
        if index + 1 == len || index + 1 + line_ending == len {
            Ok(result)
        } else {
            Err(result)
        }
    }
}
// TODO: generalize this for other containers
impl<const N: usize, T: Default> NArray<N, Box<[T]>> {
    pub fn new(size: [usize; N]) -> Self {
        let data = (0..prod(&size)).map(|_| T::default()).collect();
        // SAFETY: data is big enough by construction
        unsafe { Self::from_buffer_unchecked(data, size) }
    }
}
impl<const N: usize, D: Deref<Target: Index<usize>>> Index<[usize; N]> for NArray<N, D> {
    type Output = <<D as Deref>::Target as Index<usize>>::Output;

    fn index(&self, index: [usize; N]) -> &Self::Output {
        &self.data[self.offset + linear(index, &self.stride)]
    }
}
impl<const N: usize, D: DerefMut<Target: IndexMut<usize>>> IndexMut<[usize; N]> for NArray<N, D> {
    fn index_mut(&mut self, index: [usize; N]) -> &mut Self::Output {
        &mut self.data[self.offset + linear(index, &self.stride)]
    }
}

pub struct Iter<'a, const N: usize, D: Deref<Target: Index<usize>>> {
    data: &'a NArray<N, D>,
    current: [usize; N],
}
impl<'a, const N: usize, D: Deref<Target: Index<usize>>> Iterator for Iter<'a, N, D> {
    type Item = (
        [usize; N],
        &'a <<D as Deref>::Target as Index<usize>>::Output,
    );

    fn next(&mut self) -> Option<Self::Item> {
        if self.current[N - 1] >= self.data.size()[N - 1] {
            return None;
        }
        let result = (self.current, &self.data[self.current]);
        next(&mut self.current, self.data.size());
        Some(result)
    }
}
impl<'a, const N: usize, D: Deref<Target: Index<usize>>> IntoIterator for &'a NArray<N, D> {
    type IntoIter = Iter<'a, N, D>;
    type Item = <Self::IntoIter as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            data: self,
            current: [0; N],
        }
    }
}
impl<const N: usize, D: Deref<Target: Index<usize>>> NArray<N, D> {
    pub fn iter(&self) -> Iter<N, D> {
        Iter {
            data: self,
            current: [0; N],
        }
    }
}
pub struct IterMut<'a, const N: usize, D: DerefMut<Target: IndexMut<usize>>> {
    data: &'a mut NArray<N, D>,
    current: [usize; N],
}
impl<'a, const N: usize, D: DerefMut<Target: IndexMut<usize>>> Iterator for IterMut<'a, N, D> {
    type Item = (
        [usize; N],
        &'a mut <<D as Deref>::Target as Index<usize>>::Output,
    );

    fn next(&mut self) -> Option<Self::Item> {
        if self.current[N - 1] >= self.data.size()[N - 1] {
            return None;
        }
        // SAFETY: value will not be accessed anywhere else
        let value = unsafe {
            (&mut self.data[self.current] as *mut <<D as Deref>::Target as Index<usize>>::Output)
                .as_mut()
                .unwrap()
        };
        let result = (self.current, value);
        next(&mut self.current, self.data.size());
        Some(result)
    }
}
impl<'a, const N: usize, D: DerefMut<Target: IndexMut<usize>>> IntoIterator
    for &'a mut NArray<N, D>
{
    type IntoIter = IterMut<'a, N, D>;
    type Item = <Self::IntoIter as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        IterMut {
            data: self,
            current: [0; N],
        }
    }
}
impl<const N: usize, D: DerefMut<Target: IndexMut<usize>>> NArray<N, D> {
    pub fn iter_mut(&mut self) -> Iter<N, D> {
        Iter {
            data: self,
            current: [0; N],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Projector<const N: usize, const M: usize> {
    ranges: [(usize, RangeInclusive<usize>); M],
    pins: Box<[(usize, usize)]>,
}
impl<const N: usize, const M: usize> Projector<N, M> {
    pub fn new(desc: &[RangeAny<usize>; N]) -> Option<Self> {
        let mut ranges = MaybeUninit::uninit_array();
        let mut pins = Vec::with_capacity(N - M);
        let mut range_count = 0;
        let mut i = 0;
        while i < N {
            match &desc[i] {
                RangeAny::Single(pin) => {
                    if pins.len() >= N - M {
                        return None;
                    }
                    pins.push((i, *pin));
                }
                range => {
                    if range_count >= M {
                        return None;
                    }
                    let start = range.start().unwrap_or(0);
                    let end = range.end()?;
                    ranges[range_count].write((i, start..=end));
                    range_count += 1;
                }
            }
            i += 1;
        }
        // SAFETY: pin_count == P == N - M and range_count == M here (would have returned early otherwise)
        Some(Self {
            ranges: unsafe { MaybeUninit::array_assume_init(ranges) },
            pins: pins.into_boxed_slice(),
        })
    }
    pub fn new_with_size(desc: &[RangeAny<usize>; N], size: &[usize; N]) -> Option<Self> {
        let mut ranges = MaybeUninit::uninit_array();
        let mut pins = Vec::with_capacity(N - M);
        let mut range_count = 0;
        let mut i = 0;
        while i < N {
            match &desc[i] {
                RangeAny::Single(pin) => {
                    if pins.len() >= N - M {
                        return None;
                    }
                    pins.push((i, *pin));
                }
                range => {
                    if range_count >= M {
                        return None;
                    }
                    let start = range.start().unwrap_or(0);
                    let end = match range.end() {
                        Some(x) => x,
                        None => size[i].saturating_sub(1),
                    };
                    ranges[range_count].write((i, start..=end));
                    range_count += 1;
                }
            }
            i += 1;
        }
        // SAFETY: pin_count == P == N - M and range_count == M here (would have returned early otherwise)
        Some(Self {
            ranges: unsafe { MaybeUninit::array_assume_init(ranges) },
            pins: pins.into_boxed_slice(),
        })
    }
    pub const fn pin(&mut self, pins: &[usize]) {
        let mut i = 0;
        while i < N - M {
            self.pins[i].1 = pins[i];
            i += 1;
        }
    }
}

const fn project<const N: usize, const M: usize, D>(
    array: &NArray<N, D>,
    proj: &Projector<N, M>,
) -> Option<([usize; M], [usize; M], usize)> {
    let mut offset = 0;
    let mut i = 0;
    while i < N - M {
        let (dim, index) = proj.pins[i];
        if index >= array.size[dim] {
            return None;
        }
        offset += array.stride[dim] * index;
        i += 1;
    }
    let mut size = [0; M];
    let mut stride = [0; M];
    i = 0;
    while i < M {
        let (dim, range) = &proj.ranges[i];
        if *range.end() >= array.size[*dim] {
            return None;
        }
        stride[i] = array.stride[*dim];
        offset += stride[i] * *range.start();
        size[i] = *range.end() - *range.start() + 1;
        i += 1;
    }
    Some((size, stride, offset))
}
impl<const N: usize, D: Deref> NArray<N, D> {
    pub fn project<const M: usize>(&self, proj: &Projector<N, M>) -> Option<NArray<M, &D::Target>> {
        let (size, stride, offset) = project(self, proj)?;
        Some(NArray {
            size,
            stride,
            offset,
            data: self.data.deref(),
        })
    }
    pub fn view<const M: usize>(
        &self,
        desc: &[RangeAny<usize>; N],
    ) -> Option<NArray<M, &D::Target>> {
        let proj = Projector::<N, M>::new_with_size(desc, self.size())?;
        self.project(&proj)
    }
}
impl<const N: usize, D: DerefMut> NArray<N, D> {
    pub fn project_mut<const M: usize>(
        &mut self,
        proj: &Projector<N, M>,
    ) -> Option<NArray<M, &mut D::Target>> {
        let (size, stride, offset) = project(self, proj)?;
        Some(NArray {
            size,
            stride,
            offset,
            data: self.data.deref_mut(),
        })
    }
    pub fn view_mut<const M: usize>(
        &mut self,
        desc: &[RangeAny<usize>; N],
    ) -> Option<NArray<M, &mut D::Target>> {
        let proj = Projector::<N, M>::new_with_size(desc, self.size())?;
        self.project_mut(&proj)
    }
}
#[macro_export]
macro_rules! view {
    [$array: expr, $( $arg: expr ),*] => {
        $array.view(&[ $( RangeAny::from($arg) ),+ ])
    };
}
#[macro_export]
macro_rules! view_mut {
    [$array: expr, $( $arg: expr ),*] => {
        $array.view_mut(&[ $( RangeAny::from($arg) ),+ ])
    };
}
