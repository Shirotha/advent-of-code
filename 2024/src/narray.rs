use std::ops::{Deref, DerefMut, Index, IndexMut};

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
    data: D,
    size: [usize; N],
    stride: [usize; N],
}
impl<const N: usize, D> NArray<N, D> {
    /// # Safety
    /// Does not check if container is big enough to hold all elements required by size.
    pub unsafe fn from_buffer_unchecked(data: D, size: [usize; N]) -> Self {
        Self {
            data,
            size,
            stride: default_stride(&size),
        }
    }
    /// # Safety
    /// Does not check if container is big enough to hold all elements required by size and stride.
    pub unsafe fn from_buffer_with_stride_unchecked(
        data: D,
        size: [usize; N],
        stride: [usize; N],
    ) -> Self {
        Self { data, size, stride }
    }
    pub const fn size(&self) -> &[usize; N] {
        &self.size
    }
    pub const fn stride(&self) -> &[usize; N] {
        &self.stride
    }
    pub const fn len(&self) -> usize {
        prod(&self.size)
    }
    pub fn is_empty(&self) -> bool {
        any(&self.size, 0)
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
        &self.data[linear(index, &self.stride)]
    }
}
impl<const N: usize, D: DerefMut<Target: IndexMut<usize>>> IndexMut<[usize; N]> for NArray<N, D> {
    fn index_mut(&mut self, index: [usize; N]) -> &mut Self::Output {
        &mut self.data[linear(index, &self.stride)]
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
