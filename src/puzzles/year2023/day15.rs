use std::{
    borrow::Cow,
    collections::LinkedList,
    mem::MaybeUninit
};
use nom::{
    IResult,
    character::complete::{char, alpha1, digit1},
    multi::separated_list1,
    bytes::complete::take_until,
    branch::alt,
    sequence::{separated_pair, terminated},
    combinator::map
};
use rayon::prelude::*;
use tap::Pipe;

use crate::{*, parse::*};

#[inline(always)]
const fn hash_byte(input: u8, byte: u8) -> u8 {
    input.wrapping_add(byte).wrapping_mul(17)
}

trait Hash {
    fn hash(self, hash: u8) -> u8;
}
impl Hash for u8 {
    #[inline(always)]
    fn hash(self, hash: u8) -> u8 {
        hash_byte(hash, self)
    }
}
impl Hash for char {
    #[inline(always)]
    fn hash(self, hash: u8) -> u8 {
        (self as u8).hash(hash)
    }
}
impl Hash for &str {
    #[inline]
    fn hash(self, hash: u8) -> u8 {
        self.as_bytes().iter().copied().fold(hash, hash_byte)
    }
}
macro_rules! hash {
    ( ; ) => {
        0
    };
    ( ; $head:expr $( , $tail:expr )* ) => {
        $head.hash(hash!( ; $( $tail ),* ))
    };
    ( $head:expr $( , $tail:expr )* ; $( $rest:expr ),+ ) => {
        hash!( $( $tail ),* ; $head, $( $rest ),* )
    };
    ( $head:expr $( , $tail:expr )* ) => {
        hash!( $( $tail ),* ; $head )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Command<'a> {
    Set(&'a str, u8),
    Remove(&'a str)
}
impl<'a> Command<'a> {
    #[inline]
    fn parse<E>(input: &'a str) -> IResult<&'a str, Self, E>
        where E: nom::error::ParseError<&'a str>
    {
        let (input, code) = take_until(",")(input).map_or_else(
            |_: nom::Err<E>| ("", input),
            |(input, code)| (input, code)
        );
        let (_, cmd) = alt((
            map(
                separated_pair(
                    alpha1,
                    char('='),
                    map(digit1, |number: &str| number.parse::<u8>().unwrap() )
                ),
                |(label, value)| Self::Set(label, value)
                ),
            map(
                terminated(alpha1, char('-')),
                Self::Remove
            )
        ))(code)?;
        Ok((input, cmd))
    }
    #[inline]
    fn hash(&self) -> u8 {
        match self {
            Self::Set(label, value) => hash!(label, '=', b'0' + value),
            Self::Remove(label) => hash!(label, '-')
        }
    }
    fn run(cmds: &[Self]) -> usize {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        struct Node<'a>(&'a str, u8);
        fn create_boxes<'a>() -> [LinkedList<Node<'a>>; 256] {
            unsafe {
                let mut raw = MaybeUninit::uninit_array();
                for elem in &mut raw {
                    elem.write(LinkedList::new());
                }
                MaybeUninit::array_assume_init(raw)
            }
        }

        let mut boxes = create_boxes();
        for cmd in cmds {
            match cmd {
                Self::Remove(label) => {
                    // TODO: remove if label exists
                },
                Self::Set(label, value) => {
                    let r#box = &mut boxes[label.hash(0) as usize];
                    // TODO: check if label already exists
                    r#box.push_back(Node(label, *value));
                }
            }
        }
        let mut result = 0;
        for (i, r#box) in boxes.into_iter().enumerate() {
            for (j, node) in r#box.into_iter().enumerate() {
                result += (i + 1) * (j + 1) * node.1 as usize;
            }
        }
        result
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

    const INPUT1: &str = "rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7";
    const OUTPUT1: &str = "1320";

    const INPUT2: &str = "rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7";
    const OUTPUT2: &str = "145";

    #[test]
    fn test1() {
        assert_eq!(OUTPUT1, &part1(INPUT1).unwrap());
    }

    #[test]
    fn test2() {
        assert_eq!(OUTPUT2, &part2(INPUT2).unwrap());
    }
}