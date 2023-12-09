use std::{fmt, io::BufRead};

trait Score {
    fn score(&self) -> u32;
}

/// Result of a rock, paper, scissors competition
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum RoundOutcome {
    Lose,
    Draw,
    Win,
}

/// Error that can occur from trying to parse a `RoundOutcome` from a `u8` ASCII character
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct ParseRoundOutcomeByteError();

impl fmt::Display for ParseRoundOutcomeByteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "Provided ASCII char is not one of XYZ".fmt(f)
    }
}

/// `RoundOutcome` can be represented by the characters XYZ in the data
impl TryFrom<u8> for RoundOutcome {
    type Error = ParseRoundOutcomeByteError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            b'X' => Ok(RoundOutcome::Lose),
            b'Y' => Ok(RoundOutcome::Draw),
            b'Z' => Ok(RoundOutcome::Win),
            _ => Err(ParseRoundOutcomeByteError()),
        }
    }
}

impl Score for RoundOutcome {
    fn score(&self) -> u32 {
        match *self {
            RoundOutcome::Lose => 0,
            RoundOutcome::Draw => 3,
            RoundOutcome::Win => 6,
        }
    }
}

/// Possible hand shapes for rock, paper, scissors
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum HandShape {
    Rock,
    Paper,
    Scissors,
}

impl HandShape {
    /// Gets the `HandShape` that is needed to achieve the desired `RoundOutcome` against `other`
    fn for_outcome(outcome: RoundOutcome, other: Self) -> Self {
        match outcome {
            RoundOutcome::Lose => other.defeats(),
            RoundOutcome::Draw => other,
            RoundOutcome::Win => other.defeated_by(),
        }
    }

    /// Resolves a competition of `self` against another into a `RoundOutcome`
    fn compete(self, against: Self) -> RoundOutcome {
        match (self, against) {
            // Alternatively (a, b) if (b as u32 + 1) % 3 == a as u32
            (HandShape::Rock, HandShape::Scissors)
            | (HandShape::Paper, HandShape::Rock)
            | (HandShape::Scissors, HandShape::Paper) => RoundOutcome::Win,
            (a, b) if a == b => RoundOutcome::Draw,
            (_, _) => RoundOutcome::Lose,
        }
    }

    /// Gets the `HandShape` that can defeat `self`
    fn defeated_by(self) -> Self {
        match self {
            HandShape::Rock => HandShape::Paper,
            HandShape::Paper => HandShape::Scissors,
            HandShape::Scissors => HandShape::Rock,
        }
    }

    /// Gets the `HandShape` that `self` can defeat
    fn defeats(self) -> Self {
        match self {
            HandShape::Rock => HandShape::Scissors,
            HandShape::Paper => HandShape::Rock,
            HandShape::Scissors => HandShape::Paper,
        }
    }
}

impl Score for HandShape {
    fn score(&self) -> u32 {
        match *self {
            HandShape::Rock => 1,
            HandShape::Paper => 2,
            HandShape::Scissors => 3,
        }
    }
}

/// Error that can occur from trying to parse a `HandShape` from a `u8` ASCII character
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct ParseHandShapeByteError();

impl fmt::Display for ParseHandShapeByteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "Provided ASCII char is not one of ABCXYZ".fmt(f)
    }
}

/// `HandShape` can be represented by the characters ABCXYZ in the data
impl TryFrom<u8> for HandShape {
    type Error = ParseHandShapeByteError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            b'A' | b'X' => Ok(HandShape::Rock),
            b'B' | b'Y' => Ok(HandShape::Paper),
            b'C' | b'Z' => Ok(HandShape::Scissors),
            _ => Err(ParseHandShapeByteError()),
        }
    }
}

pub fn part_01(reader: Option<impl BufRead>) {
    let total_score = reader
        .expect("This problem requires data input")
        .lines()
        .flatten()
        .map(|l| {
            let mut hands = l.bytes().filter_map(|b| b.try_into().ok());

            let theirs: HandShape = hands.next().expect("Each round must have two hands");
            let ours = hands.next().expect("Each round must have two hands");

            ours.score() + ours.compete(theirs).score()
        })
        .sum::<u32>();

    println!("Strategy guide total score: {total_score}");
}

pub fn part_02(reader: Option<impl BufRead>) {
    let total_score = reader
        .expect("This problem requires data input")
        .lines()
        .flatten()
        .map(|l| {
            let mut parts = l.split(' ').filter_map(|c| c.as_bytes().first()).copied();

            let other = parts
                .next()
                .map(HandShape::try_from)
                .expect("Every line should consist of two characters separated by a single space")
                .expect("The first character should be one of ABC");

            let outcome = parts
                .next()
                .map(RoundOutcome::try_from)
                .expect("Every line should consist of two characters separated by a single space")
                .expect("The second character should be one of XYZ");

            HandShape::for_outcome(outcome, other).score() + outcome.score()
        })
        .sum::<u32>();

    println!("Strategy guide total score: {total_score}");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_hands() {
        let input = r"A Y
B X
C Z";

        let hands = input
            .bytes()
            .map(HandShape::try_from)
            .filter_map(Result::ok)
            .collect::<Vec<_>>();

        assert_eq!(
            vec![
                HandShape::Rock,
                HandShape::Paper,
                HandShape::Paper,
                HandShape::Rock,
                HandShape::Scissors,
                HandShape::Scissors
            ],
            hands
        );
    }

    #[test]
    fn compete_hands() {
        let hands = [HandShape::Rock,
            HandShape::Paper,
            HandShape::Paper,
            HandShape::Rock,
            HandShape::Scissors,
            HandShape::Scissors];

        let outcomes = hands
            .chunks_exact(2)
            .map(|round| round[1].compete(round[0]))
            .collect::<Vec<_>>();

        assert_eq!(
            vec![RoundOutcome::Win, RoundOutcome::Lose, RoundOutcome::Draw],
            outcomes
        );
    }

    #[test]
    fn score_hands() {
        let hands = [HandShape::Rock,
            HandShape::Paper,
            HandShape::Paper,
            HandShape::Rock,
            HandShape::Scissors,
            HandShape::Scissors];

        let outcomes = [RoundOutcome::Win, RoundOutcome::Lose, RoundOutcome::Draw];

        let scores = hands
            .iter()
            .skip(1)
            .step_by(2)
            .zip(outcomes.iter())
            .map(|(&hand, &outcome)| hand.score() + outcome.score())
            .collect::<Vec<_>>();

        assert_eq!(vec![8, 1, 6], scores);
    }

    #[test]
    fn parse_mixed() {
        let input = r"A Y
B X
C Z";

        let rounds = input
            .lines()
            .map(|l| {
                let mut iter = l.split(' ').map(|c| c.as_bytes()[0]);

                (
                    iter.next().map(HandShape::try_from).unwrap().unwrap(),
                    iter.next().map(RoundOutcome::try_from).unwrap().unwrap(),
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            vec![
                (HandShape::Rock, RoundOutcome::Draw),
                (HandShape::Paper, RoundOutcome::Lose),
                (HandShape::Scissors, RoundOutcome::Win),
            ],
            rounds
        );
    }

    #[test]
    fn score_mixed() {
        let rounds = [(HandShape::Rock, RoundOutcome::Draw),
            (HandShape::Paper, RoundOutcome::Lose),
            (HandShape::Scissors, RoundOutcome::Win)];

        let scores = rounds
            .iter()
            .map(|&(other, outcome)| {
                HandShape::for_outcome(outcome, other).score() + outcome.score()
            })
            .collect::<Vec<_>>();

        assert_eq!(vec![4, 1, 7], scores);
    }
}
