use std::io::BufRead;

use nom::{branch, bytes, character, sequence, IResult};

/// Tries to exactly match an ASCII input of "mul(`i32`,`i32`)" and extract the two `i32` values.
/// Returns `(remaining_input, (i32, i32))` on success.
fn parse_mul(input: &[u8]) -> IResult<&[u8], (i32, i32)> {
    let (rest, _) = bytes::complete::tag("mul(")(input)?;
    let (rest, res) = sequence::separated_pair(
        character::complete::i32,
        character::complete::char(','),
        character::complete::i32,
    )(rest)?;
    let (rest, _) = character::complete::char(')')(rest)?;

    Ok((rest, res))
}

/// Attemps to find a valid "mul(`i32`, `i32`)" instruction in the `input`.
/// Returns `(remaining_input, (i32, i32))` on success.
fn extract_valid_mul(input: &[u8]) -> IResult<&[u8], (i32, i32)> {
    let mut rest = input;

    loop {
        let (rem, _) = bytes::complete::take_until("mul(")(rest)?;

        match parse_mul(rem) {
            Ok(r) => {
                return Ok(r);
            }
            Err(e) => {
                if rem.is_empty() {
                    return Err(e);
                }
            }
        }

        rest = &rem[4..];
    }
}

/// Iterator struct for producing the inner `(i32, i32)` from "mul(`i32`, `i32`)" instructions
/// found in the stream.
#[derive(Debug)]
struct BufMulOpIter<'reader, R> {
    buf: Vec<u8>,
    reader: &'reader mut R,
}

impl<R> Iterator for BufMulOpIter<'_, R>
where
    R: BufRead,
{
    type Item = std::io::Result<(i32, i32)>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.buf.clear();

            // All ops end with a ')'
            match self.reader.read_until(b')', &mut self.buf) {
                Ok(amt) => {
                    if amt == 0 {
                        return None;
                    }

                    // Finding a ')' was does not guarantee 'mul' is present
                    if let Ok((_, mul)) = extract_valid_mul(&self.buf) {
                        return Some(Ok(mul));
                    }
                }
                Err(e) => {
                    return Some(Err(e));
                }
            }
        }
    }
}

/// Helper trait for creating a `BufMulOpIter` directly from an `&mut reader`.
trait IterMulOps<'reader, R> {
    fn iter_mul_ops(&'reader mut self) -> BufMulOpIter<'reader, R>;
}

impl<'reader, R> IterMulOps<'reader, R> for R
where
    R: 'reader + BufRead,
{
    fn iter_mul_ops(&'reader mut self) -> BufMulOpIter<'reader, R> {
        BufMulOpIter {
            buf: Vec::with_capacity(1024),
            reader: self,
        }
    }
}

/// All operations/instructions supported by the parser.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Op {
    Do(),
    Dont(),
    Mul(i32, i32),
}

fn parse_do_op(input: &[u8]) -> IResult<&[u8], Op> {
    let (rest, _) = bytes::complete::tag("do()")(input)?;

    Ok((rest, Op::Do()))
}

fn parse_dont_op(input: &[u8]) -> IResult<&[u8], Op> {
    let (rest, _) = bytes::complete::tag("don't()")(input)?;

    Ok((rest, Op::Dont()))
}

fn parse_mul_op(input: &[u8]) -> IResult<&[u8], Op> {
    let (rest, (l, r)) = parse_mul(input)?;

    Ok((rest, Op::Mul(l, r)))
}

fn parse_op(input: &[u8]) -> IResult<&[u8], Op> {
    branch::alt((parse_do_op, parse_dont_op, parse_mul_op))(input)
}

/// Attempts to pull the next `Op` from the `input` stream.
fn extract_valid_op(input: &[u8]) -> IResult<&[u8], Op> {
    let mut rest = input;

    loop {
        let (rem, _) = bytes::complete::take_till(|b| b == b'd' || b == b'm')(rest)?;

        match parse_op(rem) {
            Ok(r) => {
                return Ok(r);
            }
            Err(e) => {
                if rem.is_empty() {
                    return Err(e);
                }
            }
        }

        rest = &rem[1..];
    }
}

/// Iterator struct for producing all supported `Op`s found in the stream.
#[derive(Debug)]
struct BufOpIter<'reader, R> {
    buf: Vec<u8>,
    reader: &'reader mut R,
}

