use std::{collections::HashMap, io::BufRead};

fn stream_input(reader: impl BufRead) -> impl Iterator<Item = (i32, i32)> {
    reader.lines().filter_map(|r| {
        r.ok().map(|l| {
            let mut pieces = l.split(' ').filter_map(|p| p.parse::<i32>().ok());

            (
                pieces
                    .next()
                    .expect("lines should have two numbers separated by spaces"),
                pieces
                    .next()
                    .expect("lines should have two numbers separated by spaces"),
            )
        })
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
