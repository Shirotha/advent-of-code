use std::str::FromStr;

#[derive(Debug)]
pub struct Input {
    // TODO: define input here
}
impl FromStr for Input {
    type Err = nom::Err<nom::error::Error<String>>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!("parse input here")
    }
}
