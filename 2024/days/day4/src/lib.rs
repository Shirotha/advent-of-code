use advent_of_code::*;
use std::{convert::identity, str::FromStr};

#[derive(Debug)]
pub struct Input {
    pub data: NArray<2, Box<[u8]>>,
}
impl FromStr for Input {
    type Err = nom::Err<nom::error::Error<String>>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // SAFETY: assumes that input is in ASCII
        let s: Box<[_]> = s.as_bytes().into();
        let len = s.len();
        Ok(Input {
            data: NArray::from_ascii(s, 0..len).map_or_else(identity, identity),
        })
    }
}

pub type Pos<'a> = NIndex<'a, 2>;
pub type Dir = NDir<u8, 2>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Dirs {
    BottomLeft  = 0b0110,
    Bottom      = 0b0100,
    BottomRight = 0b0101,
    Left        = 0b0010,
    Neutral     = 0b0000,
    Right       = 0b0001,
    TopLeft     = 0b1010,
    Top         = 0b1000,
    TopRight    = 0b1001,
}
impl Dirs {
    pub const fn dir(self) -> Dir {
        // SAFETY: values of Dirs are valid BCT numbers by construction
        unsafe { Dir::from_bits_unchecked(self as u8) }
    }
}
pub const WORD: [u8; 4] = [b'X', b'M', b'A', b'S'];
pub const OFFSET: [u32; 8] = [0, 4, 8, 12, 16, 20, 24, 28];
pub const MASK: [u32; 8] = [
    0b111 << OFFSET[0],
    0b111 << OFFSET[1],
    0b111 << OFFSET[2],
    0b111 << OFFSET[3],
    0b111 << OFFSET[4],
    0b111 << OFFSET[5],
    0b111 << OFFSET[6],
    0b111 << OFFSET[7],
];
pub fn offset(dir: Dir) -> u32 {
    OFFSET[dir.index() - 1]
}

pub const fn linear_search(array: &[u8], item: u8) -> Option<usize> {
    let mut index = 0;
    while index < array.len() {
        if array[index] == item {
            return Some(index);
        }
        index += 1;
    }
    None
}
