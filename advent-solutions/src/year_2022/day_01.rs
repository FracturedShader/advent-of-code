use std::io::BufRead;

/// Helper struct to make generating an unknown number of Elves more idiomatic by leveraging the
/// fact that [`Elf::parse_one`] modifies the iterator and returns an `Option<Elf>`.
struct ElfGenerator<S, I>(I)
where
    S: AsRef<str>,
    I: Iterator<Item = S>;

impl<S, I> Iterator for ElfGenerator<S, I>
where
    S: AsRef<str>,
    I: Iterator<Item = S>,
{
    type Item = Elf;

    fn next(&mut self) -> Option<Self::Item> {
        Elf::parse_one(&mut self.0)
    }
}

/// Elves carry a number of items, each with a known caloric value
#[derive(Clone, Debug)]
struct Elf {
    items: Vec<u32>,
}

impl Elf {
    /// An Elf is defined as a sequence of newline separated numbers (their items) a blank line or
    /// EOF indicates the end of an Elf's list
    fn parse_one<S, I>(lines: &mut I) -> Option<Self>
    where
        S: AsRef<str>,
        I: Iterator<Item = S>,
    {
        let items: Vec<_> = lines
            .map_while(|l| {
                if l.as_ref().is_empty() {
                    None
                } else {
                    Some(
                        l.as_ref()
                            .parse::<u32>()
                            .expect("Input lines should only be empty or numbers"),
                    )
                }
            })
            .collect();

        if items.is_empty() {
            None
        } else {
            Some(Self { items })
        }
    }

    /// Leverages [`ElfGenerator`] to trivially collect all available Elves in the input into a
    /// `Vec<Elf>`
    fn parse_all<S, I>(lines: I) -> Vec<Self>
    where
        S: AsRef<str>,
        I: Iterator<Item = S>,
    {
        ElfGenerator(lines).collect()
    }

    fn calories_carried(&self) -> u32 {
        self.items.iter().sum()
    }
}

/// Parses the problem input and returns the Elf objects parsed as well as their cummulative
/// carried calories.
/// Panics if reader is `None` as a convenience for otherwise identical `expect`
fn parse_input(reader: Option<impl BufRead>) -> (Vec<Elf>, Vec<u32>) {
    let elves = Elf::parse_all(
        reader
            .expect("This problem requires data input")
            .lines()
            .filter_map(Result::ok),
    );

    let sum_calories = elves.iter().map(Elf::calories_carried).collect();

    (elves, sum_calories)
}

pub fn part_01(reader: Option<impl BufRead>) {
    let (_, sum_calories) = parse_input(reader);

    println!(
        "Most calories carried by an Elf: {}",
        sum_calories.iter().max().unwrap()
    );
}

pub fn part_02(reader: Option<impl BufRead>) {
    let (_, mut sum_calories) = parse_input(reader);

    sum_calories.sort_by(|a, b| b.partial_cmp(a).unwrap());

    println!(
        "Total calories carried by the three Elves carrying the most calories: {}",
        sum_calories[..3].iter().sum::<u32>()
    );
}

#[cfg(test)]
mod test {
    use std::io::BufReader;

    use super::*;

    /// Verify that parsing all elves and getting their total carried calories works as intended
    #[test]
    fn parse_sum() {
        let input = r#"1000
2000
3000

4000

5000
6000

7000
8000
9000

10000"#;

        let (_, sum_calories) = parse_input(Some(BufReader::new(input.as_bytes())));

        assert_eq!(vec![6000, 4000, 11000, 24000, 10000], sum_calories);
    }
}
