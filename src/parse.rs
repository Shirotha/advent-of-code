use nom::{
    IResult, Parser,
    error::ParseError,
    InputTake
};
use crate::*;

pub fn parse<T, F>(input: &str, parser: F) -> Result<T, PuzzleError> 
    where F: FnOnce(&str) -> nom::IResult<&str, T>
{
    let (_, result) = parser(input)
        .map_err( |err| PuzzleError::ParseError(err.to_owned()) )?;
    Ok(result)
}

#[inline]
pub fn many_overlapping_till<I, O1, O2, E, F, G>(f: F, g: G) -> impl FnMut(I) -> IResult<I, Vec<O1>>
where
    I: Clone + InputTake,
    F: Clone + Parser<I, O1, E>,
    G: Clone + Parser<I, O2, E>,
    E: ParseError<I>,
{
    move |mut input| {
        let mut f = f.clone();
        let mut g = g.clone();
        let mut result = Vec::new();
        loop {
            if let Ok((_, digit)) = f.parse(input.clone()) { result.push(digit); }
            match g.parse(input.clone()) {
                Ok(_) => return Ok((input, result)),
                Err(_) => (input, _) = input.take_split(1)
            }
        }
    }
}