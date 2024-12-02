use std::{collections::HashMap, io::BufRead};

use nom::{character, sequence, IResult};

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

#[allow(clippy::needless_pass_by_value)]
pub fn part_01(reader: Option<impl BufRead>) {
    let (mut left, mut right): (Vec<_>, Vec<_>) =
        stream_input(reader.expect("2024-01.txt should exist in the data directory")).unzip();

    left.sort_unstable();
    right.sort_unstable();

    let difference_sum = left
        .iter()
        .zip(right.iter())
        .map(|(&l, &r)| (r - l).abs())
        .sum::<i32>();

    println!("Sum of differences: {difference_sum}");
}

#[allow(clippy::needless_pass_by_value)]
pub fn part_02(reader: Option<impl BufRead>) {
    let mut left = Vec::new();
    let mut right = HashMap::<i32, i32>::new();

    for (l, r) in stream_input(reader.expect("2024-01.txt should exist in the data directory")) {
        left.push(l);

        *right.entry(r).or_default() += 1;
    }

    let similarity = left
        .iter()
        .map(|&v| v * right.get(&v).copied().unwrap_or(0))
        .sum::<i32>();

    println!("Similarity: {similarity}");
}
