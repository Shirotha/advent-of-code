#![feature(iterator_try_collect)]
#![feature(float_next_up_down)]
#![feature(portable_simd)]
#![feature(const_option)]
#![feature(maybe_uninit_uninit_array)]
#![feature(maybe_uninit_array_assume_init)]
#![feature(const_trait_impl)]
#![feature(type_alias_impl_trait)]
#![feature(stmt_expr_attributes)]
#![feature(array_windows)]
#![feature(allocator_api)]
#![feature(alloc_layout_extra)]
#![feature(non_null_convenience)]
#![feature(get_many_mut)]
#![feature(linked_list_cursors)]

pub mod collections;
pub mod puzzles;
mod parse;
mod iter;

use std::{
    borrow::Cow,
    collections::HashMap,
};
#[allow(unused_imports)]
use std::ops::{Deref, DerefMut};
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

pub type Puzzles<'a> = HashMap<u16, HashMap<u8, HashMap<u8, &'a Puzzle>>>;

#[allow(unused_macros)]
macro_rules! alias {
    { $name:ident = $rhs:ty } => {
        struct $name($rhs);
        impl const From<$rhs> for $name {
            #[inline(always)]
            fn from(value: $rhs) -> Self { $name(value) }
        }
        impl const Deref for $name {
            type Target = $rhs;
            #[inline(always)]
            fn deref(&self) -> &Self::Target { &self.0 }
        }
        impl const DerefMut for $name {
            #[inline(always)]
            fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
        }
    };
    { $name:ident < $( $params:tt ),+ > = $rhs:ty } => {
        struct $name< $( $params ),* >($rhs);
        impl< $( $params ),* > const From<$rhs> for $name< $( $params ),* > {
            #[inline(always)]
            fn from(value: $rhs) -> Self { $name(value) }
        }
        impl< $( $params ),* > const Deref for $name< $( $params ),* > {
            type Target = $rhs;
            #[inline(always)]
            fn deref(&self) -> &Self::Target { &self.0 }
        }
        impl< $( $params ),* > const DerefMut for $name< $( $params ),* > {
            #[inline(always)]
            fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
        }
    }
}
#[allow(unused_imports)]
pub(crate) use alias;

inventory::collect!(Puzzle);