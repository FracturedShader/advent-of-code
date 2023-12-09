use std::{collections::HashMap, io::BufRead};

fn draw_possible(draw: &str, bag: &HashMap<&str, i32>) -> bool {
    draw.split(',').all(|combo| {
        let (num, color) = combo
            .trim()
            .split_once(' ')
            .expect("each draw should be of the format '# color(; # color)*");

        if let Some(val) = bag.get(color) {
            num.parse::<i32>()
                .expect("each draw should be of the format '# color(; # color)*")
                <= *val
        } else {
            false
        }
    })
}

fn possible_game(line: &str, bag: &HashMap<&str, i32>) -> Option<i32> {
    let (game, draws) = line
        .split_once(':')
        .expect("each line should be of the format 'Game ID: draws'");

    if draws.split(';').all(|draw| draw_possible(draw, bag)) {
        Some(
            game.split(' ')
                .nth(1)
                .expect("each line should be of the format 'Game ID: draws'")
                .parse()
                .expect("game ID should be a number"),
        )
    } else {
        None
    }
}

fn draw_combos(draws: &str) -> impl Iterator<Item = (&str, i64)> {
    draws.split(';').flat_map(|draw| {
        draw.split(',').map(|combo| {
            let (num, color) = combo
                .trim()
                .split_once(' ')
                .expect("each draw should be of the format '# color(; # color)*");

            (
                color,
                num.parse()
                    .expect("each draw should be of the format '# color(; # color)*"),
            )
        })
    })
}

fn game_power(line: &str) -> i64 {
    let mut bag = HashMap::<&str, i64>::new();

    let (_, draws) = line
        .split_once(':')
        .expect("each line should be of the format 'Game ID: draws'");

    for (color, count) in draw_combos(draws) {
        bag.entry(color)
            .and_modify(|v| {
                *v = count.max(*v);
            })
            .or_insert(count);
    }

    bag.values().product()
}

pub fn part_01(reader: Option<impl BufRead>) {
    let bag = HashMap::from([("red", 12), ("green", 13), ("blue", 14)]);

    let possible_id_sum = reader
        .expect("data should be available for this problem")
        .lines()
        .map_while(std::io::Result::ok)
        .filter_map(|l| possible_game(&l, &bag))
        .sum::<i32>();

    print!("Sum of IDs for possible games: {possible_id_sum}");
}

pub fn part_02(reader: Option<impl BufRead>) {
    let power_sum = reader
        .expect("data should be available for this problem")
        .lines()
        .map_while(std::io::Result::ok)
        .map(|l| game_power(&l))
        .sum::<i64>();

    print!("Sum of set powers: {power_sum}");
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, io::BufReader};

    use super::*;

    #[test]
    fn possible_games() {
        let input = r#"Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green
Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue
Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red
Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red
Game 5: 6 red, 1 blue, 3 green; 2 blue, 1 red, 2 green"#;

        let bag = HashMap::from([("red", 12), ("green", 13), ("blue", 14)]);

        let possible = BufReader::new(input.as_bytes())
            .lines()
            .map_while(std::io::Result::ok)
            .filter_map(|l| possible_game(&l, &bag))
            .collect::<Vec<_>>();

        assert_eq!(possible, vec![1, 2, 5]);
    }

    #[test]
    fn power_of_games() {
        let input = r#"Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green
Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue
Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red
Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red
Game 5: 6 red, 1 blue, 3 green; 2 blue, 1 red, 2 green"#;

        let powers = BufReader::new(input.as_bytes())
            .lines()
            .map_while(std::io::Result::ok)
            .map(|l| game_power(&l))
            .collect::<Vec<_>>();

        assert_eq!(powers, vec![48, 12, 1560, 630, 36]);
    }
}
