use std::str::FromStr;

use nom::{
    IResult,
    bytes::complete::{tag, take_while_m_n},
    character::complete::{anychar, char},
    combinator::{map, map_res, verify},
    multi::{many_till, many0, separated_list0},
    sequence::{delimited, preceded},
};

#[derive(Debug)]
pub struct Input {
    pub matches: Box<[Box<[u32]>]>,
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
        fn mul(input: &str) -> IResult<&str, Box<[u32]>> {
            verify(preceded(tag("mul"), args), |args: &[u32]| args.len() == 2)(input)
        }
        // TODO: do this without storing the anychar result
        let (_, matches) = many0(map(many_till(anychar, mul), |(_, result)| result))(s)
            .map_err(|e| e.to_owned())?;
        Ok(Input {
            matches: matches.into_boxed_slice(),
        })
    }
}
