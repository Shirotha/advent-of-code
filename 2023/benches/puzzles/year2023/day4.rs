use std::fs::read_to_string;
use divan::{black_box, Bencher};
use advent_of_code::{
    default_input_file,
    puzzles::year2023::day4::{part1, part2}
};

#[divan::bench]
fn bench1(bencher: Bencher) {
    let file = default_input_file("./src/puzzles", 2023, 4, 1);
    let input = read_to_string(file).unwrap();
    bencher.bench_local( move || { let _ = black_box(part1(&input)); } );
}

#[divan::bench]
fn bench2(bencher: Bencher) {
    let file = default_input_file("./src/puzzles", 2023, 4, 2);
    let input = read_to_string(file).unwrap();
    bencher.bench_local( move || { let _ = black_box(part2(&input)); } );
}