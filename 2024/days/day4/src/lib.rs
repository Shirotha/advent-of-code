use std::{
    ops::{Index, IndexMut},
    str::FromStr,
};

#[derive(Debug)]
pub struct Input {
    pub data: StridedMatrix<u8>,
}
impl FromStr for Input {
    type Err = nom::Err<nom::error::Error<String>>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // SAFETY: assumes that input is in ASCII
        let s = s.as_bytes();
        let height = s.iter().filter(|c| **c == b'\n').count() + 1;
        let width = if height != 1 {
            // SAFETY: unwrap: if height is > 1 there is at least one line ending
            s.iter()
                .enumerate()
                .find(|&(_, &c)| c == b'\r' || c == b'\n')
                .unwrap()
                .0
        } else {
            s.len()
        };
        let stride = if s[width] == b'\n' {
            width + 1
        } else {
            width + 2
        };
        Ok(Input {
            data: StridedMatrix::from_buffer(s, (width, height), stride),
        })
    }
}

#[derive(Debug)]
pub struct StridedMatrix<T> {
    data: Box<[T]>,
    size: (usize, usize),
    stride: usize,
}
impl<T> StridedMatrix<T> {
    pub fn from_buffer(data: impl Into<Box<[T]>>, size: (usize, usize), stride: usize) -> Self {
        let data = data.into();
        assert!(size.0 > 0 && size.1 > 0, "matrix can't be empty");
        assert!(
            data.len() >= (size.1 - 1) * stride + size.0,
            "buffer not big enough"
        );
        Self { data, size, stride }
    }
    pub fn size(&self) -> &(usize, usize) {
        &self.size
    }
}
impl<T: Default> StridedMatrix<T> {
    pub fn new(size: (usize, usize)) -> Self {
        let data = (0..size.0 * size.1).map(|_| T::default()).collect();
        Self {
            data,
            size,
            stride: size.0,
        }
    }
}
impl<T> Index<[usize; 2]> for StridedMatrix<T> {
    type Output = T;

    fn index(&self, index: [usize; 2]) -> &Self::Output {
        &self.data[index[1] * self.stride + index[0]]
    }
}
impl<T> IndexMut<[usize; 2]> for StridedMatrix<T> {
    fn index_mut(&mut self, index: [usize; 2]) -> &mut Self::Output {
        &mut self.data[index[1] * self.stride + index[0]]
    }
}
