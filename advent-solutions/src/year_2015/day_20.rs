use std::{io::BufRead, sync::atomic::AtomicU32};

/// Determine how many presents a house would receive if the elves have no problem going to
/// infinite houses.
fn presents_received_inf(house: u32) -> u64 {
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    let root = f64::from(house).sqrt() as u64;
    let house = u64::from(house);
    let perfect_root = (root * root) == house;
    let mut sum = 0;

    for i in 1..=root {
        if (house % i) == 0 {
            sum += i;

            if !perfect_root || i != root {
                sum += house / i;
            }
        }
    }

    sum * 10
}

/// Determine howe many presents a house would receive if the elves only go to 50 houses.
fn presents_received_limited(house: u32) -> u64 {
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    let root = f64::from(house).sqrt() as u64;
    let house = u64::from(house);
    let perfect_root = (root * root) == house;
    let mut sum = 0;

    for i in 1..=root {
        if (house % i) == 0 {
            if (i * 50) >= house {
                sum += i;
            }

            if !perfect_root || i != root {
                let i2 = house / i;

                if (i2 * 50) >= house {
                    sum += i2;
                }
            }
        }
    }

    sum * 11
}

/// Find the first house, starting at `start_point` and checking every `increment` house, where
/// `present_count >= threshold`. Record the value in `first_house` if it's the better option.
///
/// **Exit conditions**:
/// - The thread finds an answer and updates `first_house`
/// - The thread would check a house higher than `first_house`
fn find_best_house<F>(
    start_point: u32,
    increment: u32,
    threshold: u64,
    present_count: &'static F,
    first_house: &AtomicU32,
) where
    F: Fn(u32) -> u64 + Sync + 'static,
{
    use std::sync::atomic::Ordering;

    let mut house = start_point;

    loop {
        // Data dependencies make Relaxed ordering okay
        let mut curr_best = first_house.load(Ordering::Relaxed);

        // Another thread found a better house already, stop here
        if house > curr_best {
            return;
        }

        let presents = present_count(house);

        if presents >= threshold {
            // Standard CAS loop. Data dependencies make Relaxed ordering okay
            while let Err(old) = first_house.compare_exchange_weak(
                curr_best,
                house,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                curr_best = old;

                // Let the compiler know what we're up to
                std::hint::spin_loop();

                // Another thread snuck in a better house; stop trying to swap
                if house > curr_best {
                    return;
                }
            }

            // This thread found an answer, stop here
            return;
        }

        house += increment;
    }
}

/// Uses multiple threads to find the first number where `present_count` returns a value greater
/// than `threshold`. Threads leap-frog one another and atomically coordinate to all stop once the
/// best answer has been found.
fn parallel_first<F>(present_count: &'static F, threshold: u64) -> u32
where
    F: Fn(u32) -> u64 + Sync + 'static,
{
    let num_threads = if let Ok(nt) = std::thread::available_parallelism() {
        nt.get()
    } else {
        4
    };
    let num_threads = u32::try_from(num_threads).unwrap_or(u32::MAX);

    let first_house = AtomicU32::new(u32::MAX);
    let first_house = &first_house;

    // Start somewhere closer to the answer
    let start_point =
        1 + u32::try_from(threshold / 80).expect("expected {target}/80 to fit in a u32");

    std::thread::scope(|s| {
        for offset in 0..num_threads {
            s.spawn(move || {
                find_best_house(
                    start_point + offset,
                    num_threads,
                    threshold,
                    present_count,
                    first_house,
                );
            });
        }
    });

    // All threads have completed, Relaxed ordering okay
    first_house.load(std::sync::atomic::Ordering::Relaxed)
}

/// Helper function to pull the target number from command line args.
fn parse_args() -> u64 {
    std::env::args()
        .nth(2)
        .expect("2015-20 needs a number argument to run")
        .parse::<u64>()
        .expect("provided argument should be a positive integer")
}

#[allow(clippy::needless_pass_by_value)]
pub fn part_01(_reader: Option<impl BufRead>) {
    let target = parse_args();
    let first = parallel_first(&presents_received_inf, target);

    println!("First house to receive {target} presents: {first}");
}

#[allow(clippy::needless_pass_by_value)]
pub fn part_02(_reader: Option<impl BufRead>) {
    let target = parse_args();
    let first = parallel_first(&presents_received_limited, target);

    println!("First house to receive {target} presents: {first}");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn house_presents() {
        let cases = vec![
            (1, 10),
            (2, 30),
            (3, 40),
            (4, 70),
            (5, 60),
            (6, 120),
            (7, 80),
            (8, 150),
            (9, 130),
        ];

        for (house, res) in cases {
            assert_eq!(presents_received_inf(house), res);
        }
    }
}
