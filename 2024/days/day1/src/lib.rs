use nom::{
    IResult,
    character::complete::{digit1, line_ending, multispace1},
    combinator::map_res,
    multi::separated_list0,
    sequence::separated_pair,
};
use std::str::FromStr;

#[derive(Debug)]
pub struct Input {
    pub left: Box<[u32]>,
    pub right: Box<[u32]>,
}
impl FromStr for Input {
    type Err = nom::Err<nom::error::Error<String>>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn number(input: &str) -> IResult<&str, u32> {
            map_res(digit1, |n: &str| n.parse::<u32>())(input)
        }
        let (_, items) =
            separated_list0(line_ending, separated_pair(number, multispace1, number))(s)
                .map_err(|e| e.to_owned())?;
        let (left, right): (Vec<_>, Vec<_>) = Iterator::unzip(items.into_iter());
        Ok(Input {
            left: left.into_boxed_slice(),
            right: right.into_boxed_slice(),
        })
    }
}
