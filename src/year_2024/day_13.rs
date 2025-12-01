use std::io::{self, BufRead};

use glam::I64Vec2;
use nom::IResult;

use crate::common::{self, a_star_unordered};

/// Encapsulates the details for a button, including the `cost` for pressing it and the `offset`
/// that gets applied on a press.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct Button {
    cost: i64,
    offset: I64Vec2,
}

/// Encapsulates the puzzle's definition of a machine with both `buttons` and the location of the
/// `prize` to be reached.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Machine {
    buttons: [Button; 2],
    prize: I64Vec2,
}

impl Machine {
    /// Works towards the `prize` using A*. Only suitable for reasonably near locations.
    ///
    /// Returns the minimum number of tokens required to reach the `prize` using the `buttons`
    fn tokens_to_prize(&self) -> Option<i64> {
        a_star_unordered(
            I64Vec2::ZERO,
            |n| {
                if n.x > self.prize.x || n.y > self.prize.y {
                    None
                } else {
                    Some(self.buttons.map(|b| (b.cost, n + b.offset)))
                }
            },
            |n| {
                if n.x > self.prize.x || n.y > self.prize.y {
                    self.prize.x + self.prize.y + 1
                } else {
                    let d = self.prize - n;

                    d.x + d.y
                }
            },
            |n| *n == self.prize,
        )
        .map(|mut b| b.next().unwrap().cost)
    }

    /// Analytically solves for the linear combination of the two `button`s required to reach the
    /// `prize`.
    ///
    /// Returns the minimum number of tokens required to reach the `prize` using the `buttons`
    fn tokens_to_far_prize(&self) -> Option<i64> {
        let a_denom = self.buttons[1].offset.perp_dot(self.buttons[0].offset);
        let b_denom = self.buttons[1].offset.x;

        if a_denom == 0 || b_denom == 0 {
            return None;
        }

        let a_num = self.buttons[1].offset.perp_dot(self.prize);

        if a_num % a_denom != 0 {
            return None;
        }

        let a_val = a_num / a_denom;

        let b_num = self.prize.x - a_val * self.buttons[0].offset.x;

        if b_num % b_denom != 0 {
            return None;
        }

        let b_val = b_num / b_denom;

        if a_val < 0 || b_val < 0 {
            None
        } else {
            Some(3 * a_val + b_val)
        }
    }
}

/// Parses a line matching the genreal form of "Button `button`: X+###, Y+###" into a [`Button`].
fn parse_button(input: &str, button: char, cost: i64) -> IResult<&str, Button> {
    let (rest, _) = nom::sequence::tuple((
        nom::bytes::complete::tag("Button "),
        nom::character::complete::char(button),
        nom::bytes::complete::tag(": X+"),
    ))(input)?;

    let (rest, x) = nom::character::complete::i64(rest)?;
    let (rest, _) = nom::bytes::complete::tag(", Y+")(rest)?;
    let (rest, y) = nom::character::complete::i64(rest)?;

    let offset = I64Vec2::new(x, y);

    Ok((rest, Button { cost, offset }))
}

/// Parses a line matching the genreal form of "Prize: X=8031, Y=11372" into a prize location.
fn parse_prize(input: &str, is_offset: bool) -> IResult<&str, I64Vec2> {
    let (rest, _) = nom::bytes::complete::tag("Prize: X=")(input)?;
    let (rest, (x, y)) = nom::sequence::separated_pair(
        nom::character::complete::i64,
        nom::bytes::complete::tag(", Y="),
        nom::character::complete::i64,
    )(rest)?;

    if is_offset {
        Ok((
            rest,
            I64Vec2::new(x + 10_000_000_000_000i64, y + 10_000_000_000_000i64),
        ))
    } else {
        Ok((rest, I64Vec2::new(x, y)))
    }
}

/// Parses input into a series of [`Machine`]s, so long as it matches the expected puzzle format:
/// <pre>
/// Button A: X+55, Y+31
/// Button B: X+26, Y+45
/// Prize: X=8031, Y=11372
/// </pre>
struct MachineProducer<T>(T, String, bool);

impl<T: BufRead> Iterator for MachineProducer<T> {
    type Item = std::io::Result<Machine>;

