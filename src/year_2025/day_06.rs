use std::{
    io::{BufRead, Read},
    str::FromStr,
};

use nalgebra::Vector2;

use crate::common;

/// The problem only needs support for addition and multiplication, so capture those options.
#[derive(Debug, Clone, Copy)]
enum MathOp {
    Add,
    Mul,
}

impl TryFrom<u8> for MathOp {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            b'+' => Ok(MathOp::Add),
            b'*' => Ok(MathOp::Mul),
            _ => Err("Not a math operation"),
        }
    }
}

/// All space-separated entries in the problem are either a number, or a mathematical operation.
#[derive(Debug, Clone, Copy)]
enum ProblemEntry {
    Number(u64),
    Op(MathOp),
}

impl ProblemEntry {
    pub fn is_number(self) -> bool {
        matches!(self, ProblemEntry::Number(_))
    }

    pub fn is_op(self) -> bool {
        matches!(self, ProblemEntry::Op(_))
    }
}

impl FromStr for ProblemEntry {
    type Err = <u64 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "+" => Ok(ProblemEntry::Op(MathOp::Add)),
            "*" => Ok(ProblemEntry::Op(MathOp::Mul)),
            _ => Ok(ProblemEntry::Number(s.parse()?)),
        }
    }
}

/// For part 1, the data is interpreted as columns of at least two numbers followed by a
/// mathematical operation.
struct ProblemGrid(common::Grid2<ProblemEntry>);

impl ProblemGrid {
    /// Get the results of each column of numbers according to the operation defined at the bottom
    /// of the column.
    pub fn problem_results(&self) -> Vec<u64> {
        if self.0.height() < 3 {
            Vec::default()
        } else {
            let last_row = self.0.height() - 1;

            (0..self.0.width())
                .map(|x| {
                    // Safety: construction ensures that top n-1 rows are only numbers
                    let nums = (0..last_row).map(|y| match self.0.get(Vector2::new(x, y)) {
                        ProblemEntry::Number(n) => n,
                        ProblemEntry::Op(_) => unreachable!(),
                    });

                    // Safety: construction ensures that last row is only operations
                    match self.0.get(Vector2::new(x, last_row)) {
                        ProblemEntry::Op(MathOp::Add) => nums.sum(),
                        ProblemEntry::Op(MathOp::Mul) => nums.product(),
                        ProblemEntry::Number(_) => unreachable!(),
                    }
                })
                .collect()
        }
    }

    /// Try to take the lines of input and turn them into a [[`ProblemGrid`]]. Lines split by
    /// whitespace should:
    /// - All have the same number of entries
    /// - Have only number entries followed by exactly one row of only operator entries
    pub fn try_from_lines<I, S>(lines: I) -> Result<Self, &'static str>
    where
        I: Iterator<Item = S>,
        S: AsRef<str>,
    {
        let mut entries: Vec<_> = Vec::default();
        let mut width = None;

        // First, split the lines by whitepsace and ensure they all have the same number of entries
        for l in lines {
            let l = l.as_ref();

            entries.extend(l.split_whitespace().map(ProblemEntry::from_str));

            if let Some(w) = width {
                if !entries.len().is_multiple_of(w) {
                    return Err("Lines don't all have the same number of entries");
                }
            } else {
                width = Some(entries.len());
            }
        }

        if let Some(width) = width {
            // Operations need at least two numbers to work on
            if entries.len() < width * 3 {
                return Err("Puzzle input does not have enough rows (at least 3)");
            }

            let width32 =
                u32::try_from(width).map_err(|_| "Puzzle width should fit within `u32`")?;

            // Unwrap the parsing results to ensure all pieces are valid
            let entries = entries
                .into_iter()
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| "Input should consist of only '+', '*', and positive numbers")?;

            // Verify that the top n-1 rows are only numbers
            if entries[..(entries.len() - width)].iter().any(|v| v.is_op()) {
                return Err("All rows except last row should only be numbers");
            }

            // Verify that the bottom row is only operations
            if entries[(entries.len() - width)..]
                .iter()
                .any(|v| v.is_number())
            {
                return Err("Last row must be only operations");
            }

            // Turn it into a grid!
            Ok(Self(common::Grid2::from_vec(entries, width32).unwrap()))
        } else {
            // Empty input makes for an empty result
            Ok(Self(common::Grid2::default()))
        }
    }
}

