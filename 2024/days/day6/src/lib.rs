use advent_of_code::*;
use std::{convert::identity, str::FromStr};

#[derive(Debug)]
pub struct Input {
    pub map: NArray<2, Box<[u8]>>,
}
impl FromStr for Input {
    type Err = nom::Err<nom::error::Error<String>>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // SAFETY: assumes that input is in ASCII
        let s: Box<[_]> = s.as_bytes().into();
        let len = s.len();
        let map = NArray::from_ascii(s, 0..len).map_or_else(identity, identity);
        // TODO: find guard
        Ok(Input { map })
    }
}
