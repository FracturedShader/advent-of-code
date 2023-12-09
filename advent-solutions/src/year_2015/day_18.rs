use std::{
    fmt::{Display, Write},
    io::BufRead,
};

struct World {
    a: Vec<Vec<bool>>,
    b: Vec<Vec<bool>>,
    a_src: bool,
    dims: (usize, usize),
}

impl World {
    fn from_data<S, I>(lines: I) -> Self
    where
        S: AsRef<str>,
        I: Iterator<Item = S>,
    {
        let b = lines
            .map(|l| {
                l.as_ref()
                    .chars()
                    .map(|c| match c {
                        '#' => true,
                        '.' => false,
                        _ => panic!("Invalid character"),
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let dims = (b.len(), b[0].len());

        World {
            a: b.clone(),
            b,
            a_src: true,
            dims,
        }
    }

    fn count(&self) -> usize {
        self.current()
            .iter()
            .flat_map(|r| r.iter().map(|&b| usize::from(b)))
            .sum()
    }

    fn current(&self) -> &Vec<Vec<bool>> {
        if self.a_src {
            &self.a
        } else {
            &self.b
        }
    }

    fn enable_corners(&mut self) {
        let src = if self.a_src { &mut self.a } else { &mut self.b };
        let last = (self.dims.0 - 1, self.dims.1 - 1);

        src[0][0] = true;
        src[0][last.1] = true;
        src[last.0][0] = true;
        src[last.0][last.1] = true;
    }

    fn step(&mut self) {
        let (src, dest) = if self.a_src {
            (&self.a, &mut self.b)
        } else {
            (&self.b, &mut self.a)
        };

        let idxs = (0isize..9)
            .filter_map(|i| {
                if i == 4 {
                    None
                } else {
                    Some((i / 3 - 1, i % 3 - 1))
                }
            })
            .collect::<Vec<_>>();

        for (i, r) in dest.iter_mut().enumerate() {
            for (j, c) in r.iter_mut().enumerate() {
                let prev = src[i][j];

                let i = isize::try_from(i).expect("board should fit in isize");
                let j = isize::try_from(j).expect("board should fit in isize");

                let living_neighbors = idxs
                    .iter()
                    .filter_map(|&(ri, rj)| {
                        src.get(usize::try_from(i + ri).expect("neighbor should map to usize"))
                            .and_then(|sr| {
                                sr.get(
                                    usize::try_from(j + rj).expect("neighbor should map to usize"),
                                )
                            })
                    })
                    .map(|&b| i32::from(b))
                    .sum::<i32>();

                *c = matches!((prev, living_neighbors), (true, 2 | 3) | (false, 3));
            }
        }

        self.a_src = !self.a_src;
    }
}

impl Display for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for r in self.current() {
            for c in r {
                if *c {
                    f.write_char('#')?;
                } else {
                    f.write_char('.')?;
                }
            }

            f.write_char('\n')?;
        }

        Ok(())
    }
}

pub fn part_01(reader: Option<impl BufRead>) {
    let mut w = World::from_data(reader.unwrap().lines().map_while(Result::ok));

    for _ in 0..100 {
        w.step();
    }

    println!("{}", w.count());
}

pub fn part_02(reader: Option<impl BufRead>) {
    let mut w = World::from_data(reader.unwrap().lines().map_while(Result::ok));

    w.enable_corners();

    for _ in 0..100 {
        w.step();
        w.enable_corners();
    }

    println!("{}", w.count());
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn game_of_life_step() {
        let initial = r".#.#.#
...##.
#....#
..#...
#.#..#
####..";

        let expected = r"......
......
..##..
..##..
......
......";

        let mut initial = World::from_data(initial.lines());
        let expected = World::from_data(expected.lines());

        for _ in 0..4 {
            println!("{initial}");
            initial.step();
        }

        println!("{initial}");

        assert_eq!(initial.current(), expected.current());
    }

    #[test]
    fn step_broken_corners() {
        let initial = r"##.#.#
...##.
#....#
..#...
#.#..#
####.#";

        let expected = r"##.###
.##..#
.##...
.##...
#.#...
##...#";

        let mut initial = World::from_data(initial.lines());
        let expected = World::from_data(expected.lines());

        for _ in 0..5 {
            println!("{initial}");
            initial.step();
            initial.enable_corners();
        }

        println!("{initial}");

        assert_eq!(initial.current(), expected.current());
    }
}
