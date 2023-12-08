use std::{
    borrow::Cow,
    ops::Range,
    iter::once
};
use itertools::Itertools;
use nom::{
    IResult,
    bytes::complete::tag,
    character::complete::{char, digit1, line_ending, anychar},
    sequence::{tuple, delimited, preceded, separated_pair},
    combinator::map_res,
    multi::{separated_list0, many_till, count},
};
use num_traits::PrimInt;
use tap::Pipe;

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
    #[inline]
    fn range(&self, range: Range<T>) -> Iter<T> {
        Iter { pl: self, range, position: None }
    }
    fn contract(self, other: Self) -> Self {
        let (mut borders, shift) = once(T::min_value())
            .chain(self.borders)
            .chain(once(T::max_value()))
            .tuple_windows()
            .zip(self.shift)
            .flat_map( |((start, end), shift)|
                other.range((start + shift)..(end + shift))
                    .map( move |(range, value)| (range.end - shift, shift + value) )
            )
            .unzip::<T, T, Vec<_>, Vec<_>>();
        borders.pop();
        Self { borders, shift }
    }
}

#[derive(Debug)]
struct Iter<'a, T> {
    pl: &'a PiecewiseLinear<T>,
    range: Range<T>,
    position: Option<(bool, usize)>
}
impl<'a, T: PrimInt> Iterator for Iter<'a, T> {
    type Item = (Range<T>, T);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((last, i)) = self.position {
            if last { return None; }
            if let Some(end) = self.pl.borders.get(i + 1)
                .and_then( |&end| if end < self.range.end { Some(end) } else { None } ) 
            {
                self.position = Some((false, i + 1));
                Some((self.pl.borders[i]..end, self.pl.shift[i + 1]))
            } else {
                self.position = Some((true, i + 1));
                Some((self.pl.borders[i]..self.range.end, self.pl.shift[i + 1]))
            }
        } else {
            let i = self.pl.borders
                .partition_point( |border| *border < self.range.start );
            if i == self.pl.borders.len() || self.pl.borders[i] >= self.range.end {
                self.position = Some((true, i));
                Some((self.range.clone(), self.pl.shift[i]))
            } else {
                self.position = Some((false, i));
                Some((self.range.start..self.pl.borders[i], self.pl.shift[i]))
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
    parse(input, separated_pair(seeds::<i64>, count(line_ending, 2), maps))?
        .pipe( |(seeds, maps)|
            seeds.iter()
                .map( |seed| 
                    maps.iter()
                        .fold(*seed, |value, map| map.apply(value) )
                ).min().unwrap()
        )
        .pipe( |result| Ok(Cow::Owned(result.to_string())) )
}

pub fn part2(input: &str) -> Answer {
    parse(input, separated_pair(seeds::<i64>, count(line_ending, 2), maps))?
        .pipe( |(seeds, maps)|
            maps.into_iter()
                .fold(PiecewiseLinear::new(0),
                    |acc, map| acc.contract(map)
                )
                .pipe( |map|
                    seeds.into_iter()
                        .tuples()
                        .flat_map( |(start, end)| map.range(start..end) )
                        .map( |(range, value)| range.start + value )
                        .min().unwrap()
                )
        )
        .pipe( |result| Ok(Cow::Owned(result.to_string())) )
}

inventory::submit! { Puzzle::new(2023, 5, 1, part1) }
inventory::submit! { Puzzle::new(2023, 5, 2, part2) }

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;
    use itertools::unfold;

    const INPUT1: &str = indoc! {"
        seeds: 79 14 55 13 82

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

    const INPUT2: &str = indoc! {"
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
    const OUTPUT2: &str = "46";

    #[test]
    fn test1() {
        assert_eq!(OUTPUT1, &part1(INPUT1).unwrap());
    }

    #[test]
    fn test2() {
        assert_eq!(OUTPUT2, &part2(INPUT2).unwrap());
    }

    #[test]
    fn apply_chain() {
        let locations = parse(INPUT1, separated_pair(seeds::<i64>, count(line_ending, 2), maps))
        .unwrap()
        .pipe( |(seeds, maps)|
            seeds.iter()
                .map( |seed|
                    once(*seed).chain(unfold((0, *seed), |(i, value)| {
                        if let Some(map) = maps.get(*i) {
                            *i += 1;
                            *value = map.apply(*value);
                            Some(*value)
                        } else { None }
                    } )).collect_vec()
                ).collect_vec()
        );
        assert_eq!(locations, vec![
            vec![79, 81, 81, 81, 74, 78, 78, 82],
            vec![14, 14, 53, 49, 42, 42, 43, 43],
            vec![55, 57, 57, 53, 46, 82, 82, 86],
            vec![13, 13, 52, 41, 34, 34, 35, 35],
            vec![82, 84, 84, 84, 77, 45, 46, 46]
        ])
    }
    
}