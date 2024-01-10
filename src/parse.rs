use nom::{
    IResult, Parser,
    error::{Error, ParseError},
    InputTake,
    InputLength
};
use ndarray::prelude::*;
use rayon::prelude::*;
use smallvec::SmallVec;
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
pub fn many_overlapping_till<const N: usize, I, O1, O2, E, F, G>(f: F, g: G) -> impl FnMut(I) -> IResult<I, SmallVec<[O1; N]>>
where
    I: Clone + InputTake,
    F: Clone + Parser<I, O1, E>,
    G: Clone + Parser<I, O2, E>,
    E: ParseError<I>,
{
    move |mut input| {
        let (mut f, mut g, mut result) = (f.clone(), g.clone(), SmallVec::new());
        loop {
            if let Ok((_, item)) = f.parse(input.clone()) { result.push(item); }
            match g.parse(input.clone()) {
                Ok(_) => return Ok((input, result)),
                Err(_) => (input, _) = input.take_split(1)
            }
        }
    }
}

#[inline]
pub fn find_many_till<I, O1, O2, E, F, G>(f: F, g: G) -> impl FnMut(I) -> IResult<I, Vec<(usize, O1)>>
where
    I: Clone + InputTake + InputLength,
    F: Clone + Parser<I, O1, E>,
    G: Clone + Parser<I, O2, E>,
    E: ParseError<I>,
{
    move |mut input| {
        let (mut f, mut g, mut i, mut result) = (f.clone(), g.clone(), 0, Vec::new());
        loop {
            if let Ok((rest, item)) = f.parse(input.clone()) {
                result.push((i, item));
                i += input.input_len() - rest.input_len();
                input = rest;
            }
            else {
                match g.parse(input.clone()) {
                    Ok(_) => return Ok((input, result)),
                    Err(_) => {
                        (input, _) = input.take_split(1);
                        i += 1;
                    }
                }
            }
        }
    }
}

pub fn grid<T, F>(f: &mut F) -> impl '_ + FnMut(&str) -> IResult<&str, Array2<T>>
    where F: FnMut(char) -> T
{
    |mut input: &str| {
        let mut buffer = Vec::new();
        let (mut current, mut width, mut height, mut last) = (0, None, 0, 0);
        for (i, chr) in input.chars().enumerate() {
            match chr {
                '\n' => {
                    if let Some(width) = width {
                        if width != current {
                            break;
                        }
                    } else {
                        width = Some(current);
                    }
                    current = 0;
                    height += 1;
                    last = i;
                }
                '\r' => (),
                _ => {
                    buffer.push(f(chr));
                    current += 1;
                }
            }
        }
        let width = width.unwrap_or(current);
        if width == 0 {
            return Err(nom::Err::Incomplete(nom::Needed::Unknown));
        }
        if current == width {
            height += 1;
            input = "";
        } else {
            input = &input[last..];
        }
        let len = width * height;
        buffer.truncate(len);
        let data = Array2::from_shape_vec((width, height).f(), buffer).unwrap();
        Ok((input, data))
    }
}