impl<R> Iterator for BufOpIter<'_, R>
where
    R: BufRead,
{
    type Item = std::io::Result<Op>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.buf.clear();

            // All ops end with a ')'
            match self.reader.read_until(b')', &mut self.buf) {
                Ok(amt) => {
                    if amt == 0 {
                        return None;
                    }

                    // Finding a ')' was does not guarantee 'mul' is present
                    if let Ok((_, op)) = extract_valid_op(&self.buf) {
                        return Some(Ok(op));
                    }
                }
                Err(e) => {
                    return Some(Err(e));
                }
            }
        }
    }
}

/// Iterator struct for producing the inner `(i32, i32)` from "mul(`i32`, `i32`)" instructions
/// found in the stream that aren't filter by a previous "don't" instructions.
#[derive(Debug)]
struct FilteredMulOpIter<'reader, R> {
    keep: bool,
    inner: BufOpIter<'reader, R>,
}

impl<R> Iterator for FilteredMulOpIter<'_, R>
where
    R: BufRead,
{
    type Item = std::io::Result<(i32, i32)>;

    fn next(&mut self) -> Option<Self::Item> {
        for r in self.inner.by_ref() {
            match r {
                Ok(Op::Do()) => {
                    self.keep = true;
                }
                Ok(Op::Dont()) => {
                    self.keep = false;
                }
                Ok(Op::Mul(l, r)) => {
                    if self.keep {
                        return Some(Ok((l, r)));
                    }
                }
                Err(e) => {
                    return Some(Err(e));
                }
            }
        }

        None
    }
}

/// Helper trait for creating a `FilteredMulOpIter` directly from an `&mut reader`.
trait IterFilteredMulOps<'reader, R> {
    fn iter_filtered_mul_ops(&'reader mut self) -> FilteredMulOpIter<'reader, R>;
}

impl<'reader, R> IterFilteredMulOps<'reader, R> for R
where
    R: 'reader + BufRead,
{
    fn iter_filtered_mul_ops(&'reader mut self) -> FilteredMulOpIter<'reader, R> {
        FilteredMulOpIter {
            keep: true,
            inner: BufOpIter {
                buf: Vec::with_capacity(1024),
                reader: self,
            },
        }
    }
}

/// Part 1 approach:
/// 1. Use a custom `BufRead` consumer and `Nom` to extract all valid "mul(`i32`, `i32`)"
///    instructions as pairs of numbers
/// 2. Sum the multiples of the extracted numbers
#[allow(clippy::needless_pass_by_value)]
pub fn part_01(reader: Option<impl BufRead>) {
    let mut reader = reader.expect("2024-03.txt should exsist in the data directory");

    let mul_sum = reader
        .iter_mul_ops()
        .filter_map(Result::ok)
        .map(|(l, r)| l * r)
        .sum::<i32>();

    println!("Sum of multiplications: {mul_sum}");
}

/// Part 2 approach:
/// 1. Use a custom `BufRead` consumer and `Nom` to extract all unfilterd "mul(`i32`, `i32`)"
///    instructions by:
///     1. Grabbing chunks from `BufRead` by searching for ')'
///     2. Skipping irrelevant characters until a valid `Op` is found (if any)
///     3. Honor the "do"/"don't" operations while moving forward to filter out "mul" instructions
/// 2. Sum the multiples of the extracted numbers
#[allow(clippy::needless_pass_by_value)]
pub fn part_02(reader: Option<impl BufRead>) {
    let mut reader = reader.expect("2024-03.txt should exsist in the data directory");

    let mul_sum = reader
        .iter_filtered_mul_ops()
        .filter_map(Result::ok)
        .map(|(l, r)| l * r)
        .sum::<i32>();

    println!("Sum of multiplications: {mul_sum}");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn extract_muls() {
        let input = "xmul(2,4)%&mul[3,7]!@^do_not_mul(5,5)+mul(32,64]then(mul(11,8)mul(8,5))";
        let mut reader = std::io::BufReader::new(input.as_bytes());
        let muls = reader
            .iter_mul_ops()
            .filter_map(Result::ok)
            .collect::<Vec<_>>();

        assert_eq!(muls, vec![(2, 4), (5, 5), (11, 8), (8, 5)]);
    }

    #[test]
    fn filtered_muls() {
        let input = "xmul(2,4)&mul[3,7]!^don't()_mul(5,5)+mul(32,64](mul(11,8)undo()?mul(8,5))";
        let mut reader = std::io::BufReader::new(input.as_bytes());
        let muls = reader
            .iter_filtered_mul_ops()
            .filter_map(Result::ok)
            .collect::<Vec<_>>();

        assert_eq!(muls, vec![(2, 4), (8, 5)]);
    }
}
