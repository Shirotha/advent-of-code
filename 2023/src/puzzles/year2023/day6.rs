use std::{
    borrow::Cow,
};
use glam::{DVec4, U64Vec4};
use itertools::Itertools;
use nom::{
    IResult,
    bytes::complete::tag,
    character::complete::{char, line_ending, digit1},
    sequence::{separated_pair, preceded, pair},
    multi::{many0, separated_list1, many1}, combinator::map_res
};
use tap::Pipe;

use crate::{*, parse::*};

struct Param {
    time: DVec4,
    distance: DVec4
}
impl Param {
    fn parse1(input: &str) -> IResult<&str, Self> {
        let (input, (mut time, mut distance)) = separated_pair(
            preceded(
                pair(tag("Time:"), many0(char(' '))),
                separated_list1(many1(char(' ')), 
                    map_res(digit1, |number: &str| number.parse::<f64>())
                )
            ),
            line_ending,
            preceded(
                pair(tag("Distance:"), many0(char(' '))),
                separated_list1(many1(char(' ')), 
                    map_res(digit1, |number: &str| number.parse::<f64>())
                )
            ),
        )(input)?;
        time.resize(4, 0.0);
        distance.resize(4, 0.0);
        Ok((input, Self { 
            time: DVec4::from_slice(&time),
            distance: DVec4::from_slice(&distance)
        }))
    }
    fn parse2(input: &str) -> IResult<&str, Self> {
        let (input, (time, distance)) = separated_pair(
            preceded(
                pair(tag("Time:"), many0(char(' '))),
                separated_list1(many1(char(' ')), digit1)
            ),
            line_ending,
            preceded(
                pair(tag("Distance:"), many0(char(' '))),
                separated_list1(many1(char(' ')), digit1)
                )
            )(input)?;
        Ok((input, Self { 
            time: DVec4::splat(time.join("").parse::<f64>().unwrap()),
            distance: DVec4::splat(distance.join("").parse::<f64>().unwrap())
        }))
    }
    #[inline]
    fn count(&self) -> U64Vec4 {
        const EPSILON: f64 = 1e-3;
        let (a, d) = (0.5 * self.time, self.distance);
        let a2 = a * a;
        let b = (a2 - d).powf(0.5);
        ((a + b - EPSILON).ceil() - (a - b + EPSILON).ceil()).as_u64vec4()
    }
}

#[inline]
fn prod(v: U64Vec4) -> u64 {
    v.to_array().iter().take_while( |x| **x != 0).product1().unwrap()
}

pub fn part1(input: &str) -> Answer {
    parse(input, Param::parse1)?.count()
        .pipe(prod)
        .pipe( |result| Ok(Cow::Owned(result.to_string())) )
}

pub fn part2(input: &str) -> Answer {
    parse(input, Param::parse2)?.count().x
        .pipe( |result| Ok(Cow::Owned(result.to_string())) )
}

inventory::submit! { Puzzle::new(2023, 6, 1, part1) }
inventory::submit! { Puzzle::new(2023, 6, 2, part2) }

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    const INPUT1: &str = indoc! {"
        Time:      7  15   30
        Distance:  9  40  200
    "};
    const OUTPUT1: &str = "288";

    const INPUT2: &str = indoc! {"
        Time:      7  1 5 30
        Distance:  940   2 00
    "};
    const OUTPUT2: &str = "71503";

    #[test]
    fn test1() {
        assert_eq!(OUTPUT1, &part1(INPUT1).unwrap());
    }

    #[test]
    fn test2() {
        assert_eq!(OUTPUT2, &part2(INPUT2).unwrap());
    }
}