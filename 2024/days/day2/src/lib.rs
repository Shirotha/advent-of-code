use std::str::FromStr;

use nom::{
    IResult,
    character::complete::{digit1, line_ending, space1},
    combinator::map_res,
    multi::{separated_list0, separated_list1},
};

#[derive(Debug)]
pub struct Input {
    pub lines: Box<[Box<[i32]>]>,
}
impl FromStr for Input {
    type Err = nom::Err<nom::error::Error<String>>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn number(input: &str) -> IResult<&str, i32> {
            map_res(digit1, |n: &str| n.parse::<i32>())(input)
        }
        let (_, data) = separated_list0(line_ending, separated_list1(space1, number))(s)
            .map_err(|e| e.to_owned())?;
        Ok(Input {
            lines: data.into_iter().map(Vec::into_boxed_slice).collect(),
        })
    }
}
