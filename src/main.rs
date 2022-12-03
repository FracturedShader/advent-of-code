use std::{collections::HashMap, fs::File, io::BufReader};

mod year_2015;

type PartMethod = fn(Option<BufReader<File>>);

fn main() {
    let parts = std::env::args()
        .nth(1)
        .unwrap()
        .split('-')
        .filter_map(|p| p.parse().ok())
        .collect::<Vec<i32>>();

    let year = parts[0];
    let day = parts[1];
    let part = parts[2];

    let mut cases: HashMap<(i32, i32, i32), PartMethod> = HashMap::new();

    cases.insert((2015, 1, 1), year_2015::day_01::part_01);
    cases.insert((2015, 1, 2), year_2015::day_01::part_02);
    cases.insert((2015, 2, 1), year_2015::day_02::part_01);
    cases.insert((2015, 2, 2), year_2015::day_02::part_02);
    cases.insert((2015, 3, 1), year_2015::day_03::part_01);
    cases.insert((2015, 3, 2), year_2015::day_03::part_02);
    cases.insert((2015, 4, 1), year_2015::day_04::part_01);
    cases.insert((2015, 4, 2), year_2015::day_04::part_02);
    cases.insert((2015, 5, 1), year_2015::day_05::part_01);
    cases.insert((2015, 5, 2), year_2015::day_05::part_02);
    cases.insert((2015, 6, 1), year_2015::day_06::part_01);
    cases.insert((2015, 6, 2), year_2015::day_06::part_02);
    cases.insert((2015, 7, 1), year_2015::day_07::part_01);
    cases.insert((2015, 7, 2), year_2015::day_07::part_02);
    cases.insert((2015, 8, 1), year_2015::day_08::part_01);
    cases.insert((2015, 8, 2), year_2015::day_08::part_02);
    cases.insert((2015, 9, 1), year_2015::day_09::part_01);
    cases.insert((2015, 9, 2), year_2015::day_09::part_02);
    cases.insert((2015, 10, 1), year_2015::day_10::part_01);
    cases.insert((2015, 10, 2), year_2015::day_10::part_02);
    cases.insert((2015, 11, 1), year_2015::day_11::part_01);
    cases.insert((2015, 11, 2), year_2015::day_11::part_02);
    cases.insert((2015, 12, 1), year_2015::day_12::part_01);
    cases.insert((2015, 12, 2), year_2015::day_12::part_02);
    cases.insert((2015, 13, 1), year_2015::day_13::part_01);
    cases.insert((2015, 13, 2), year_2015::day_13::part_02);
    cases.insert((2015, 14, 1), year_2015::day_14::part_01);
    cases.insert((2015, 14, 2), year_2015::day_14::part_02);
    cases.insert((2015, 15, 1), year_2015::day_15::part_01);
    cases.insert((2015, 15, 2), year_2015::day_15::part_02);
    cases.insert((2015, 16, 1), year_2015::day_16::part_01);
    cases.insert((2015, 16, 2), year_2015::day_16::part_02);
    cases.insert((2015, 17, 1), year_2015::day_17::part_01);
    cases.insert((2015, 17, 2), year_2015::day_17::part_02);
    cases.insert((2015, 18, 1), year_2015::day_18::part_01);
    cases.insert((2015, 18, 2), year_2015::day_18::part_02);
    cases.insert((2015, 19, 1), year_2015::day_19::part_01);
    cases.insert((2015, 19, 2), year_2015::day_19::part_02);

    let reader = File::open(format!("data/{}-{:02}.txt", year, day))
        .map(BufReader::new)
        .ok();

    let key = (year, day, part);

    if let Some(part) = cases.get(&key) {
        part(reader);
    } else {
        eprintln!("{} day {} part {} does not exist!", year, day, part);
    }
}
