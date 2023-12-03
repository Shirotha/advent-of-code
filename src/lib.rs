#![feature(iterator_try_collect)]

pub mod puzzles;
mod parse;

use std::borrow::Cow;
use thiserror::Error;
use miette::Diagnostic;

#[derive(Debug, Error, Diagnostic)]
pub enum PuzzleError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("parsing failed")]
    ParseError(#[from] nom::Err<nom::error::Error<String>>),
    #[error("bad command line argument: {1:} ({0:})")]
    ArgumentError(String, String)
}

pub type Answer<'a> = Result<Cow<'a, str>, PuzzleError>;
pub type Solver = for <'a> fn(&'a str) -> Answer<'a>;

#[inline]
pub fn default_input_file(root: &str, year: u16, day: u8, part: u8) -> String {
    format!("{}/year{}/day{}/part{}.txt", root, year, day, part)
}

#[derive(Debug)]
pub struct Puzzle {
    year: u16,
    day: u8,
    part: u8,
    solution: Solver
}

impl Puzzle {
    pub const fn new(year: u16, day: u8, part: u8, solution: Solver) -> Self {
        Puzzle { year, day, part, solution }
    }
    #[inline(always)] pub const fn year(&self) -> u16 { self.year }
    #[inline(always)] pub const fn day(&self) -> u8 { self.day }
    #[inline(always)] pub const fn part(&self) -> u8 { self.part }
    
    pub fn solve<'a>(&self, input: &'a str) -> Answer<'a> { (self.solution)(input) }
}

inventory::collect!(Puzzle);