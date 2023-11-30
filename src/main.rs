mod puzzle;
mod puzzles;

use std::error::Error;
use std::collections::HashMap;
use std::fmt::Display;

use dialoguer::{theme::ColorfulTheme, Select};

use puzzle::Puzzle;

fn select_key<'a, K, V>(prompt: &str, options: &'a HashMap<K, V>) -> Result<&'a K, Box<dyn Error>>
    where K: Display + Ord
{
    let mut items = options.keys().collect::<Vec<_>>();
    items.sort_unstable();
    match items.len() {
        0 => Err("no valid options")?,
        1 => {
            let key = items[0];
            println!("{} {}", prompt, key);
            Ok(key)
        },
        _ => {
            let index = Select::with_theme(&ColorfulTheme::default())
                .with_prompt(prompt)
                .items(&items)
                .default(items.len() - 1)
                .interact()?;
            Ok(items[index])
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut puzzles = HashMap::new();
    for puzzle in inventory::iter::<Puzzle> {
        puzzles.entry(puzzle.year())
            .or_insert_with(HashMap::new)
            .entry(puzzle.day())
            .or_insert_with(HashMap::new)
            .insert(puzzle.part(), puzzle);
    }
    
    let year = select_key("year", &puzzles)?;
    let puzzles = puzzles.get(year).unwrap();
    let day = select_key("day", puzzles)?;
    let puzzles = puzzles.get(day).unwrap();
    let part = select_key("part", puzzles)?;
    let puzzle = puzzles.get(part).unwrap();
    let result = puzzle.solve_default("./src/puzzles")?;
    println!("result: {}", result);
    Ok(())
}