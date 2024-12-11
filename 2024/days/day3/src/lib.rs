use std::str::FromStr;

use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, take_while_m_n},
    character::complete::{anychar, char},
    combinator::{map, map_res, value, verify},
    multi::{many_till, many0, separated_list0},
    sequence::{delimited, pair, preceded},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    Mul(u32, u32),
    Do,
    Dont,
}
impl Instruction {
    pub fn into_mul(self) -> Option<(u32, u32)> {
        if let Self::Mul(a, b) = self {
            Some((a, b))
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Input {
    pub instructions: Box<[Instruction]>,
}
impl FromStr for Input {
    type Err = nom::Err<nom::error::Error<String>>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn number(input: &str) -> IResult<&str, u32> {
            map_res(
                take_while_m_n(1, 3, |c: char| c.is_ascii_digit()),
                |n: &str| n.parse::<u32>(),
            )(input)
        }
        fn args(input: &str) -> IResult<&str, Box<[u32]>> {
            map(
                delimited(char('('), separated_list0(char(','), number), char(')')),
                Vec::into_boxed_slice,
            )(input)
        }
        fn mul(input: &str) -> IResult<&str, Instruction> {
            map_res(preceded(tag("mul"), args), |args: Box<[u32]>| {
                if args.len() == 2 {
                    Ok(Instruction::Mul(args[0], args[1]))
                } else {
                    Err("wrong number of arguments")
                }
            })(input)
        }
        fn r#do(input: &str) -> IResult<&str, Instruction> {
            value(
                Instruction::Do,
                pair(tag("do"), verify(args, |args: &[u32]| args.is_empty())),
            )(input)
        }
        fn r#dont(input: &str) -> IResult<&str, Instruction> {
            value(
                Instruction::Dont,
                pair(tag("don't"), verify(args, |args: &[u32]| args.is_empty())),
            )(input)
        }
        // TODO: do this without storing the anychar result
        let (_, instructions) = many0(map(
            many_till(anychar, alt((mul, r#do, dont))),
            |(_, result)| result,
        ))(s)
        .map_err(|e| e.to_owned())?;
        Ok(Input {
            instructions: instructions.into_boxed_slice(),
        })
    }
}
