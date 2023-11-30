use std::{fs, io};

#[derive(Debug)]
pub struct Puzzle {
    year: u16,
    day: u8,
    part: u8,
    solution: fn(String) -> String
}

impl Puzzle {
    pub const fn new(year: u16, day: u8, part: u8, solution: fn(String) -> String) -> Self {
        Puzzle { year, day, part, solution }
    }
    pub fn solve(&self, input: String) -> String { (self.solution)(input) }
    fn default_input_file(&self, root: &str) -> String {
        format!("{}/year_{}/day_{}/part_{}.txt", root, self.year, self.day, self.part)
    }
    pub fn solve_default(&self, root: &str) -> io::Result<String> {
        let input = fs::read_to_string(self.default_input_file(root))?;
        Ok(self.solve(input))
    }
    #[inline(always)] pub const fn year(&self) -> u16 { self.year }
    #[inline(always)] pub const fn day(&self) -> u8 { self.day }
    #[inline(always)] pub const fn part(&self) -> u8 { self.part }
}

inventory::collect!(Puzzle);