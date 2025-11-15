use std::io::{self, BufRead};

type DiskSpan = (std::ops::Range<usize>, Option<usize>);

/// Maximize the free space at the end of the disk by working with individual blocks rather than
/// being concerned about whole files.
fn defrag_naive(disk_map: &mut [Option<usize>]) {
    let mut li = 0;
    let mut ri = disk_map.len() - 1;

    loop {
        while disk_map[li].is_some() {
            li += 1;

            if li >= ri {
                return;
            }
        }

        while disk_map[ri].is_none() {
            ri -= 1;

            if ri == 0 || li >= ri {
                return;
            }
        }

        disk_map.swap(li, ri);
    }
}

/// Defragment the disk by moving files to the first span of empty space that can contain them.
/// Note: this routine does not add empty spans to account for moved files. It merely updates the
/// ranges of provided spans.
fn defrag_move_once(disk_map: &mut [DiskSpan]) {
    for src_idx in (0..disk_map.len()).rev() {
        if disk_map[src_idx].1.is_none() {
            continue;
        }

        let file_size = disk_map[src_idx].0.len();

        if let Some(empty_idx) = &disk_map[0..src_idx]
            .iter()
            .position(|(range, v)| v.is_none() && range.len() >= file_size)
        {
            let empty_start = disk_map[*empty_idx].0.start;

            disk_map[src_idx].0 = empty_start..(empty_start + file_size);

            disk_map[*empty_idx].0.start += file_size;
        }
    }
}

/// Calculate the checksum of the disk according to the puzzle rules.
fn disk_checksum(disk_map: &[Option<usize>]) -> usize {
    disk_map
        .iter()
        .enumerate()
        .filter_map(|(i, opt)| opt.map(|id| i * id))
        .sum()
}

/// Expands a map of spans into a full disk map of the specified length.
fn expand_map(
    disk_map: &[(std::ops::Range<usize>, Option<usize>)],
    length: usize,
) -> Vec<Option<usize>> {
    let mut expanded = vec![None; length];

    for (range, val) in disk_map.iter().filter(|(_, v)| v.is_some()).cloned() {
        for dest in &mut expanded[range] {
            *dest = val;
        }
    }

    expanded
}

/// Converts the bytes of a `line` into a more structured stream of `(count, value)` pairs.
fn stream_parsed_input(line: &str) -> impl Iterator<Item = (usize, Option<usize>)> + use<'_> {
    line.bytes().enumerate().map(|(i, c)| {
        let count = c - b'0';

        let val = if i % 2 == 0 { Some(i / 2) } else { None };

        (count as usize, val)
    })
}

/// Expands the puzzle input into a block-wise disk map.
fn expand_input_line(line: &str) -> Vec<Option<usize>> {
    stream_parsed_input(line)
        .flat_map(|(count, val)| [val].into_iter().cycle().take(count))
        .collect()
}

/// Expands the puzzle input into a file-wise disk map.
fn expand_tracked_input_line(line: &str) -> (Vec<DiskSpan>, usize) {
    let mut idx = 0;
    let collected = stream_parsed_input(line)
        .map(|(count, val)| {
            let res = (idx..(idx + count), val);
            idx += count;

            res
        })
        .collect();

    (collected, idx)
}

pub fn part_01(reader: io::Result<impl BufRead>) {
    let input_line = reader
        .expect("2024-09.txt should be in the data folder")
        .lines()
        .map_while(Result::ok)
        .next()
        .expect("input should have one valid line");

    let mut disk_map = expand_input_line(&input_line);

    defrag_naive(&mut disk_map);

    let checksum = disk_checksum(&disk_map);

    println!("Defragged checksum: {checksum}");
}

pub fn part_02(reader: io::Result<impl BufRead>) {
    let input_line = reader
        .expect("2024-09.txt should be in the data folder")
        .lines()
        .map_while(Result::ok)
        .next()
        .expect("input should have one valid line");

    let (mut disk_map, length) = expand_tracked_input_line(&input_line);

    defrag_move_once(&mut disk_map);

    let expanded = expand_map(&disk_map, length);

    let checksum = disk_checksum(&expanded);

    println!("Defragged checksum: {checksum}");
}

#[cfg(test)]
mod test {
    use super::*;

    fn map_expanded(line: &str) -> Vec<Option<usize>> {
        line.bytes()
            .map(|b| match b {
                n @ b'0'..=b'9' => Some((n - b'0').into()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn parse() {
        let expanded = expand_input_line("12345");
        let expected = map_expanded("0..111....22222");

        assert_eq!(expanded, expected);
    }

    #[test]
    fn defrag_naive_small() {
        let mut disk_map = expand_input_line("12345");
        let expected = map_expanded("022111222......");

        defrag_naive(&mut disk_map);

        assert_eq!(disk_map, expected);
    }

    #[test]
    fn defrag_naive_medium() {
        let mut disk_map = expand_input_line(&"2333133121414131402");
        let expected = map_expanded("0099811188827773336446555566..............");

        defrag_naive(&mut disk_map);

        assert_eq!(disk_map, expected);

        assert_eq!(disk_checksum(&disk_map), 1928);
    }

    #[test]
    fn defrag_move_once() {
        let (mut disk_map, length) = expand_tracked_input_line(&"2333133121414131402");
        let expected = map_expanded("00992111777.44.333....5555.6666.....8888..");

        super::defrag_move_once(&mut disk_map);

        let expanded = expand_map(&disk_map, length);

        assert_eq!(expanded, expected);

        assert_eq!(disk_checksum(&expanded), 2858);
    }
}
