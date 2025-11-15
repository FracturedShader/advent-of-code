use std::collections::HashMap;

fn log10_ceil(v: u64) -> u32 {
    let mut v = v;
    let mut n = 0;

    while v > 0 {
        n += 1;
        v /= 10;
    }

    n
}

/// Possible outcomes for "blinking" at a stone.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum BlinkResult {
    Changed(u64),
    Split(u64, u64),
}

impl BlinkResult {
    fn len(self) -> usize {
        match self {
            BlinkResult::Changed(_) => 1,
            BlinkResult::Split(_, _) => 2,
        }
    }
}

/// Applies the puzzle "blink" rules to a single stone.
fn blink_stone(s: u64) -> BlinkResult {
    if s == 0 {
        BlinkResult::Changed(1)
    } else {
        let digits = log10_ceil(s);

        if digits.is_multiple_of(2) {
            let factor = 10u64.pow(digits / 2);

            BlinkResult::Split(s / factor, s % factor)
        } else {
            BlinkResult::Changed(s * 2024)
        }
    }
}

/// Determine the final number of stones after "blinking" `n` times at `stones`. Uses a top-down
/// memoized dynamic programming approach to cache as many results as possible.
fn blink_n(stones: &[u64], n: u32) -> usize {
    let mut solved: HashMap<(u64, u32), usize> = HashMap::new();
    let mut unsolved = stones.iter().map(|&s| (s, n)).collect::<Vec<_>>();

    while let Some(pair) = unsolved.pop() {
        if solved.contains_key(&pair) {
            // Already solved. Nothing to do.
            continue;
        }

        if pair.1 == 1 {
            // Base case: insert how many stones there are after one blink
            solved.insert(pair, blink_stone(pair.0).len());
        } else {
            let (s, n) = pair;

            // Results of blink either exist and can be summed, or need to be calculated
            match blink_stone(s) {
                BlinkResult::Changed(sn) => {
                    let snp = (sn, n - 1);

                    if let Some(&c) = solved.get(&snp) {
                        solved.insert((s, n), c);
                    } else {
                        unsolved.push((s, n));
                        unsolved.push(snp);
                    }
                }
                BlinkResult::Split(sl, sr) => {
                    let left = (sl, n - 1);
                    let right = (sr, n - 1);

                    if let (Some(&cl), Some(&cr)) = (solved.get(&left), solved.get(&right)) {
                        solved.insert((s, n), cl + cr);
                    } else {
                        unsolved.push((s, n));
                        unsolved.push(left);
                        unsolved.push(right);
                    }
                }
            }
        }
    }

    stones.iter().map(|&s| solved.get(&(s, n)).unwrap()).sum()
}

pub fn part_01() {
    let arrangement = vec![6_563_348, 67, 395, 0, 6, 4425, 89567, 739_318];
    let final_size = blink_n(&arrangement, 25);

    println!("Number of stones: {final_size}");
}

pub fn part_02() {
    let arrangement = vec![6_563_348, 67, 395, 0, 6, 4425, 89567, 739_318];
    let final_size = blink_n(&arrangement, 75);

    println!("Number of stones: {final_size}");
}

#[cfg(test)]
mod test {
    use super::*;

    // Original implementation that directly mutated a `Vec<u64>` in place.
    fn blink(stones: &mut Vec<u64>) {
        for i in 0..stones.len() {
            match blink_stone(stones[i]) {
                BlinkResult::Changed(s) => {
                    stones[i] = s;
                }
                BlinkResult::Split(s0, s1) => {
                    stones[i] = s0;
                    stones.push(s1);
                }
            }
        }
    }

    #[test]
    fn progression() {
        let mut arrangement = vec![125, 17];
        let step_sizes = vec![3, 4, 5, 9, 13, 22];

        for size in step_sizes.into_iter() {
            blink(&mut arrangement);

            assert_eq!(arrangement.len(), size);
        }
    }

    #[test]
    fn progress_n() {
        let arrangement = vec![125, 17];
        let final_length = blink_n(&arrangement, 25);

        assert_eq!(final_length, 55312);
    }
}
