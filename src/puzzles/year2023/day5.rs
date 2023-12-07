use std::{
    borrow::Cow,
    ops::Add, cmp::Ordering,
};
use nom::{
    IResult,
    bytes::complete::tag,
    character::complete::{char, digit1, line_ending, anychar},
    sequence::{tuple, delimited, preceded, separated_pair},
    combinator::map_res,
    multi::{separated_list0, many_till, count},
};
use itertools::Itertools;
use num_traits::PrimInt;
use tap::{Tap, Pipe};

use crate::{*, parse::*};

#[derive(Debug)]
struct PiecewiseLinear<T> {
    borders: Vec<T>,
    shift: Vec<T>,
}
impl<T> PiecewiseLinear<T> {
    #[inline]
    fn new(default: T) -> Self {
        Self { borders: Vec::new(), shift: vec![default] }
    }
}
impl<T: PrimInt> PiecewiseLinear<T> {
    fn parse(input: &str) -> IResult<&str, Self> {
        let (input, mut segments) = preceded(
            many_till(anychar, line_ending), 
            separated_list0(line_ending,
                tuple((
                    map_res(digit1, |number| T::from_str_radix(number, 10) ),
                    delimited(
                        char(' '),
                        map_res(digit1, |number| T::from_str_radix(number, 10) ),
                        char(' ')
                    ),
                    map_res(digit1, |number| T::from_str_radix(number, 10))
                ))
            )
        )(input)?;
        segments.sort_unstable_by_key( |(_, src, _)| *src );
        let (mut borders, mut shift) = (segments.len() << 1)
            .pipe( |n| (Vec::with_capacity(n), Vec::with_capacity(n)) );
        shift.push(T::zero());
        for (dst, src, len) in segments.into_iter() {
            if borders.last().is_some_and( |last| *last == src ) { 
                *shift.last_mut().unwrap() = dst - src;
            } else {
                borders.push(src);
                shift.push(dst - src);
            }
            borders.push(src + len);
            shift.push(T::zero());
        }
        Ok((input, Self { shift, borders }))
    }
    #[inline]
    fn apply(&self, value: T) -> T {
        self.borders.partition_point( |border| *border < value)
            .pipe( |i| value + self.shift[i] )
    }
}
impl<T: PrimInt> Add for PiecewiseLinear<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let (mut borders1, mut shift1) =
            (self.borders.into_iter().peekable(), self.shift.into_iter());
        let (mut borders2, mut shift2) =
            (rhs.borders.into_iter().peekable(), rhs.shift.into_iter());
        let (mut borders, mut shift) = (shift1.len() + shift2.len())
            .pipe( |n| (Vec::with_capacity(n), Vec::with_capacity(n)) );
        shift.push(shift1.next().unwrap() + shift2.next().unwrap());
        loop {
            match (borders1.peek(), borders2.peek()) {
                (Some(lhs), Some(rhs)) => match lhs.cmp(rhs) {
                    Ordering::Less => {
                        // push lhs
                    },
                    Ordering::Equal => {
                        // push lhs + rhs
                    },
                    Ordering::Greater => {
                        // push rhs
                    }
                },
                (Some(_), None) => {
                    // push lhs
                },
                (None, Some(_)) => {
                    // push rhs
                },
                (None, None) => return Self { borders, shift }
            }
        }
    }
}

#[inline]
fn seeds<T: PrimInt>(input: &str) -> IResult<&str, Vec<T>> {
    preceded(tag("seeds: "), separated_list0(char(' '), 
        map_res(digit1, |number| T::from_str_radix(number, 10))
    ))(input)
}

#[inline]
fn maps<T: PrimInt>(input: &str) -> IResult<&str, Vec<PiecewiseLinear<T>>> {
    separated_list0(count(line_ending, 2), PiecewiseLinear::parse)(input)
}

pub fn part1(input: &str) -> Answer {
    parse(input, separated_pair(seeds, count(line_ending, 2), maps))?
        .pipe( |(seeds, maps)|
            maps.into_iter()
                .fold(PiecewiseLinear::new(0),
                    |acc, map| acc.add(map)
                )
                .pipe( |map| 
                    seeds.into_iter()
                        .map( |seed| map.apply(seed) )
                        .min()
                        .expect("non empty collection")
                )
        )
        .pipe( |result: i32| Ok(Cow::Owned(result.to_string())) )
}

pub fn part2(input: &str) -> Answer {
    todo!()
}

inventory::submit! { Puzzle::new(2023, 5, 1, part1) }
inventory::submit! { Puzzle::new(2023, 5, 2, part2) }

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    const INPUT1: &str = indoc! {"
        seeds: 79 14 55 13

        seed-to-soil map:
        50 98 2
        52 50 48

        soil-to-fertilizer map:
        0 15 37
        37 52 2
        39 0 15

        fertilizer-to-water map:
        49 53 8
        0 11 42
        42 0 7
        57 7 4

        water-to-light map:
        88 18 7
        18 25 70

        light-to-temperature map:
        45 77 23
        81 45 19
        68 64 13

        temperature-to-humidity map:
        0 69 1
        1 0 69

        humidity-to-location map:
        60 56 37
        56 93 4
    "};
    const OUTPUT1: &str = "";

    const INPUT2: &str = indoc!{"
    
    "};
    const OUTPUT2: &str = "";

    #[test]
    fn test1() {
        assert_eq!(OUTPUT1, &part1(INPUT1).unwrap());
    }

    #[test]
    fn test2() {
        assert_eq!(OUTPUT2, &part2(INPUT2).unwrap());
    }
}