use std::{collections::HashMap, io::BufRead};

use nom::{character, sequence, IResult};

use crate::common;

fn parse_line(input: &str) -> IResult<&str, (i32, i32)> {
    sequence::separated_pair(
        character::complete::i32,
        character::complete::space1,
        character::complete::i32,
    )(input)
}

fn stream_input(reader: impl BufRead) -> impl Iterator<Item = (i32, i32)> {
    reader.lines().map_while(Result::ok).filter_map(|l| {
        if l.is_empty() {
            None
        } else {
            Some(
                parse_line(&l)
                    .expect("lines should be pairs of numbers separated by whitespace")
                    .1,
            )
        }
    })
}

/// Process for Part 1 is:
/// 1. Split the incoming stream into two `Vec`s of numbers
/// 2. Sort each `Vec` independently
/// 3. Walk both `Vec`s and sum their absolute differences
pub fn part_01() {
    let (mut left, mut right): (Vec<_>, Vec<_>) =
        stream_input(common::puzzle_input("2024-01").unwrap()).unzip();

    left.sort_unstable();
    right.sort_unstable();

    let difference_sum = left
        .iter()
        .zip(right.iter())
        .map(|(&l, &r)| (r - l).abs())
        .sum::<i32>();

    println!("Sum of differences: {difference_sum}");
}

/// Process for Part 2 is:
/// 1. Split the incoming stream into pairs of numbers, putting the left in a `Vec` and the right in
///    a `HashMap` to track how many times a number is seen
/// 2. Walk the `Vec` and multiply the value by what's in the `HashMap` for it (or 0)
pub fn part_02() {
    let mut left = Vec::new();
    let mut right = HashMap::<i32, i32>::new();

    for (l, r) in stream_input(common::puzzle_input("2024-01").unwrap()) {
        left.push(l);

        *right.entry(r).or_default() += 1;
    }

    let similarity = left
        .iter()
        .map(|&v| v * right.get(&v).copied().unwrap_or(0))
        .sum::<i32>();

    println!("Similarity: {similarity}");
}
