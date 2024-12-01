use std::borrow::Cow;
use nom::{
    IResult,
    branch::alt,
    character::complete::{anychar, digit1},
    combinator::{verify, map, map_res, eof}
};
use itertools::Itertools;
use smallvec::SmallVec;
use tap::Pipe;
use crate::{*, parse::*};

enum AdjacentNumbers {
    Neither,
    Single(u16),
    Both(u16, u16)
}
impl From<Option<u16>> for AdjacentNumbers {
    #[inline]
    fn from(value: Option<u16>) -> Self {
        match value {
            Some(number) => Self::Single(number),
            None => Self::Neither
        }
    }
}
impl From<(u16, Option<u16>)> for AdjacentNumbers {
    #[inline]
    fn from((left, right): (u16, Option<u16>)) -> Self {
        match right {
            Some(right) => AdjacentNumbers::Both(left, right),
            None => AdjacentNumbers::Single(left)
        }
    }
}
impl IntoIterator for AdjacentNumbers {
    type Item = u16;
    type IntoIter = smallvec::IntoIter<[Self::Item; 2]>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Neither => SmallVec::new().into_iter(),
            Self::Single(number) => SmallVec::from_buf_and_len([number, 0], 1).into_iter(),
            Self::Both(left, right) => SmallVec::from_buf([left, right]).into_iter()
        }
    }
}

enum Scheme {
    PartNumber(u16, u8),
    Symbol(char)
}
impl Scheme {
    fn parse(input: &str) -> IResult<&str, Self> {
        alt((
            map_res(digit1, 
                |number: &str|
                    number.parse::<u16>()
                        .map( |n| Self::PartNumber(n, number.len() as u8) )),
            map(verify(anychar, |chr| *chr != '.' ), Scheme::Symbol )
        ))(input)
    }
    #[inline]
    fn is_symbol(&self) -> bool {
        matches!(self, Scheme::Symbol(_))
    }
    #[inline]
    fn number(&self) -> Option<(u16, u8)> {
        match self {
            Scheme::PartNumber(number, len) => Some((*number, *len)),
            Scheme::Symbol(_) => None
        }
    }
}
struct SparseRow {
    offsets: Vec<usize>,
    items: Vec<Scheme>
}
impl SparseRow {
    fn parse(input: &str) -> IResult<&str, Self> {
        map(
            find_many_till(Scheme::parse, eof),
            |schemes| {
                let (offsets, items) = schemes.into_iter().unzip();
                SparseRow { offsets, items }
            }
        )(input)
    }
    #[inline]
    fn is_symbol(&self, offset: usize) -> bool {
        self.offsets
            .partition_point( |pos| *pos < offset )
            .pipe( |i| i != self.offsets.len() && self.offsets[i] == offset && self.items[i].is_symbol() )
    }
    #[inline]
    fn symbol_in_range(&self, min: usize, max: usize) -> bool {
        self.offsets
            .partition_point( |pos| *pos < min )
            .pipe( |min| {
                min != self.offsets.len() && self.offsets[min..].iter()
                    .zip(self.items[min..].iter())
                    .take_while( |&(pos, _)| *pos <= max )
                    .any( |(_, scheme)| scheme.is_symbol() )
            } )
    }
    #[inline]
    fn find_number(&self, offset: usize) -> Option<usize> {
        self.offsets
            .partition_point( |pos| *pos <= offset )
            .pipe( |i| {
                if i == 0 { return None; }
                self.items[i - 1].number()
                    .and_then( |(_, len)| if self.offsets[i - 1] + len as usize > offset { Some(i - 1) } else { None } )
            } )
    }
    #[inline]
    fn number(&self, offset: usize) -> Option<u16> {
        self.find_number(offset).and_then( |i| self.items[i].number().map( |(number, _)| number ) )
    }
    #[inline]
    fn adjacent_numbers(&self, offset: usize) -> AdjacentNumbers {
        if let Some(i) = self.find_number(offset - 1) {
            let pos = self.offsets[i];
            let (left, len) = self.items[i].number().unwrap();
            if pos + len as usize == offset {
                (left, self.number(offset + 1)).into()
            } else { AdjacentNumbers::Single(left) }
        } else { self.number(offset).or_else( || self.number(offset + 1) ).into() }
    }
    fn iter(&self) -> impl Iterator<Item = (&usize, &Scheme)> {
        self.offsets.iter().zip(self.items.iter())
    }
}
struct Schematic {
    rows: Vec<SparseRow>
}
impl Schematic {
    fn parse(input: &str) -> Result<Self, PuzzleError> {
        Ok(Schematic { rows: parse(input, lines(SparseRow::parse))? })
    }
    fn part_numbers(&self) -> impl Iterator<Item = u32> + '_ {
        let imax = self.rows.len() - 1;
        self.rows.iter().enumerate().flat_map( move |(i, row)| {
            row.iter().filter_map( move |(offset, scheme)| {
                match scheme {
                    Scheme::PartNumber(number, len) => {
                        let len = *len as usize;
                        let (left, right) = (offset.saturating_sub(1), offset + len);
                        if row.is_symbol(left) || row.is_symbol(right)
                            || i != 0 && self.rows[i - 1].symbol_in_range(left, right)
                            || i != imax && self.rows[i + 1].symbol_in_range(left, right)
                        {
                            Some(*number as u32)
                        } else { None }
                    },
                    Scheme::Symbol(_) => None
                }
            } )
        } )
    }
    fn gears(&self) -> impl Iterator<Item = (u16, u16)> + '_ {
        let imax = self.rows.len() - 1;
        self.rows.iter().enumerate().flat_map( move |(i, row)| {
            row.iter().filter_map( move |(&offset, scheme)| {
                match scheme {
                    Scheme::Symbol(chr) if *chr == '*' => {
                        let mut iter = self.rows[i].number(offset - 1).into_iter()
                            .chain(self.rows[i].number(offset + 1))
                            .chain(if i == 0 { AdjacentNumbers::Neither }
                                else { self.rows[i - 1].adjacent_numbers(offset) })
                            .chain(if i == imax { AdjacentNumbers::Neither }
                                else { self.rows[i + 1].adjacent_numbers(offset) });
                        iter.next_tuple().and_then(
                            |(left, right)| 
                                if iter.next().is_none() { Some((left, right)) } else { None } 
                        )
                    },
                    _ => None,
                }
            } )
        } )
    }
}

