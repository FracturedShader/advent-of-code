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
mod day_14;
mod day_15;
mod day_16;
mod day_17;
mod day_18;
mod day_19;
mod day_20;
mod day_21;

pub const DAYS_SOLVED: i32 = 21;

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
        (14, 1) => day_14::part_01(),
        (14, 2) => day_14::part_02(),
        (15, 1) => day_15::part_01(),
        (15, 2) => day_15::part_02(),
        (16, 1) => day_16::part_01(),
        (16, 2) => day_16::part_02(),
        (17, 1) => day_17::part_01(),
        (17, 2) => day_17::part_02(),
        (18, 1) => day_18::part_01(),
        (18, 2) => day_18::part_02(),
        (19, 1) => day_19::part_01(),
        (19, 2) => day_19::part_02(),
        (20, 1) => day_20::part_01(),
        (20, 2) => day_20::part_02(),
        (21, 1) => day_21::part_01(),
        (21, 2) => day_21::part_02(),
        _ => {
            eprintln!("Solution not implemented for {day:02}-{part:02}");
        }
    }
}
