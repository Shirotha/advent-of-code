use std::{
    borrow::Cow,
};
use tap::{Tap, Pipe};

use crate::{*, parse::*, /*collections::rangemap::RangeMap*/};

pub fn part1(input: &str) -> Answer {
    todo!()
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