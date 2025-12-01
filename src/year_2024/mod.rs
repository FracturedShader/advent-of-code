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
    match (day, part) {
        (1, 1) => day_01::part_01(),
        (1, 2) => day_01::part_02(),
        (2, 1) => day_02::part_01(),
        (2, 2) => day_02::part_02(),
        (3, 1) => day_03::part_01(),
        (3, 2) => day_03::part_02(),
        (4, 1) => day_04::part_01(),
        (4, 2) => day_04::part_02(),
        (5, 1) => day_05::part_01(),
        (5, 2) => day_05::part_02(),
        (6, 1) => day_06::part_01(),
        (6, 2) => day_06::part_02(),
        (7, 1) => day_07::part_01(),
        (7, 2) => day_07::part_02(),
        (8, 1) => day_08::part_01(),
        (8, 2) => day_08::part_02(),
        (9, 1) => day_09::part_01(),
        (9, 2) => day_09::part_02(),
        (10, 1) => day_10::part_01(),
        (10, 2) => day_10::part_02(),
        (11, 1) => day_11::part_01(),
        (11, 2) => day_11::part_02(),
        (12, 1) => day_12::part_01(),
        (12, 2) => day_12::part_02(),
        (13, 1) => day_13::part_01(),
        (13, 2) => day_13::part_02(),
        _ => {
            eprintln!("Solution not implemented for {day:02}-{part:02}");
        }
    }
}
