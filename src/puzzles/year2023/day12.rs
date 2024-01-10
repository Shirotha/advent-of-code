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

type Node = (usize, usize, usize);
type Branch = smallvec::IntoIter<[Node; 2]>;
type Solutions = DFSIter<Node, Branch, impl FnMut(&Node) -> Option<Branch>>;

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
    fn unfold(mut self, factor: usize) -> Self {
        self.data.push(Spring::Unknown);
        let mut data = self.data.repeat(factor);
        data.pop();
        let hint = self.hint.repeat(factor);
        Self { data, hint }
    }
    #[inline]
    fn into_solutions(self) -> Solutions {
        let reserved = self.hint.iter().sum::<u8>() as usize + self.hint.len() - 1;
        let root = (0, 0, self.data.len() - reserved);
        let neighbours = move |(data_i, hint_i, last): &Node| {
            let (children, len) = if data_i > last {
                ([root, root], 0)
            } else if let Some(&hint) = self.hint.get(*hint_i) {
                let (mut data_i, hint) = (*data_i, hint as usize);
                while let Some(&spring) = self.data.get(data_i) {
                    if spring == Spring::Operational {
                        data_i += 1;
                    } else {
                        break;
                    }
                }
                if let Some(&spring) = self.data.get(data_i) {
                    if (hint == 1 || self.data.iter().skip(data_i).take(hint)
                            .all( |spring| *spring != Spring::Operational )
                        ) && !self.data.get(data_i + hint)
                            .is_some_and( |spring| *spring == Spring::Damaged )
                    {
                        if spring == Spring::Unknown {
                            ([(data_i + hint + 1, hint_i + 1, last + hint + 1), (data_i + 1, *hint_i, *last)], 2)
                        } else {
                            ([(data_i + hint + 1, hint_i + 1, last + hint + 1), root], 1)
                        }
                    } else if spring == Spring::Unknown {
                        ([(data_i + 1, *hint_i, *last), root], 1)
                    } else {
                        ([root, root], 0)
                    }
                } else {
                    ([root, root], 0)
                }
            } else if self.data.iter().skip(*data_i)
                .all( |spring| *spring != Spring::Damaged )
            {
                return None;
            } else { ([root, root], 0) };
            Some(SmallVec::from_buf_and_len(children, len).into_iter())
        };
        DFSIter::new(neighbours, root)
    }
}

pub fn part1(input: &str) -> Answer {
    parse(input, lines(Record::parse))?.into_par_iter()
        .map( |record| record.into_solutions().count_leaves::<u16>() )
        .sum::<u16>()
        .pipe( |result| Ok(Cow::Owned(result.to_string())) )
}

pub fn part2(input: &str) -> Answer {
    parse(input, lines(Record::parse))?.into_par_iter().enumerate()
        .map( |(row, record)| {
            let result = record.unfold(5).into_solutions().count_leaves::<u64>();
            println!("row {}: {}", row + 1, result);
            result
        } )
        .sum::<u64>()
        .pipe( |result| Ok(Cow::Owned(result.to_string())) )
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
        ???.### 1,1,3
        .??..??...?##. 1,1,3
        ?#?#?#?#?#?#?#? 1,3,1,6
        ????.#...#... 4,1,1
        ????.######..#####. 1,6,5
        ?###???????? 3,2,1
    "};
    const OUTPUT2: &str = "525152";

    #[test]
    fn test1() {
        assert_eq!(OUTPUT1, &part1(INPUT1).unwrap());
    }

    #[test]
    fn test2() {
        assert_eq!(OUTPUT2, &part2(INPUT2).unwrap());
    }
}