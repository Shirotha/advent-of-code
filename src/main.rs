use std::{
    error::Error,
    collections::HashMap,
    fmt::Display,
    fs::read_to_string,
    time::Instant,
    env::args
};
use dialoguer::{theme::ColorfulTheme, Select};
use advent_of_code::{Puzzle, default_input_file};

fn select_key<'a, K, V>(prompt: &str, options: &'a HashMap<K, V>) -> Result<&'a K, Box<dyn Error>>
    where K: Display + Ord
{
    let mut items = options.keys().collect::<Vec<_>>();
    match options.len() {
        0 => Err("no valid options")?,
        1 => {
            let key = options.keys().next().unwrap();
            println!("{} {}", prompt, key);
            Ok(key)
        },
        _ => {
            items.sort_unstable();
            let index = Select::with_theme(&ColorfulTheme::default())
                .with_prompt(prompt)
                .items(&items)
                .default(items.len() - 1)
                .interact()?;
            Ok(items[index])
        }
    }
}

fn print_keys<K, V>(data: &HashMap<K, V>)
    where K: Display + Ord
{
    let mut items = data.keys().collect::<Vec<_>>();
    items.sort_unstable();
    for item in items {
        println!("{}", item);
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
    let mut args = args().skip(1);
    match args.next().as_deref() {
        Some("solve") => {
            let year = if let Some(year) = args.next() { year.parse::<u16>()? } 
                else { *select_key("year", &puzzles)? };
            let puzzles = puzzles.get(&year).unwrap();
            let day = if let Some(day) = args.next() { day.parse::<u8>()? }
                else { *select_key("day", puzzles)? };
            let puzzles = puzzles.get(&day).unwrap();
            let part = if let Some(part) = args.next() { part.parse::<u8>()? }
                else { *select_key("part", puzzles)? };
            let puzzle = puzzles.get(&part).unwrap();
        
            let file = if let Some(file) = args.next() { file }
                else { default_input_file("./src/puzzles", year, day, part) };
            let input = read_to_string(file)?;
            let timer = Instant::now();
            let result = puzzle.solve(&input)?;
            let duration = timer.elapsed();
            println!("completed in {:.2?}    result: {}", duration, result);
        },
        Some("list") => {
            if let Some(year) = args.next() {
                let year = year.parse::<u16>()?;
                let puzzles = puzzles.get(&year).unwrap();
                if let Some(day) = args.next() {
                    if args.next().is_some() { Err("Too many arguments")? }
                    let day = day.parse::<u8>()?;
                    let puzzles = puzzles.get(&day).unwrap();
                    print_keys(puzzles);
                } else { print_keys(puzzles); }
            } else { print_keys(&puzzles); }
        }
        Some(_) | None => {
            println!("usage");
            println!("solve <year> <day> <part> [input] -- solve a specific puzzle");
            println!("list [year] [day] -- list all puzzles in category")
        }
    }
    
    Ok(())
}