/// Grabs all but the last row of a column (for puzzle reasons), effectively transposes it and
/// parses it as an ASCII number.
fn number_from_ascii_column(
    grid: &common::Grid2<u8>,
    column: i32,
) -> Result<Option<u64>, &'static str> {
    (0..(grid.height() - 1))
        .filter_map(|y| match grid.get(Vector2::new(column, y)) {
            n @ b'0'..=b'9' => Some(Ok(n - b'0')),
            b' ' => None,
            _ => Some(Err("Column has non-numeric non-whitespace entry")),
        })
        .try_fold(None, |acc, r| match r {
            Ok(v) => Ok(Some(acc.unwrap_or(0) * 10 + u64::from(v))),
            Err(e) => Err(e),
        })
}

/// Applies `op` to the sequence of numbers produced by `nums`.
fn splat_op<I>(op: MathOp, nums: I) -> u64
where
    I: Iterator<Item = u64>,
{
    match op {
        MathOp::Add => nums.sum(),
        MathOp::Mul => nums.product(),
    }
}

/// Treats the supplied `grid` as columns of numbers with operators in the last row. Sub-problems
/// are expected to be separated by whitespace-only columns.
fn columnar_results(grid: &common::Grid2<u8>) -> Result<Vec<u64>, &'static str> {
    static NO_OP_ERR: &str = "Found a group of numbers without an operation";

    let last_row = grid.height() - 1;
    let mut results = Vec::with_capacity((grid.width() / 2).try_into().unwrap());
    let mut nums = Vec::with_capacity(8);
    let mut op: Option<MathOp> = None;

    for column in (0..grid.width()).rev() {
        // Find the operation in the bottom row, but should only find one
        op = match (
            op,
            (*grid.get(Vector2::new(column, last_row))).try_into().ok(),
        ) {
            (None, p) => Ok(p),
            (Some(_), Some(_)) => Err("Found a group of numbers with more than one operation"),
            _ => Ok(op),
        }?;

        // Pull the number from the column and hold onto it. If there isn't one (empty column),
        // apply the found operation to all numbers found.
        if let Some(n) = number_from_ascii_column(grid, column)? {
            nums.push(n);
        } else {
            let found_op = op.take().ok_or(NO_OP_ERR)?;

            results.push(splat_op(found_op, nums.drain(..)));
        }
    }

    if !nums.is_empty() {
        let found_op = op.take().ok_or(NO_OP_ERR)?;

        results.push(splat_op(found_op, nums.drain(..)));
    }

    Ok(results)
}

pub fn part_01() {
    let results_total: u64 = ProblemGrid::try_from_lines(
        common::puzzle_input("2025-06")
            .unwrap()
            .lines()
            .map_while(Result::ok),
    )
    .unwrap()
    .problem_results()
    .iter()
    .sum();

    println!("Sum of row-wise results: {results_total}");
}

pub fn part_02() {
    let grid = common::Grid2::try_from_bytes(
        common::puzzle_input("2025-06")
            .unwrap()
            .bytes()
            .map_while(Result::ok),
    )
    .unwrap();

    let results_total: u64 = columnar_results(&grid).unwrap().into_iter().sum();

    println!("Sum of column-wise results: {results_total}");
}

#[cfg(test)]
mod test {
    use super::*;

    fn data() -> &'static str {
        "123 328  51 64 \n 45 64  387 23 \n  6 98  215 314\n*   +   *   +  "
    }

    #[test]
    fn horizonal_numbers() {
        let op_results = ProblemGrid::try_from_lines(data().lines())
            .unwrap()
            .problem_results();

        assert_eq!(op_results, vec![33210, 490, 4243455, 401]);
    }

    #[test]
    fn vertical_numbers() {
        let grid = common::Grid2::try_from_bytes(data().bytes()).unwrap();
        let op_results = columnar_results(&grid).unwrap();

        assert_eq!(op_results, vec![1058, 3253600, 625, 8544]);
    }
}
