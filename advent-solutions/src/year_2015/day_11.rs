use std::{collections::HashSet, io::BufRead};

fn increment_password(pass: &mut [u8]) {
    const A: u8 = b'a';
    const Z: u8 = b'z';

    for c in pass.iter_mut().rev() {
        if *c == Z {
            *c = A;
        } else {
            *c += 1;
            break;
        }
    }
}

fn replace_invalid(pass: &mut [u8]) {
    const A: u8 = b'a';
    const INVALID: &[u8] = &[b'i', b'l', b'o'];

    let mut it = pass.iter_mut();

    while let Some(c) = it.next() {
        if INVALID.contains(c) {
            *c += 1;

            for c in it.by_ref() {
                *c = A;
            }
        }
    }
}

fn has_straight(pass: &[u8]) -> bool {
    for w in pass.windows(3) {
        let (p, c, n) = (w[0], w[1], w[2]);

        if (c == (p + 1)) && (n == (c + 1)) {
            return true;
        }
    }

    false
}

fn has_two_pairs(pass: &[u8]) -> bool {
    let mut seen_pairs = HashSet::new();

    for (p, c) in pass.windows(2).map(|w| (w[0], w[1])) {
        if c == p && seen_pairs.insert(c) && (seen_pairs.len() > 1) {
            return true;
        }
    }

    false
}

fn next_valid_password(pass: &str) -> String {
    let mut pass_bytes = pass.to_owned().into_bytes();

    loop {
        increment_password(&mut pass_bytes);
        replace_invalid(&mut pass_bytes);

        if has_straight(&pass_bytes) && has_two_pairs(&pass_bytes) {
            return unsafe { String::from_utf8_unchecked(pass_bytes) };
        }
    }
}

pub fn part_01(_reader: Option<impl BufRead>) {
    println!("{}", next_valid_password("hepxcrrq"));
}

pub fn part_02(_reader: Option<impl BufRead>) {
    println!("{}", next_valid_password("hepxcrrq"));
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn detect_straight() {
        assert!(has_straight(b"hijklmmn"));
    }

    #[test]
    fn detect_two_pairs() {
        assert!(has_two_pairs(b"abbceffg"));
    }

    #[test]
    fn successful_next() {
        assert_eq!(next_valid_password("abcdefgh"), "abcdffaa");
        assert_eq!(next_valid_password("ghijklmn"), "ghjaabcc");
    }
}
