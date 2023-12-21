use std::borrow::Cow;
use nom::{
    IResult,
    character::complete::char,
    multi::separated_list1,
    bytes::complete::take_until
};
use rayon::prelude::*;
use tap::Pipe;

use crate::{*, parse::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Command<'a> {
    code: &'a str
}
impl<'a> Command<'a> {
    #[inline]
    fn parse<E>(input: &'a str) -> IResult<&'a str, Self, E>
        where E: nom::error::ParseError<&'a str>
    {
        take_until(",")(input).map_or_else(
            |_: nom::Err<E>| ("", Command { code: input }),
            |(input, code)| (input, Command { code })
        ).pipe(Ok)
    }
    #[inline]
    fn hash(&self) -> u8 {
        self.code.as_bytes().iter()
            .fold(0, |acc, byte| acc.wrapping_add(*byte).wrapping_mul(17) )
    }
}

pub fn part1(input: &str) -> Answer {
    parse(input, separated_list1(char(','), Command::parse))?.into_par_iter()
        .map( |command| command.hash() as u32 )
        .sum::<u32>()
        .pipe( |result| Ok(Cow::Owned(result.to_string())) )
}

pub fn part2(input: &str) -> Answer {
    todo!()
}

inventory::submit! { Puzzle::new(2023, 15, 1, part1) }
inventory::submit! { Puzzle::new(2023, 15, 2, part2) }

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    const INPUT1: &str = "rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7";
    const OUTPUT1: &str = "1320";

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