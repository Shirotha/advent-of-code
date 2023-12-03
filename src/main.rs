use std::{
    collections::HashMap,
    fmt::Display,
    fs::read_to_string,
    time::Instant
};
use itertools::Itertools;
use tap::{Tap, Pipe};
use clap::{Parser, Subcommand, Args};
use miette::{Result, IntoDiagnostic};
use advent_of_code::*;

fn print_keys<K, V>(data: &HashMap<K, V>)
    where K: Display + Ord
{
    let items = data.keys().collect_vec()
        .tap_mut( |col| col.sort_unstable() );
    for item in items {
        println!("{}", item);
    }
}

#[derive(Debug, Parser)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Command
}
#[derive(Debug, Subcommand)]
enum Command {
    Solve(Solve),
    List(List)
}
impl Command {
    fn run(self, puzzles: &Puzzles) -> Result<()> {
        match self {
            Command::Solve(solve) => solve.solve(puzzles),
            Command::List(list) => list.list(puzzles),
        }
    }
}
#[derive(Debug, Args)]
#[command(arg_required_else_help = true)]
struct Solve {
    year: u16,
    day: u8,
    part: u8,
    input: Option<String>
}
impl Solve {
    fn solve(self, puzzles: &Puzzles) -> Result<()> {
        let puzzle = puzzles.get(&self.year)
            .ok_or_else( || PuzzleError::ArgumentError("invalid year".to_owned(), self.year.to_string()))?
            .get(&self.day)
            .ok_or_else( || PuzzleError::ArgumentError("invalid day".to_owned(), self.day.to_string()))?
            .get(&self.part)
            .ok_or_else( || PuzzleError::ArgumentError("invalid part".to_owned(), self.day.to_string()))?;
        let input = self.input
            .unwrap_or_else( || default_input_file("./src/puzzles", self.year, self.day, self.part) )
            .pipe(read_to_string).into_diagnostic()?;
        let timer = Instant::now();
        let result = puzzle.solve(&input)?;
        let duration = timer.elapsed();
        println!("solved in {:.2?}    result: {}", duration, result);
        Ok(())
    }
}
#[derive(Debug, Args)]
struct List {
    year: Option<u16>,
    day: Option<u8>
}
impl List {
    fn list(self, puzzles: &Puzzles) -> Result<()> {
        if let Some(year) = self.year {
            let puzzles = puzzles.get(&year)
                .ok_or_else( || PuzzleError::ArgumentError("no entries for year".to_owned(), year.to_string()))?;
            if let Some(day) = self.day {
                let puzzles = puzzles.get(&day)
                    .ok_or_else( || PuzzleError::ArgumentError("no entries for year".to_owned(), day.to_string()))?;
                print_keys(puzzles);
            } else { print_keys(puzzles); }
        } else { print_keys(puzzles); }
        Ok(())
    }
}

fn main() -> Result<()> {
    let puzzles = HashMap::new().tap_mut(
        |puzzles: &mut Puzzles|
        {
            for puzzle in inventory::iter::<Puzzle> {
                puzzles.entry(puzzle.year())
                    .or_default()
                    .entry(puzzle.day())
                    .or_default()
                    .insert(puzzle.part(), puzzle);
            }
        });
    Cli::parse().command.run(&puzzles)
}