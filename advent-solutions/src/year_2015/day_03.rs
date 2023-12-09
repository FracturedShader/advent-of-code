use std::{collections::HashMap, io::BufRead};

fn move_by(dir: char, curr_x: &mut i32, curr_y: &mut i32) {
    match dir {
        '>' => *curr_x += 1,
        '<' => *curr_x -= 1,
        '^' => *curr_y += 1,
        'v' => *curr_y -= 1,
        _ => unreachable!(),
    };
}

fn visit_houses(directions: &str) -> usize {
    let mut visit_counts = HashMap::new();
    let mut curr_x = 0;
    let mut curr_y = 0;

    visit_counts.insert((curr_x, curr_y), 1);

    for dir in directions.chars() {
        move_by(dir, &mut curr_x, &mut curr_y);

        let count = visit_counts.entry((curr_x, curr_y)).or_insert(0);

        *count += 1;
    }

    visit_counts.len()
}

fn visit_houses_split(directions: &str) -> usize {
    let mut visit_counts = HashMap::new();
    let mut curr_x = 0;
    let mut curr_y = 0;

    visit_counts.insert((curr_x, curr_y), 2);

    for dir in directions.chars().step_by(2) {
        move_by(dir, &mut curr_x, &mut curr_y);

        let count = visit_counts.entry((curr_x, curr_y)).or_insert(0);

        *count += 1;
    }

    curr_x = 0;
    curr_y = 0;

    for dir in directions.chars().skip(1).step_by(2) {
        move_by(dir, &mut curr_x, &mut curr_y);

        let count = visit_counts.entry((curr_x, curr_y)).or_insert(0);

        *count += 1;
    }

    visit_counts.len()
}

fn for_instruction_set<F>(reader: impl BufRead, visit_method: F)
where
    F: Fn(&str) -> usize,
{
    for line in reader.lines().map_while(Result::ok) {
        println!("Houses visited at least once: {}", visit_method(&line));
    }
}

pub fn part_01(reader: Option<impl BufRead>) {
    for_instruction_set(reader.unwrap(), visit_houses);
}

pub fn part_02(reader: Option<impl BufRead>) {
    for_instruction_set(reader.unwrap(), visit_houses_split);
}

#[cfg(test)]
mod test {
    use super::{visit_houses, visit_houses_split};

    #[test]
    fn straight_delivery() {
        let tests = vec![(">", 2), ("^>v<", 4), ("^v^v^v^v^v", 2)];

        for (directions, expected) in tests {
            let res = visit_houses(directions);

            assert_eq!(expected, res);
        }
    }

    #[test]
    fn split_delivery() {
        let tests = vec![("^>v<", 3), ("^v^v^v^v^v", 11)];

        for (directions, expected) in tests {
            let res = visit_houses_split(directions);

            assert_eq!(expected, res);
        }
    }
}
