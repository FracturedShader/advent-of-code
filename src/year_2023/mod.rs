use std::{fs::File, io::BufReader};

mod day_01;
mod day_02;

pub const DAYS_SOLVED: i32 = 2;

pub fn run_solution(day: i32, part: i32) {
    let reader = File::open(format!("data/2015-{day:02}.txt")).map(BufReader::new);

    match (day, part) {
        (1, 1) => day_01::part_01(reader),
        (1, 2) => day_01::part_02(reader),
        (2, 1) => day_02::part_01(reader),
        (2, 2) => day_02::part_02(reader),
        _ => {
            eprintln!("Unknown solution {day:02}-{part:02}");
        }
    }
}
