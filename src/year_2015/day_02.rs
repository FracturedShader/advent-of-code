use std::io::prelude::*;

#[derive(Default)]
struct WrapRequirements {
    paper_area: u32,
    ribbon_length: u32,
}

fn wrap_more_gifts(req: &mut WrapRequirements, line: &str) {
    let mut dims: Vec<u32> = line.split('x').map(|d| d.parse().unwrap()).collect();

    dims.sort_unstable();

    req.paper_area += dims
        .iter()
        .zip(dims.iter().cycle().skip(1))
        .map(|(d1, d2)| 2 * d1 * d2)
        .sum::<u32>()
        + dims.iter().take(2).product::<u32>();

    req.ribbon_length +=
        dims.iter().take(2).map(|d| 2 * d).sum::<u32>() + dims.iter().product::<u32>();
}

fn wrap_gifts(reader: impl BufRead) -> WrapRequirements {
    let mut reqs = Default::default();

    for line in reader.lines().filter_map(|l| l.ok()) {
        wrap_more_gifts(&mut reqs, &line);
    }

    reqs
}

pub fn part_01(reader: Option<impl BufRead>) {
    println!(
        "Total wrapping paper needed: {}",
        wrap_gifts(reader.unwrap()).paper_area
    );
}

pub fn part_02(reader: Option<impl BufRead>) {
    println!(
        "Total feet of ribbon: {}",
        wrap_gifts(reader.unwrap()).ribbon_length
    );
}

#[cfg(test)]
mod test {
    use super::wrap_more_gifts;

    #[test]
    fn paper_ribbons() {
        let mut reqs = Default::default();

        wrap_more_gifts(&mut reqs, "2x3x4");

        assert_eq!(reqs.paper_area, 58);
        assert_eq!(reqs.ribbon_length, 34);

        let mut reqs = Default::default();

        wrap_more_gifts(&mut reqs, "1x1x10");

        assert_eq!(reqs.paper_area, 43);
        assert_eq!(reqs.ribbon_length, 14);
    }
}
