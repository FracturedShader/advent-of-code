mod day_01;
mod day_02;
mod day_03;

pub const DAYS_SOLVED: i32 = 2;

pub fn run_solution(day: i32, part: i32) {
    match (day, part) {
        (1, 1) => day_01::part_01(),
        (1, 2) => day_01::part_02(),
        (2, 1) => day_02::part_01(),
        (2, 2) => day_02::part_02(),
        (3, 1) => day_03::part_01(),
        (3, 2) => day_03::part_02(),
        _ => {
            eprintln!("Solution not implemented for {day:02}-{part:02}");
        }
    }
}
