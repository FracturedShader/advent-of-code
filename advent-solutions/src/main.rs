mod year_2015;

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

    match year {
        2015 => year_2015::run_solution(day, part),
        _ => eprintln!("No solutions found for the year {}", year),
    }
}
