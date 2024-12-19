use advent_of_code::*;
use std::{convert::identity, mem::transmute, str::FromStr};

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

pub const WORD: [u8; 4] = [b'X', b'M', b'A', b'S'];
pub const OFFSET: [u32; 10] = [32, 0, 4, 8, 12, 32, 16, 20, 24, 28];
pub const MASK: [u32; 10] = [
    0xffffffff,
    0b111 << OFFSET[1],
    0b111 << OFFSET[2],
    0b111 << OFFSET[3],
    0b111 << OFFSET[4],
    0xffffffff,
    0b111 << OFFSET[6],
    0b111 << OFFSET[7],
    0b111 << OFFSET[8],
    0b111 << OFFSET[9],
];
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Dir {
    BottomLeft  = 1,
    Bottom      = 2,
    BottomRight = 3,
    Left        = 4,
    Center      = 5,
    Right       = 6,
    TopLeft     = 7,
    Top         = 8,
    TopRight    = 9,
}
impl Dir {
    pub const fn invert(&self) -> Self {
        // SAFETY: valid Dir values will always produce a valid Dir value
        unsafe { transmute(10 - *self as u8) }
    }
    pub const fn mask(&self) -> u32 {
        MASK[*self as usize]
    }
    pub const fn offset(&self) -> u32 {
        OFFSET[*self as usize]
    }
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
pub const fn shift(
    pos: [usize; 2],
    size: [usize; 2],
    dir: Dir,
    distance: usize,
) -> Option<[usize; 2]> {
    if distance == 0 {
        return Some(pos);
    }
    let dir = dir as u8;
    let x = match (dir - 1) % 3 {
        /* Left */ 0 if pos[0] >= distance => pos[0] - distance,
        /* Center */ 1 => pos[0],
        /* Right */ 2 if pos[0] + distance < size[0] => pos[0] + distance,
        _ => return None,
    };
    let y = match (dir - 1) / 3 {
        /* Bottom */ 0 if pos[1] + distance < size[1] => pos[1] + distance,
        /* Center */ 1 => pos[1],
        /* Top */ 2 if pos[1] >= distance => pos[1] - distance,
        _ => return None,
    };
    Some([x, y])
}
