use std::{collections::HashSet, io::BufRead};

fn string_is_nice(input: &str) -> bool {
    let vowels = "aeiou";
    let disallowed = [('a', 'b'), ('c', 'd'), ('p', 'q'), ('x', 'y')];
    let mut vowel_count = 0;
    let mut double_count = 0;
    let mut prev_char = '\0';

    for c in input.chars() {
        if c == prev_char {
            double_count += 1;
        }

        if vowels.contains(c) {
            vowel_count += 1;
        }

        if disallowed.contains(&(prev_char, c)) {
            return false;
        }

        prev_char = c;
    }

    (vowel_count > 2) && (double_count != 0)
}

fn string_is_nicer(input: &str) -> bool {
    let mut seen_pairs = HashSet::new();

    let mut pair_condition_met = false;
    let mut alternating_condition_met = false;
    let mut prev_char = '\0';

    for (l, r) in input.chars().zip(input.chars().skip(1)) {
        pair_condition_met = pair_condition_met || seen_pairs.contains(&(l, r));

        alternating_condition_met = alternating_condition_met || (prev_char == r);

        if alternating_condition_met && pair_condition_met {
            break;
        }

        seen_pairs.insert((prev_char, l));
        prev_char = l;
    }

    alternating_condition_met && pair_condition_met
}

fn count_strings<F>(reader: impl BufRead, mut tester: F) -> usize
where
    F: FnMut(&str) -> bool + 'static,
{
    reader
        .lines()
        .map_while(Result::ok)
        .filter(|s| tester(s))
        .count()
}

pub fn part_01(reader: Option<impl BufRead>) {
    println!(
        "Total nice strings: {}",
        count_strings(reader.unwrap(), string_is_nice)
    );
}

pub fn part_02(reader: Option<impl BufRead>) {
    println!(
        "Total nice strings: {}",
        count_strings(reader.unwrap(), string_is_nicer)
    );
}

#[cfg(test)]
mod tests {
    use super::{string_is_nice, string_is_nicer};

    #[test]
    fn nice_strings() {
        assert!(string_is_nice("ugknbfddgicrmopn"));
        assert!(string_is_nice("aaa"));
    }

    #[test]
    fn not_nice_strings() {
        assert!(!string_is_nice("jchzalrnumimnmhp"));
        assert!(!string_is_nice("haegwjzuvuyypxyu"));
        assert!(!string_is_nice("dvszwmarrgswjxmb"));
    }

    #[test]
    fn nicer_strings() {
        assert!(string_is_nicer("qjhvhtzxzqqjkmpb"));
        assert!(string_is_nicer("xxyxx"));
    }

    #[test]
    fn not_nicer_strings() {
        assert!(!string_is_nicer("uurcxstgmygtbstg"));
        assert!(!string_is_nicer("ieodomkazucvgmuy"));
    }
}
