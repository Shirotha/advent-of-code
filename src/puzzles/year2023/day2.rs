use std::{
    borrow::Cow,
    cmp::Ordering, ops::AddAssign
};
use nom::{
    IResult,
    bytes::complete::tag,
    character::complete::{char, digit1},
    branch::alt,
    multi::separated_list1,
    sequence::{delimited, tuple},
    combinator::{map_res, value, map}
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tap::Pipe;

use crate::{*, parse::*};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Set {
    red: u8,
    green: u8,
    blue: u8
}
impl Set {
    #[inline]
    const fn new(red: u8, green: u8, blue: u8) -> Self {
        Set { red, green, blue }
    }
    #[inline]
    const fn empty() -> Self {
        Set::new(0, 0, 0)
    }
    fn parse(input: &str) -> IResult<&str, Self> {
        map(
            separated_list1(tag(", "), Self::parse_unit),
            |units| units.into_iter().sum()
        )(input)
    }
    fn parse_unit(input: &str) -> IResult<&str, Self> {
        let (input, count) = map_res(
            digit1,
            |digit: &str| digit.parse::<u8>()
        )(input)?;
        let (input, _) = char(' ')(input)?;
        alt((
            value( Self::new(count, 0, 0), tag("red")),
            value( Self::new(0, count, 0), tag("green")),
            value( Self::new(0, 0, count), tag("blue")),
        ))(input)
    }
    #[inline]
    fn max(&self, other: &Self) -> Self {
        Self::new(self.red.max(other.red), self.green.max(other.green), self.blue.max(other.blue))
    }
    #[inline]
    fn power(&self) -> u32 {
        (self.red as u32) * (self.green as u32) * (self.blue as u32)
    }
}
impl PartialOrd for Set {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other { Some(Ordering::Equal) }
        else if self.red <= other.red && self.green <= other.green && self.blue <= other.blue { Some(Ordering::Less) }
        else if self.red >= other.red && self.green >= other.green && self.blue >= other.blue { Some(Ordering::Greater) }
        else { None }
    }
}
impl std::iter::Sum for Set {
    #[inline]
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut sum = Set::empty();
        iter.for_each( |set| sum += set );
        sum
    }
}
impl AddAssign for Set {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.red += rhs.red;
        self.green += rhs.green;
        self.blue += rhs.blue;
    }
}

#[derive(Debug)]
struct Game {
    id: u32,
    subsets: Vec<Set>
}
impl Game {
    #[inline]
    fn new(id: u32, subsets: Vec<Set>) -> Self { Game { id, subsets } }
    fn parse(input: &str) -> IResult<&str, Self> {
        let (input, (id, subsets)) = tuple((
            delimited(tag("Game "), map_res(
                digit1,
                |digit: &str| digit.parse::<u32>()
            ), tag(": ")),
            separated_list1(tag("; "), Set::parse)
        ))(input)?;
        Ok((input, Self::new(id, subsets)))
    }
    #[inline]
    fn is_valid(&self, limit: &Set) -> bool {
        self.subsets.iter().all( |ss| ss <= limit )
    }
    #[inline]
    fn power(&self) -> u32 {
        self.subsets.iter()
            .fold(Set::empty(), |a, b| a.max(b))
            .pipe( |limit| limit.power() )
    }
}

pub fn part1(input: &str) -> Answer {
    const MAX: Set = Set::new(12, 13, 14);
    parse(input, lines(Game::parse))?.par_iter()
        .filter_map( |game| if game.is_valid(&MAX) { Some(game.id) } else { None } )
        .sum::<u32>()
        .pipe( |result| Ok(Cow::Owned(result.to_string())) )
}

pub fn part2(input: &str) -> Answer {
    parse(input, lines(Game::parse))?.par_iter()
        .map( |game| game.power() )
        .sum::<u32>()
        .pipe( |result| Ok(Cow::Owned(result.to_string())) )
}

inventory::submit! { Puzzle::new(2023, 2, 1, part1) }
inventory::submit! { Puzzle::new(2023, 2, 2, part2) }

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    const INPUT1: &str = indoc! {"
        Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green
        Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue
        Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red
        Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red
        Game 5: 6 red, 1 blue, 3 green; 2 blue, 1 red, 2 green
    "};
    const OUTPUT1: &str = "8";

    const INPUT2: &str = indoc!{"
        Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green
        Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue
        Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red
        Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red
        Game 5: 6 red, 1 blue, 3 green; 2 blue, 1 red, 2 green
    "};
    const OUTPUT2: &str = "2286";

    #[test]
    fn test1() {
        assert_eq!(OUTPUT1, &part1(INPUT1).unwrap());
    }

    #[test]
    fn test2() {
        assert_eq!(OUTPUT2, &part2(INPUT2).unwrap());
    }
}