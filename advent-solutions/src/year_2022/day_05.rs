use std::{io::BufRead, str::FromStr};

use anyhow::Context;

/// Representation for moving at least one crate between two separate stacks
#[derive(Clone, Debug, PartialEq, Eq)]
struct StackMove {
    count: usize,
    from_stack: usize,
    to_stack: usize,
}

impl FromStr for StackMove {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(' ').skip(1).step_by(2);

        let count = parts
            .next()
            .context("input should be \"move [count] from [stack] to [stack]\"")?
            .parse()?;

        let from_stack = parts
            .next()
            .context("input should be \"move [count] from [stack] to [stack]\"")?
            .parse::<usize>()?
            - 1;

        let to_stack = parts
            .next()
            .context("input should be \"move [count] from [stack] to [stack]\"")?
            .parse::<usize>()?
            - 1;

        if count == 0 {
            Err(anyhow::Error::msg(
                "A move operation should move at least one crate",
            ))
        } else if from_stack == to_stack {
            Err(anyhow::Error::msg(
                "A move operation should move between different stacks",
            ))
        } else {
            Ok(StackMove {
                count,
                from_stack,
                to_stack,
            })
        }
    }
}

/// Represents stacks of crates that can be manipulated via `StackMove`s. The moves can be
/// performed one at a time by calling `apply_move_9000`, or a whole move at a time by calling
/// `apply_move_9001`.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Stacks(Vec<Vec<char>>);

impl Stacks {
    /// Converts the first part of a line-based input into stacks by rotating from a visually
    /// intuitive vertical form into a memory friendly orientation.
    fn from_lines<S, L>(lines: &mut L) -> Self
    where
        S: AsRef<str>,
        L: Iterator<Item = S>,
    {
        let stacks_transposed: Vec<Vec<_>> = lines
            .take_while(|l| !l.as_ref().is_empty())
            .map(|l| l.as_ref().chars().skip(1).step_by(4).collect())
            .collect();

        // All the information is there, but transposed
        let n_rows = stacks_transposed.len();
        let n_cols = stacks_transposed[0].len();

        let mut stacks = vec![vec![' '; n_rows]; n_cols];

        for r in 0..n_rows {
            for (c, item) in stacks_transposed[n_rows - r - 1].iter().enumerate() {
                stacks[c][r] = *item;
            }
        }

        for s in &mut stacks {
            let first_blank = s
                .iter()
                .enumerate()
                .find(|&(_, &c)| c.is_ascii_whitespace())
                .map(|(idx, _)| idx);

            if let Some(idx) = first_blank {
                s.truncate(idx);
            }
        }

        Self(stacks)
    }

    /// Applies the movements in a 9000 series compliant manner by moving one crate at a time.
    fn apply_move_9000(&mut self, m: &StackMove) {
        for _ in 0..m.count {
            let v = self.0[m.from_stack]
                .pop()
                .expect("moves should not occur on exhausted stacks");

            self.0[m.to_stack].push(v);
        }
    }

    /// Applies the movements in a 9001 series compliant manner by moving whole groups at a time.
    fn apply_move_9001(&mut self, m: &StackMove) {
        let from_len = self.0[m.from_stack].len();

        assert!(m.count <= from_len, "Connot move more items than the stack contains");

        let split_point = from_len - m.count;
        let old_len = self.0[m.to_stack].len();

        self.0[m.to_stack].resize(old_len + m.count, ' ');

        // Safety: `StackMove` guarantees source and destination are different, source is checked
        // to conatin the requested count, and destination has required space set aside
        unsafe {
            std::ptr::copy_nonoverlapping(
                &self.0[m.from_stack][split_point] as _,
                &mut self.0[m.to_stack][old_len] as _,
                m.count,
            );

            self.0[m.from_stack].set_len(split_point);
        }
    }

    /// Iterates the crates residing at the top of all stacks in the same order as the stacks.
    fn top_crates(&self) -> impl Iterator<Item = char> + '_ {
        self.0.iter().filter_map(|s| s.last().copied())
    }
}

pub fn part_01(reader: Option<impl BufRead>) {
    let mut lines = reader
        .expect("data should be available for this problem")
        .lines()
        .flatten();

    let mut stacks = Stacks::from_lines(&mut lines);

    for m in lines.map(|s| {
        s.parse::<StackMove>()
            .expect("remaining lines should all be moves")
    }) {
        stacks.apply_move_9000(&m);
    }

    println!(
        "Crates on the tops of the stacks: {}",
        stacks.top_crates().collect::<String>()
    );
}

