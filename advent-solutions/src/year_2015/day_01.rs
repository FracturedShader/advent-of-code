use std::io::BufRead;

struct FloorTraversalResult {
    end_floor: i32,
    first_saw_basement: usize,
}

fn walk_floors(instructions: &str) -> FloorTraversalResult {
    let mut first_basement = true;
    let mut first_saw_basement = 0;

    let end_floor = instructions.char_indices().fold(0, |acc, (idx, elem)| {
        let curr_floor = acc
            + match elem {
                '(' => 1,
                ')' => -1,
                _ => 0,
            };

        if first_basement && (curr_floor == -1) {
            first_basement = false;
            first_saw_basement = idx + 1;
        }

        curr_floor
    });

    FloorTraversalResult {
        end_floor,
        first_saw_basement,
    }
}

fn for_each_result<F>(reader: impl BufRead, callback: F)
where
    F: Fn(&FloorTraversalResult) + 'static,
{
    for line in reader.lines().map_while(Result::ok) {
        let walk_results = walk_floors(&line);

        callback(&walk_results);
    }
}

pub fn part_01(reader: Option<impl BufRead>) {
    for_each_result(reader.unwrap(), |walk_results| {
        println!("Ended up on floor {}", walk_results.end_floor);
    });
}

pub fn part_02(reader: Option<impl BufRead>) {
    for_each_result(reader.unwrap(), |walk_results| {
        println!(
            "First got to the basement at step {}",
            walk_results.first_saw_basement
        );
    });
}

#[cfg(test)]
mod test {
    use super::{walk_floors, FloorTraversalResult};

    #[test]
    fn end_floor() {
        let floor_tests = vec![
            ("(())", 0),
            ("()()", 0),
            ("(((", 3),
            ("(()(()(", 3),
            ("))(((((", 3),
            ("())", -1),
            ("))(", -1),
            (")))", -3),
            (")())())", -3),
        ];

        for (directions, expected) in floor_tests {
            let FloorTraversalResult { end_floor, .. } = walk_floors(directions);
            assert_eq!(end_floor, expected);
        }
    }

    #[test]
    fn first_basement_position() {
        let basement_tests = vec![(")", 1), ("()())", 5)];

        for (directions, expected) in basement_tests {
            let FloorTraversalResult {
                end_floor: _,
                first_saw_basement,
            } = walk_floors(directions);
            assert_eq!(first_saw_basement, expected);
        }
    }
}