pub fn part1(input: &str) -> Answer {
    Schematic::parse(input)?.part_numbers().sum::<u32>()
        .pipe( |result| Ok(Cow::Owned(result.to_string())) )
}

pub fn part2(input: &str) -> Answer {
    Schematic::parse(input)?.gears()
        .map( |(left, right)| (left as u32) * (right as u32) ).sum::<u32>()
        .pipe( |result| Ok(Cow::Owned(result.to_string())) )
}

inventory::submit! { Puzzle::new(2023, 3, 1, part1) }
inventory::submit! { Puzzle::new(2023, 3, 2, part2) }

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    const INPUT1: &str = indoc! {"
        467..114..
        ...*......
        ..35..633.
        .........!
        617.......
        ..*..+.58.
        ..592.....
        1.....755.
        ...$.*....
        .664.598..
    "};
    const OUTPUT1: &str = "4361";

    const INPUT2: &str = indoc! {"
        467..114..
        ...*......
        ..35..633.
        ......#...
        617*......
        .....+.58.
        ..592.....
        ......755.
        ...$.*....
        .664.598..
    "};
    const OUTPUT2: &str = "467835";

    #[test]
    fn test1() {
        assert_eq!(OUTPUT1, &part1(INPUT1).unwrap());
    }

    #[test]
    fn test2() {
        assert_eq!(OUTPUT2, &part2(INPUT2).unwrap());
    }
}