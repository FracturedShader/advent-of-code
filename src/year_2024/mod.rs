use std::{fs::File, io::BufReader};

mod day_01;
mod day_02;
mod day_03;
mod day_04;
mod day_05;
mod day_06;
mod day_07;
mod day_08;
mod day_09;
mod day_10;
mod day_11;
mod day_12;
mod day_13;

pub const DAYS_SOLVED: i32 = 13;

pub fn run_solution(day: i32, part: i32) {
    let reader = File::open(format!("data/2015-{day:02}.txt")).map(BufReader::new);

    match (day, part) {
        (1, 1) => day_01::part_01(reader),
        (1, 2) => day_01::part_02(reader),
        (2, 1) => day_02::part_01(reader),
        (2, 2) => day_02::part_02(reader),
        (3, 1) => day_03::part_01(reader),
        (3, 2) => day_03::part_02(reader),
        (4, 1) => day_04::part_01(reader),
        (4, 2) => day_04::part_02(reader),
        (5, 1) => day_05::part_01(reader),
        (5, 2) => day_05::part_02(reader),
        (6, 1) => day_06::part_01(reader),
        (6, 2) => day_06::part_02(reader),
        (7, 1) => day_07::part_01(reader),
        (7, 2) => day_07::part_02(reader),
        (8, 1) => day_08::part_01(reader),
        (8, 2) => day_08::part_02(reader),
        (9, 1) => day_09::part_01(reader),
        (9, 2) => day_09::part_02(reader),
        (10, 1) => day_10::part_01(reader),
        (10, 2) => day_10::part_02(reader),
        (11, 1) => day_11::part_01(),
        (11, 2) => day_11::part_02(),
        (12, 1) => day_12::part_01(reader),
        (12, 2) => day_12::part_02(reader),
        (13, 1) => day_13::part_01(reader),
        (13, 2) => day_13::part_02(reader),
        _ => {
            eprintln!("Unknown solution {day:02}-{part:02}");
        }
    }
}
