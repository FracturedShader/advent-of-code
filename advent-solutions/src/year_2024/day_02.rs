use std::io::BufRead;

use nom::{character, multi::separated_list1, IResult};

fn parse_line(input: &str) -> IResult<&str, Vec<i32>> {
    separated_list1(character::complete::space1, character::complete::i32)(input)
}

/// Does the sign of the delta match the overall gradient and have a magnitude in `1..=3`?
fn is_delta_safe(delta: i32, grad_signum: i32) -> bool {
    delta.signum() == grad_signum && (1..=3).contains(&delta.abs())
}

/// Determines if the differences between neighboring elements in `[first, ...rest]` are safe.
fn is_gradient_safe_split(first: i32, rest: &[i32]) -> bool {
    if rest.is_empty() {
        true
    } else {
        let delta = rest[0] - first;
        let grad_signum = delta.signum();

        if grad_signum == 0 {
            false
        } else {
            is_delta_safe(delta, grad_signum)
                && rest
                    .windows(2)
                    .all(|w| is_delta_safe(w[1] - w[0], grad_signum))
        }
    }
}

fn is_gradient_safe(values: &[i32]) -> bool {
    if values.len() < 2 {
        true
    } else {
        is_gradient_safe_split(values[0], &values[1..])
    }
}

/// Are the differences between neighboring elements safe, assuming we are allowed to skip one
/// element in `values[2..]`?
fn is_gradient_safe_skip_one_inner(values: &[i32]) -> bool {
    if values.len() < 2 {
        true
    } else {
        let delta = values[1] - values[0];
        let grad_signum = delta.signum();
        let delta_safe = is_delta_safe(delta, grad_signum);

        if values.len() < 3 {
            delta_safe
        } else if delta_safe {
            let mut prev = values[1];
            let mut can_skip = true;

            for v in &values[2..] {
                if !is_delta_safe(v - prev, grad_signum) {
                    if can_skip {
                        can_skip = false;
                        continue;
                    }

                    return false;
                }

                prev = *v;
            }

            true
        } else {
            false
        }
    }
}

/// Are the differences between neighboring elements safe, assuming we're allowed to remove any one
/// element we like?
fn is_gradient_partially_safe(values: &[i32]) -> bool {
    is_gradient_safe_skip_one_inner(values)
        || is_gradient_safe(&values[1..])
        || is_gradient_safe_split(values[0], &values[2..])
}

#[allow(clippy::needless_pass_by_value)]
pub fn part_01(reader: Option<impl BufRead>) {
    let num_safe = reader
        .expect("2024-02.txt should exsist in the data directory")
        .lines()
        .map_while(Result::ok)
        .map(|l| {
            i32::from(is_gradient_safe(
                &parse_line(&l)
                    .expect("lines should be space-separated numbers")
                    .1,
            ))
        })
        .sum::<i32>();

    println!("Safe reports: {num_safe}");
}

#[allow(clippy::needless_pass_by_value)]
pub fn part_02(reader: Option<impl BufRead>) {
    let num_safe = reader
        .expect("2024-02.txt should exsist in the data directory")
        .lines()
        .map_while(Result::ok)
        .map(|l| {
            i32::from(is_gradient_partially_safe(
                &parse_line(&l)
                    .expect("lines should be space-separated numbers")
                    .1,
            ))
        })
        .sum::<i32>();

    println!("Safe reports: {num_safe}");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn gradient_safety() {
        let input = r"7 6 4 2 1
1 2 7 8 9
9 7 6 2 1
1 3 2 4 5
8 6 4 4 1
1 3 6 7 9";

        let lines_safe: Vec<_> = input
            .lines()
            .map(|l| is_gradient_safe(&parse_line(l).unwrap().1))
            .collect();

        assert_eq!(lines_safe, vec![true, false, false, false, false, true]);
    }

    #[test]
    fn gradient_partial_safety() {
        let input = r"7 6 4 2 1
1 2 7 8 9
9 7 6 2 1
1 3 2 4 5
8 6 4 4 1
1 3 6 7 9";

        let lines_safe_partial: Vec<_> = input
            .lines()
            .map(|l| is_gradient_partially_safe(&parse_line(l).unwrap().1))
            .collect();

        assert_eq!(
            lines_safe_partial,
            vec![true, false, false, true, true, true]
        );
    }
}
