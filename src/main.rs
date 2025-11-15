mod common;
mod year_2015;
mod year_2022;
mod year_2023;
mod year_2024;

fn main() {
    let mut args = std::env::args();

    if args.len() == 1 {
        println!("This application expects one argument in the form YYYY-DD-PP (year-day-part) and any needed inputs to exist in data/YYYY-DD.txt");
        println!("The following solutions are implemented:");

        for (year, solved) in [
            (2015, year_2015::DAYS_SOLVED),
            (2022, year_2022::DAYS_SOLVED),
            (2023, year_2023::DAYS_SOLVED),
            (2024, year_2024::DAYS_SOLVED),
        ] {
            print!(" - {year}: The first ");

            if solved == 1 {
                print!("day is");
            } else {
                print!("{solved} days are");
            }

            println!(" complete.");
        }

        return;
    }

    let parts = args
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
        2022 => year_2022::run_solution(day, part),
        2023 => year_2023::run_solution(day, part),
        2024 => year_2024::run_solution(day, part),
        _ => eprintln!("No solutions found for the year {year}"),
    }
}
