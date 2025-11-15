use std::io::{self, BufRead};

use nom::{bytes, character, multi, sequence, IResult};

use rayon::prelude::*;

fn parse_input_line(input: &str) -> IResult<&str, (u64, Vec<u64>)> {
    sequence::separated_pair(
        character::complete::u64,
        bytes::complete::tag(": "),
        multi::separated_list1(character::complete::char(' '), character::complete::u64),
    )(input)
}

/// Allowed operations within the scope of the puzzle.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum Op {
    Add,
    Mul,
    Cat,
}

impl Op {
    /// Cycles through `[Op::Add, Op::Mul]`.
    /// Returns `(next_op, looped_to_add)`.
    /// Panics if `self == Op::Cat`
    fn next_add_mul(self) -> (Op, bool) {
        match self {
            Op::Add => (Op::Mul, false),
            Op::Mul => (Op::Add, true),
            Op::Cat => panic!("`Op::Cat` is not allowed as input for `Op::next_add_mul`"),
        }
    }

    /// Cycles through `[Op::Add, Op::Mul, Op::Cat]`.
    /// Returns `(next_op, looped_to_add)`.
    fn next_op(self) -> (Op, bool) {
        match self {
            Op::Add => (Op::Mul, false),
            Op::Mul => (Op::Cat, false),
            Op::Cat => (Op::Add, true),
        }
    }

    fn apply(self, lhs: u64, rhs: u64) -> u64 {
        match self {
            Op::Add => lhs.checked_add(rhs).unwrap(),
            Op::Mul => lhs * rhs,
            Op::Cat => {
                let mut val = rhs;
                let mut lhs = lhs;

                while val != 0 {
                    lhs *= 10;
                    val /= 10;
                }

                lhs + rhs
            }
        }
    }
}

/// Attempts to convert `ops` into its next add/mul permutation.
/// Returns `false` if the number of changes required is more than the length of the input
/// (overflow).
fn permute_add_mul(ops: &mut [Op]) -> bool {
    for o in ops {
        let (next, carry) = o.next_add_mul();

        *o = next;

        if !carry {
            return true;
        }
    }

    false
}

/// Attempts to convert `ops` into its next permutation.
/// Returns `false` if the number of changes required is more than the length of the input
/// (overflow).
fn permute_ops(ops: &mut [Op]) -> bool {
    for o in ops {
        let (next, carry) = o.next_op();

        *o = next;

        if !carry {
            return true;
        }
    }

    false
}

/// Determines if any permutation of operations (as driven by `permute`) can result in `expected`
/// when applied to `values`.
fn can_calibrate_to<F>(values: &[u64], expected: u64, permute: F) -> bool
where
    F: Fn(&mut [Op]) -> bool,
{
    if values.len() == 1 {
        return values[0] == expected;
    }

    let mut ops = vec![Op::Add; values.len() - 1];

    loop {
        let applied = values
            .iter()
            .skip(1)
            .zip(&ops)
            .fold(values[0], |acc, (&v, o)| o.apply(acc, v));

        if applied == expected {
            return true;
        }

        if !permute(&mut ops) {
            return false;
        }
    }
}

fn parse_input<R>(reader: R) -> Vec<(u64, Vec<u64>)>
where
    R: BufRead,
{
    reader
        .lines()
        .map_while(Result::ok)
        .filter_map(|l| parse_input_line(&l).map(|(_, v)| v).ok())
        .collect()
}

pub fn part_01(reader: io::Result<impl BufRead>) {
    let input = parse_input(reader.expect("2024-07.txt should be in the data folder"));

    let calibration_total = input
        .par_iter()
        .filter_map(|(e, v)| {
            if can_calibrate_to(v, *e, permute_add_mul) {
                Some(*e)
            } else {
                None
            }
        })
        .sum::<u64>();

    println!("Calibration total: {calibration_total}");
}

pub fn part_02(reader: io::Result<impl BufRead>) {
    let input = parse_input(reader.expect("2024-07.txt should be in the data folder"));

    let calibration_total = input
        .par_iter()
        .filter_map(|(e, v)| {
            if can_calibrate_to(v, *e, permute_ops) {
                Some(*e)
            } else {
                None
            }
        })
        .sum::<u64>();

    println!("Calibration total: {calibration_total}");
}

#[cfg(test)]
mod test {
    use super::*;

    fn parsed_test_input() -> Vec<(u64, Vec<u64>)> {
        let input = r"190: 10 19
3267: 81 40 27
83: 17 5
156: 15 6
7290: 6 8 6 15
161011: 16 10 13
192: 17 8 14
21037: 9 7 18 13
292: 11 6 16 20";

        input
            .lines()
            .filter_map(|l| parse_input_line(l).map(|(_, v)| v).ok())
            .collect()
    }

    #[test]
    fn ops_mul_add() {
        let results = parsed_test_input()
            .into_iter()
            .filter_map(|(e, v)| {
                if can_calibrate_to(&v, e, permute_add_mul) {
                    Some(e)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        assert_eq!(results, vec![190, 3267, 292]);
    }

    #[test]
    fn ops_mul_add_cat() {
        let results = parsed_test_input()
            .into_iter()
            .filter_map(|(e, v)| {
                if can_calibrate_to(&v, e, permute_ops) {
                    Some(e)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        assert_eq!(results, vec![190, 3267, 156, 7290, 192, 292]);
    }
}
