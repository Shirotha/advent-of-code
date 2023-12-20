use std::{borrow::Cow, iter::Sum};
use itertools::Itertools;
use nom::{
    IResult,
    character::complete::{one_of, line_ending},
    multi::separated_list1,
    combinator::{iterator, opt}
};
use num_traits::PrimInt;
use tap::Pipe;

use crate::{*, parse::*};

#[derive(Debug, Clone)]
struct BitMatrix<T>(Vec<T>, u32);
impl<T: PrimInt + Sum<T>> BitMatrix<T> {
    fn parse(input: &str) -> IResult<&str, Self> {
        let zero = T::zero();
        let one = T::one();
        let mut input = input;
        let mut data = Vec::new();
        let mut width = None;
        loop {
            let mut digit = one;
            let mut iter = iterator(input, one_of(".#"));
            let row = iter.map( |chr| {
                let tmp = if chr == '#' { digit } else { zero };
                digit = digit << 1;
                tmp
            } ).sum::<T>();
            if digit == one { break; }
            data.push(row);
            if let Some(w) = width {
                assert_eq!(w, digit.trailing_zeros());
            } else {
                width = Some(digit.trailing_zeros());
            }
            input = opt(line_ending)(iter.finish()?.0)?.0;
        }
        Ok((input, Self(data, width.unwrap())))
    }
    #[inline]
    fn transpose(&self) -> Self {
        let one = T::one();
        let data = (0..(self.1 as usize)).map( |column| {
            let mask = one << column;
            self.0.iter().enumerate()
                .map( |(row, value)| (*value & mask) >> column << row )
                .sum::<T>()
        } ).collect_vec();
        Self(data, self.0.len() as u32)
    }
}
impl<T: Eq> BitMatrix<T> {
    #[inline]
    fn mirror_axis(&self) -> Option<usize> {
        let max = self.0.len() - 1;
        'outer: for axis in self.0.array_windows()
            .positions( |[a, b]| a == b )
        {
            let (mut i, mut j) = (axis, axis + 1);
            while i != 0 && j != max {
                i -= 1;
                j += 1;
                if self.0[i] != self.0[j] {
                    continue 'outer;
                }
            }
            return Some(axis + 1);
        }
        None
    }
}

pub fn part1(input: &str) -> Answer {
    parse(input, separated_list1(line_ending, BitMatrix::<u32>::parse))?.into_iter()
        .map( |matrix| {
            matrix.mirror_axis().map_or_else(
                || matrix.transpose().mirror_axis().unwrap(),
                |x| 100 * x
            )
        } )
        .sum::<usize>()
        .pipe( |result| Ok(Cow::Owned(result.to_string())) )
}

pub fn part2(input: &str) -> Answer {
    todo!()
}

inventory::submit! { Puzzle::new(2023, 13, 1, part1) }
inventory::submit! { Puzzle::new(2023, 13, 2, part2) }

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    const INPUT1: &str = indoc! {"
        #.##..##.
        ..#.##.#.
        ##......#
        ##......#
        ..#.##.#.
        ..##..##.
        #.#.##.#.

        #...##..#
        #....#..#
        ..##..###
        #####.##.
        #####.##.
        ..##..###
        #....#..#
    "};
    const OUTPUT1: &str = "405";

    const INPUT2: &str = indoc! {"
        #.##..##.
        ..#.##.#.
        ##......#
        ##......#
        ..#.##.#.
        ..##..##.
        #.#.##.#.
        
        #...##..#
        #....#..#
        ..##..###
        #####.##.
        #####.##.
        ..##..###
        #....#..#
    "};
    const OUTPUT2: &str = "400";

    #[test]
    fn test1() {
        assert_eq!(OUTPUT1, &part1(INPUT1).unwrap());
    }

    #[test]
    fn test2() {
        assert_eq!(OUTPUT2, &part2(INPUT2).unwrap());
    }
}