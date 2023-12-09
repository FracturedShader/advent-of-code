mod year_2015;
mod year_2022;
mod year_2023;

fn main() {
    let mut args = std::env::args();

    if args.len() == 1 {
        println!("This application expects one argument in the form YYYY-DD-PP (year-day-part) and any needed inputs to exist in data/YYYY-DD.txt");
        println!("The following solutions are implemented:");

        let opts: Vec<(i32, &dyn Fn() -> i32)> = vec![
            (2015, &year_2015::days_solved),
            (2022, &year_2022::days_solved),
            (2023, &year_2023::days_solved),
        ];

        for (year, solved) in opts {
            let avail = solved();

            print!(" - {year}: The first ");

            if avail == 1 {
                print!("day is");
            } else {
                print!("{avail} days are");
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
        _ => eprintln!("No solutions found for the year {year}"),
    }
}
