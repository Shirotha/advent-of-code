use std::borrow::Cow;
use nom::{
    IResult,
    bytes::complete::tag,
    character::complete::{char, digit1, line_ending, anychar},
    sequence::{tuple, delimited, preceded, separated_pair},
    combinator::map_res,
    multi::{separated_list0, many_till, count},
};
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
    fn contract(mut self, mut other: Self) -> Self {
        if self.borders.is_empty() {
            let default = self.shift[0];
            other.shift.iter_mut().for_each( |shift| *shift = *shift + default );
            return other;
        } else if other.borders.is_empty() {
            let default = other.shift[0];
            self.shift.iter_mut().for_each( |shift| *shift = *shift + default );
            return self;
        }
        let (mut borders, mut shift) = (self.shift.len() + other.shift.len())
            .pipe( |n| (Vec::with_capacity(n), Vec::with_capacity(n)) );
        shift.push(self.shift[0] + other.shift[0]);
        let first = self.borders[0];
        for (border, value) in other.borders.iter()
            .zip(other.shift.iter().skip(1))
            .take_while( |(border, _)| **border < first)
        {
            borders.push(*border);
            shift.push(*value);
        }
        let (mut i, mut carry) = (0, None);
        loop {
            let border = match (self.borders.get(i), carry) {
                (Some(&a), Some(b)) =>
                    if a <= b { i += 1; a } else { b },
                (Some(&a), None) => { i += 1; a },
                (None, Some(b)) => b,
                (None, None) => return Self { borders, shift }
            };
            borders.push(border);
            shift.push(
                other.borders
                    .partition_point( |other| *other <= border )
                    .tap ( |j| carry = other.borders.get(*j).copied() )
                    .pipe( |j| self.shift[i] + other.shift[j] )
            )
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
                    |acc, map| acc.contract(map)
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
    const OUTPUT1: &str = "35";

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