pub fn part_02(reader: Option<impl BufRead>) {
    let mut lines = reader
        .expect("data should be available for this problem")
        .lines()
        .flatten();

    let mut stacks = Stacks::from_lines(&mut lines);

    for m in lines.map(|s| {
        s.parse::<StackMove>()
            .expect("remaining lines should all be moves")
    }) {
        stacks.apply_move_9001(&m);
    }

    println!(
        "Crates on the tops of the stacks: {}",
        stacks.top_crates().collect::<String>()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_input() {
        let input = r"    [D]    
[N] [C]    
[Z] [M] [P]
 1   2   3 

move 1 from 2 to 1
move 3 from 1 to 3
move 2 from 2 to 1
move 1 from 1 to 2
";

        let mut lines = input.lines();

        let stacks = Stacks::from_lines(&mut lines);

        assert_eq!(
            vec![
                vec!['1', 'Z', 'N'],
                vec!['2', 'M', 'C', 'D'],
                vec!['3', 'P']
            ],
            stacks.0
        );

        let moves = lines.flat_map(StackMove::from_str).collect::<Vec<_>>();

        assert_eq!(
            vec![
                StackMove {
                    count: 1,
                    from_stack: 1,
                    to_stack: 0
                },
                StackMove {
                    count: 3,
                    from_stack: 0,
                    to_stack: 2
                },
                StackMove {
                    count: 2,
                    from_stack: 1,
                    to_stack: 0
                },
                StackMove {
                    count: 1,
                    from_stack: 0,
                    to_stack: 1
                },
            ],
            moves
        );
    }

    #[test]
    fn apply_moves_9000() {
        let mut stacks = Stacks(vec![
            vec!['1', 'Z', 'N'],
            vec!['2', 'M', 'C', 'D'],
            vec!['3', 'P'],
        ]);

        let moves = vec![
            StackMove {
                count: 1,
                from_stack: 1,
                to_stack: 0,
            },
            StackMove {
                count: 3,
                from_stack: 0,
                to_stack: 2,
            },
            StackMove {
                count: 2,
                from_stack: 1,
                to_stack: 0,
            },
            StackMove {
                count: 1,
                from_stack: 0,
                to_stack: 1,
            },
        ];

        let outcomes = vec![
            vec![
                vec!['1', 'Z', 'N', 'D'],
                vec!['2', 'M', 'C'],
                vec!['3', 'P'],
            ],
            vec![
                vec!['1'],
                vec!['2', 'M', 'C'],
                vec!['3', 'P', 'D', 'N', 'Z'],
            ],
            vec![
                vec!['1', 'C', 'M'],
                vec!['2'],
                vec!['3', 'P', 'D', 'N', 'Z'],
            ],
            vec![
                vec!['1', 'C'],
                vec!['2', 'M'],
                vec!['3', 'P', 'D', 'N', 'Z'],
            ],
        ];

        for (m, o) in moves.into_iter().zip(outcomes) {
            stacks.apply_move_9000(&m);

            assert_eq!(o, stacks.0);
        }

        assert_eq!("CMZ", stacks.top_crates().collect::<String>());
    }

    #[test]
    fn apply_moves_9001() {
        let mut stacks = Stacks(vec![
            vec!['1', 'Z', 'N'],
            vec!['2', 'M', 'C', 'D'],
            vec!['3', 'P'],
        ]);

        let moves = vec![
            StackMove {
                count: 1,
                from_stack: 1,
                to_stack: 0,
            },
            StackMove {
                count: 3,
                from_stack: 0,
                to_stack: 2,
            },
            StackMove {
                count: 2,
                from_stack: 1,
                to_stack: 0,
            },
            StackMove {
                count: 1,
                from_stack: 0,
                to_stack: 1,
            },
        ];

        let outcomes = vec![
            vec![
                vec!['1', 'Z', 'N', 'D'],
                vec!['2', 'M', 'C'],
                vec!['3', 'P'],
            ],
            vec![
                vec!['1'],
                vec!['2', 'M', 'C'],
                vec!['3', 'P', 'Z', 'N', 'D'],
            ],
            vec![
                vec!['1', 'M', 'C'],
                vec!['2'],
                vec!['3', 'P', 'Z', 'N', 'D'],
            ],
            vec![
                vec!['1', 'M'],
                vec!['2', 'C'],
                vec!['3', 'P', 'Z', 'N', 'D'],
            ],
        ];

        for (m, o) in moves.into_iter().zip(outcomes) {
            stacks.apply_move_9001(&m);

            assert_eq!(o, stacks.0);
        }

        assert_eq!("MCD", stacks.top_crates().collect::<String>());
    }
}
