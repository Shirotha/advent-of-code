use nom::{
    IResult, Parser,
    error::{Error, ParseError},
    InputTake
};
use rayon::prelude::*;
use crate::*;

pub fn parse<'a, O, F>(input: &'a str, mut f: F) -> Result<O, PuzzleError> 
    where F: Parser<&'a str, O, Error<&'a str>>
{
    let (_, result) = f.parse(input)
        .map_err( |err| PuzzleError::ParseError(err.to_owned()) )?;
    Ok(result)
}

#[inline]
pub fn lines<'a, O: Send, E, F>(f: F) -> impl FnMut(&'a str) -> IResult<&'a str, Vec<O>, E>
where
    F: Send + Sync + Clone + Parser<&'a str, O, E>,
    E: Send + ParseError<&'a str>
{
    move |input| {
        let f = f.clone();
        let x = input.par_lines()
            .map( move |input| f.clone().parse(input).map( |(_, digit)| digit ) )
            .collect::<Result<Vec<_>, _>>()?;
        Ok((input, x))
    }
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
        let (mut f, mut g, mut result) = (f.clone(), g.clone(), Vec::new());
        loop {
            if let Ok((_, digit)) = f.parse(input.clone()) { result.push(digit); }
            match g.parse(input.clone()) {
                Ok(_) => return Ok((input, result)),
                Err(_) => (input, _) = input.take_split(1)
            }
        }
    }
}