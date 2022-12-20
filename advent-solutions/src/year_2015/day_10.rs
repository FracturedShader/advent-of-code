use std::io::BufRead;

fn look_and_say(seq: &str) -> String {
    let mut res = String::new();
    let mut iter = seq.chars();
    let mut current = iter.next().unwrap();
    let mut count = 1;

    for c in iter {
        if c == current {
            count += 1;
        } else {
            res.push_str(&count.to_string());
            res.push(current);

            current = c;
            count = 1;
        }
    }

    res.push_str(&count.to_string());
    res.push(current);

    res
}

pub fn part_01(_reader: Option<impl BufRead>) {
    let mut data = "3113322113".to_string();

    for _ in 0..40 {
        data = look_and_say(&data);
    }

    println!("{}", data.len());
}

pub fn part_02(_reader: Option<impl BufRead>) {
    let mut data = "3113322113".to_string();

    for _ in 0..50 {
        data = look_and_say(&data);
    }

    println!("{}", data.len());
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn look_say() {
        let tests = vec![
            ("1", "11"),
            ("11", "21"),
            ("21", "1211"),
            ("1211", "111221"),
            ("111221", "312211"),
        ];

        for (src, exp) in tests {
            assert_eq!(exp, look_and_say(src));
        }
    }
}
