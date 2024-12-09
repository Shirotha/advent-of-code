use std::str::FromStr;

use nom::{
    IResult,
    character::complete::{digit1, line_ending, multispace1},
    combinator::map_res,
    multi::{separated_list0, separated_list1},
};

#[derive(Debug)]
pub struct Input {
    pub lines: Box<[Box<[u32]>]>,
}
impl FromStr for Input {
    type Err = nom::Err<nom::error::Error<String>>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn number(input: &str) -> IResult<&str, u32> {
            map_res(digit1, |n: &str| n.parse::<u32>())(input)
        }
        let (_, data) = separated_list0(line_ending, separated_list1(multispace1, number))(s)
            .map_err(|e| e.to_owned())?;
        Ok(Input {
            lines: data.into_iter().map(Vec::into_boxed_slice).collect(),
        })
    }
}
