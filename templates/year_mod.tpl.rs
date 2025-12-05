mod day_01;

pub const DAYS_SOLVED: i32 = 1;

pub fn run_solution(day: i32, part: i32) {
    match (day, part) {
        (1, 1) => day_01::part_01(),
        (1, 2) => day_01::part_02(),
        _ => {
            eprintln!("Solution not implemented for {day:02}-{part:02}");
        }
    }
}
