#![feature(
    maybe_uninit_uninit_array,
    maybe_uninit_array_assume_init,
    const_maybe_uninit_write
)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

mod range;
pub use range::*;
mod narray;
pub use narray::*;

use std::io::{Read, stdin};

pub type Error = Box<dyn std::error::Error>;
pub type DResult<T> = Result<T, Error>;

pub fn get_input() -> std::io::Result<String> {
    let mut buffer = String::new();
    stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}
