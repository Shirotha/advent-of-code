use std::borrow::Cow;
use nom::{
    IResult,
    character::complete::{one_of, multispace0, char, digit1},
    sequence::separated_pair,
    multi::{many1, separated_list1},
    combinator::map_res
};
use rayon::prelude::*;
use smallvec::SmallVec;
use tap::Pipe;

use crate::{*, parse::*, iter::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Spring {
    Operational,
    Damaged,
    Unknown
}
impl Spring {
    #[inline]
    fn from_char(chr: char) -> Result<Self, char> {
        match chr {
            '.' => Ok(Self::Operational),
            '#' => Ok(Self::Damaged),
            '?' => Ok(Self::Unknown),
            _ => Err(chr)
        }
    }
}

type Node = (usize, usize, u8, Option<Spring>);

struct Record {
    data: Vec<Spring>,
    hint: Vec<u8>
}
impl Record {
    fn parse(input: &str) -> IResult<&str, Self> {
        let (input, (data, hint)) = separated_pair(
            many1(map_res(one_of(".#?"), Spring::from_char)),
            multispace0,
            separated_list1(char(','), map_res(digit1, |number: &str| number.parse::<u8>() ))
        )(input)?;
        Ok((input, Self { data, hint }))
    }
    #[inline]
    fn into_solutions(self) -> impl Iterator<Item = Vec<Spring>> {
        const ROOT: Node = (0, 0, 0, None);
        let neighbours = move |(data_i, hint_i, count, _): &Node| {
            let complete_hint = (data_i + 1, hint_i + 1, 0, Some(Spring::Operational));
            let in_group = (data_i + 1, *hint_i, count + 1, Some(Spring::Damaged));
            let between_groups = (data_i + 1, *hint_i, 0, Some(Spring::Operational));
            let deadend = SmallVec::from_buf_and_len([ROOT, ROOT], 0);
            let either = SmallVec::from_buf([in_group, between_groups]);
            let in_group = SmallVec::from_buf_and_len([in_group, ROOT], 1);
            let between_groups = SmallVec::from_buf_and_len([between_groups, ROOT], 1);
            let complete_hint = SmallVec::from_buf_and_len([complete_hint, ROOT], 1);

            let spring = if let Some(current) = self.data.get(*data_i) {
                *current
            } else if *hint_i == self.hint.len() 
                || *hint_i == self.hint.len() - 1 && *count == self.hint[*hint_i]
            {
                return None;
            } else {
                return Some(deadend.into_iter());
            };
            let items = if let Some(&hint) = self.hint.get(*hint_i) {
                if *count == 0 {
                    match spring {
                        Spring::Operational => between_groups,
                        Spring::Damaged => in_group,
                        Spring::Unknown => either
                    }
                } else if *count == hint {
                    if spring == Spring::Damaged { deadend }
                    else { complete_hint }
                } else if spring == Spring::Operational { deadend }
                else { in_group }
            } else if spring == Spring::Damaged { deadend }
            else { between_groups };
            Some(items.into_iter())
        };
        let path_map = #[inline] 
            |(_, _, _, current): &Node| current.unwrap_or(Spring::Unknown);
        PathIter::new(neighbours, path_map, ROOT)
    }
}

pub fn part1(input: &str) -> Answer {
    parse(input, lines(Record::parse))?.into_par_iter()
        .map( |record| record.into_solutions().count() )
        .sum::<usize>()
        .pipe( |result| Ok(Cow::Owned(result.to_string())) )
}

pub fn part2(input: &str) -> Answer {
    todo!()
}

inventory::submit! { Puzzle::new(2023, 12, 1, part1) }
inventory::submit! { Puzzle::new(2023, 12, 2, part2) }

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    const INPUT1: &str = indoc! {"
        ???.### 1,1,3
        .??..??...?##. 1,1,3
        ?#?#?#?#?#?#?#? 1,3,1,6
        ????.#...#... 4,1,1
        ????.######..#####. 1,6,5
        ?###???????? 3,2,1
    "};
    const OUTPUT1: &str = "21";

    const INPUT2: &str = indoc! {"
    
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