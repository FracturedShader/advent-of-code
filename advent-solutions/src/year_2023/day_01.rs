use std::io::BufRead;

/// Handles traversing a sequence of bytes to find a match
struct ByteSequence<const N: usize> {
    bytes: [u8; N],
    idx: usize,
}

impl<const N: usize> ByteSequence<N> {
    pub fn new(bytes: &[u8; N]) -> Self {
        Self {
            bytes: *bytes,
            idx: 0,
        }
    }

    /// Progresses along the internal byte sequence by one more byte
    /// and returns `true` if the sequence has completed.
    pub fn completes_sequence(&mut self, byte: u8) -> bool {
        if byte == self.bytes[self.idx] {
            self.idx += 1;

            if self.idx == N {
                self.idx = 0;

                true
            } else {
                false
            }
        } else {
            // Handle the case where the failing byte matches the first
            if byte == self.bytes[0] {
                self.idx = 1;
            } else {
                self.idx = 0;
            }

            false
        }
    }
}

/// Helper for checking if any spelled-out numbers in [1, 9] are contained in
/// a byte sequence in a potentially overlapping way.
struct NumberWords {
    one: ByteSequence<3>,
    two: ByteSequence<3>,
    three: ByteSequence<5>,
    four: ByteSequence<4>,
    five: ByteSequence<4>,
    six: ByteSequence<3>,
    seven: ByteSequence<5>,
    eight: ByteSequence<5>,
    nine: ByteSequence<4>,
}

impl NumberWords {
    pub fn new() -> Self {
        Self {
            one: ByteSequence::new(b"one"),
            two: ByteSequence::new(b"two"),
            three: ByteSequence::new(b"three"),
            four: ByteSequence::new(b"four"),
            five: ByteSequence::new(b"five"),
            six: ByteSequence::new(b"six"),
            seven: ByteSequence::new(b"seven"),
            eight: ByteSequence::new(b"eight"),
            nine: ByteSequence::new(b"nine"),
        }
    }

    /// Checks if `byte` completes any spelled-out numbers in [1, 9] and
    /// advances tracking for all potential numbers regardless.
    pub fn completes_any(&mut self, byte: u8) -> Option<i32> {
        let mut ret = None;

        if self.one.completes_sequence(byte) {
            ret = Some(1);
        }

        if self.two.completes_sequence(byte) {
            ret = Some(2);
        }

        if self.three.completes_sequence(byte) {
            ret = Some(3);
        }

        if self.four.completes_sequence(byte) {
            ret = Some(4);
        }

        if self.five.completes_sequence(byte) {
            ret = Some(5);
        }

        if self.six.completes_sequence(byte) {
            ret = Some(6);
        }

        if self.seven.completes_sequence(byte) {
            ret = Some(7);
        }

        if self.eight.completes_sequence(byte) {
            ret = Some(8);
        }

        if self.nine.completes_sequence(byte) {
            ret = Some(9);
        }

        ret
    }
}

fn calibration_numerals(line: &str) -> i32 {
    let mut nums = line.bytes().filter_map(|b| match b {
        b'0'..=b'9' => Some(i32::from(b - b'0')),
        _ => None,
    });

    let first_num = nums.next().unwrap_or(0);
    let second_num = nums.next_back().unwrap_or(first_num);

    first_num * 10 + second_num
}

fn calibration_numbers(line: &str) -> i32 {
    let mut first_num = None::<i32>;
    let mut second_num = None::<i32>;

    let mut word_matcher = NumberWords::new();

    for b in line.bytes() {
        let num = match b {
            b'0'..=b'9' => Some(i32::from(b - b'0')),
            _ => word_matcher.completes_any(b),
        };

        if let Some(num) = num {
            if first_num.is_none() {
                first_num = Some(num);
            }

            second_num = Some(num);
        }
    }

    first_num.unwrap_or(0) * 10 + second_num.unwrap_or(0)
}

pub fn part_01(reader: Option<impl BufRead>) {
    let total = reader
        .expect("data should be available for this problem")
        .lines()
        .filter_map(|l| l.ok().as_deref().map(calibration_numerals))
        .sum::<i32>();

    println!("Calibration total: {total}");
}

pub fn part_02(reader: Option<impl BufRead>) {
    let total = reader
        .expect("data should be available for this problem")
        .lines()
        .filter_map(|l| l.ok().as_deref().map(calibration_numbers))
        .sum::<i32>();

    println!("Calibration total: {total}");
}

#[cfg(test)]
mod test {
    use std::io::BufReader;

    use super::*;

    #[test]
    fn extract_numerals() {
        let input = r#"1abc2
pqr3stu8vwx
a1b2c3d4e5f
treb7uchet"#;

        let vals = BufReader::new(input.as_bytes())
            .lines()
            .filter_map(|l| l.ok().as_deref().map(calibration_numerals))
            .collect::<Vec<_>>();

        assert_eq!(vals, vec![12, 38, 15, 77]);
    }

    #[test]
    fn extract_numbers() {
        let input = r#"two1nine
eightwothree
abcone2threexyz
xtwone3four
4nineeightseven2
zoneight234
7pqrstsixteen"#;

        let vals = BufReader::new(input.as_bytes())
            .lines()
            .filter_map(|l| l.ok().as_deref().map(calibration_numbers))
            .collect::<Vec<_>>();

        assert_eq!(vals, vec![29, 83, 13, 24, 42, 14, 76]);
    }
}
