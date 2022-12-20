use std::io::BufRead;

fn str_lengths(input: &str) -> (usize, usize) {
    let code_len = input.len();
    let mut mem_len = 0;

    let mut iter = input.chars();

    while let Some(c) = iter.next() {
        if c == '\\' {
            let nc = iter.next().unwrap();

            if nc == 'x' {
                iter.next();
                iter.next();
            }
        }

        mem_len += 1;
    }

    mem_len -= 2; // First/last '"'

    (code_len, mem_len)
}

fn str_encode_lengths(input: &str) -> (usize, usize) {
    let code_len = input.len();
    let encode_len = input
        .chars()
        .map(|c| match c {
            '"' | '\\' => 2,
            _ => 1,
        })
        .sum::<usize>()
        + 2; // + 2 for end '"'

    (code_len, encode_len)
}

pub fn part_01(reader: Option<impl BufRead>) {
    let total_diff = reader
        .unwrap()
        .lines()
        .filter_map(|l| l.ok())
        .map(|l| {
            let lengths = str_lengths(&l);

            lengths.0 - lengths.1
        })
        .sum::<usize>();

    println!("Total diff: {}", total_diff);
}

pub fn part_02(reader: Option<impl BufRead>) {
    let total_diff = reader
        .unwrap()
        .lines()
        .filter_map(|l| l.ok())
        .map(|l| {
            let lengths = str_encode_lengths(&l);

            lengths.1 - lengths.0
        })
        .sum::<usize>();

    println!("Total diff: {}", total_diff);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn mem_lengths() {
        let cases = vec![
            (r#""""#, (2, 0)),
            (r#""abc""#, (5, 3)),
            (r#""aaa\"aaa""#, (10, 7)),
            (r#""\x27""#, (6, 1)),
        ];

        for (s, l) in cases {
            assert_eq!(l, str_lengths(s));
        }
    }

    #[test]
    fn encode_lengths() {
        let cases = vec![
            (r#""""#, (2, 6)),
            (r#""abc""#, (5, 9)),
            (r#""aaa\"aaa""#, (10, 16)),
            (r#""\x27""#, (6, 11)),
        ];

        for (s, l) in cases {
            assert_eq!(l, str_encode_lengths(s));
        }
    }
}