    fn next(&mut self) -> Option<Self::Item> {
        // Skip all blank lines until Button A definition is found
        let first_button = loop {
            self.1.clear();

            match self.0.read_line(&mut self.1) {
                Ok(0) => return None,
                Ok(_) => {
                    if !self.1.is_empty() && self.1 != "\n" && self.1 != "\r\n" {
                        break parse_button(&self.1, 'A', 3)
                            .expect("First machine line should be for Button A")
                            .1;
                    }
                }
                Err(e) => return Some(Err(e)),
            }
        };

        self.1.clear();

        // After "Button A" is found, there should be a "Button B"
        let second_button = match self.0.read_line(&mut self.1) {
            Ok(0) => {
                return Some(Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "Machine definitions are exactly 3 lines long",
                )))
            }
            Ok(_) => {
                parse_button(&self.1, 'B', 1)
                    .expect("Second machine line should be for Button B")
                    .1
            }
            Err(e) => return Some(Err(e)),
        };

        self.1.clear();

        // Following both buttons there should be a prize
        let prize = match self.0.read_line(&mut self.1) {
            Ok(0) => {
                return Some(Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "Machine definitions are exactly 3 lines long",
                )))
            }
            Ok(_) => {
                parse_prize(&self.1, self.2)
                    .expect("Third machine line should be for the prize")
                    .1
            }
            Err(e) => return Some(Err(e)),
        };

        Some(Ok(Machine {
            buttons: [first_button, second_button],
            prize,
        }))
    }
}

trait IntoMachines<T> {
    /// Parses formatted input into a series of [`Machine`]s (optionally separated by blank lines).
    fn into_machines(self, is_offset: bool) -> MachineProducer<T>;
}

impl<T: BufRead> IntoMachines<T> for T {
    fn into_machines(self, is_offset: bool) -> MachineProducer<T> {
        MachineProducer(self, String::new(), is_offset)
    }
}

pub fn part_01() {
    let total_tokens = common::puzzle_input("2024-13")
        .unwrap()
        .into_machines(false)
        .filter_map(|m| m.unwrap().tokens_to_prize())
        .sum::<i64>();

    println!("Fewest tokens to win all prizes: {total_tokens}");
}

pub fn part_02() {
    let total_tokens = common::puzzle_input("2024-13")
        .unwrap()
        .into_machines(true)
        .filter_map(|m| m.unwrap().tokens_to_far_prize())
        .sum::<i64>();

    println!("Fewest tokens to win all prizes: {total_tokens}");
}

#[cfg(test)]
mod test {
    use io::BufReader;

    use super::*;

    fn parsed_input(is_offset: bool) -> Vec<Machine> {
        let input = r"Button A: X+94, Y+34
Button B: X+22, Y+67
Prize: X=8400, Y=5400

Button A: X+26, Y+66
Button B: X+67, Y+21
Prize: X=12748, Y=12176

Button A: X+17, Y+86
Button B: X+84, Y+37
Prize: X=7870, Y=6450

Button A: X+69, Y+23
Button B: X+27, Y+71
Prize: X=18641, Y=10279";

        BufReader::new(input.as_bytes())
            .into_machines(is_offset)
            .filter_map(Result::ok)
            .collect()
    }

    #[test]
    fn machine_parsing() {
        let machines = parsed_input(false);

        assert_eq!(
            machines,
            vec![
                Machine {
                    buttons: [
                        Button {
                            cost: 3,
                            offset: I64Vec2::new(94, 34),
                        },
                        Button {
                            cost: 1,
                            offset: I64Vec2::new(22, 67),
                        }
                    ],
                    prize: I64Vec2::new(8400, 5400)
                },
                Machine {
                    buttons: [
                        Button {
                            cost: 3,
                            offset: I64Vec2::new(26, 66),
                        },
                        Button {
                            cost: 1,
                            offset: I64Vec2::new(67, 21),
                        }
                    ],
                    prize: I64Vec2::new(12748, 12176)
                },
                Machine {
                    buttons: [
                        Button {
                            cost: 3,
                            offset: I64Vec2::new(17, 86),
                        },
                        Button {
                            cost: 1,
                            offset: I64Vec2::new(84, 37),
                        }
                    ],
                    prize: I64Vec2::new(7870, 6450)
                },
                Machine {
                    buttons: [
                        Button {
                            cost: 3,
                            offset: I64Vec2::new(69, 23),
                        },
                        Button {
                            cost: 1,
                            offset: I64Vec2::new(27, 71),
                        }
                    ],
                    prize: I64Vec2::new(18641, 10279)
                },
            ]
        );
    }

    #[test]
    fn prize_tokens() {
        let tokens = parsed_input(false)
            .iter()
            .map(Machine::tokens_to_prize)
            .collect::<Vec<_>>();

        assert_eq!(tokens, vec![Some(280), None, Some(200), None]);
    }

    #[test]
    fn offset_prize_tokens() {
        let can_win = parsed_input(true)
            .iter()
            .map(Machine::tokens_to_far_prize)
            .map(|o| o.is_some())
            .collect::<Vec<_>>();

        assert_eq!(can_win, vec![false, true, false, true]);
    }
}
