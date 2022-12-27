use std::io::BufRead;

/// Looks for `SEQ_LEN` unique characters in a row and returns the index after the sequence ends if
/// such a sequence can be found.
/// Requires input to consist of all lowercase ASCII characters.
/// Panics if any value is outside the range `b'a'..=b'z'`
fn marker_end<const SEQ_LEN: usize, B>(data: B) -> Option<usize>
where
    B: Iterator<Item = u8>,
{
    // With only 26 characters it's easy to use the bits of a u32 as a hash set
    // Buffer is initially full to short-circut early checks without distinct logic
    let mut ring_buffer = [u32::MAX; SEQ_LEN];

    data.enumerate()
        .find(|(i, b)| {
            // We store the values pre-indexed to make checks direct and fast
            ring_buffer[i % SEQ_LEN] = 1u32 << (b - b'a');

            *i >= SEQ_LEN
                && ring_buffer
                    .iter()
                    .try_fold(0, |a, &v| if a & v != 0 { None } else { Some(a | v) })
                    .is_some()
        })
        .map(|(i, _)| i + 1)
}

pub fn part_01(reader: Option<impl BufRead>) {
    let reader = reader.expect("data should be available for this problem");
    let start = marker_end::<4, _>(reader.bytes().flatten())
        .expect("data should contain start-of-packet marker");

    println!("Packet data starts after character: {}", start);
}

pub fn part_02(reader: Option<impl BufRead>) {
    let reader = reader.expect("data should be available for this problem");
    let start = marker_end::<14, _>(reader.bytes().flatten())
        .expect("data should contain start-of-message marker");

    println!("Message starts after character: {}", start);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn start_of_packet() {
        let cases = vec![
            ("mjqjpqmgbljsphdztnvjfqwrcgsmlb", Some(7)),
            ("bvwbjplbgvbhsrlpgdmjqwftvncz", Some(5)),
            ("nppdvjthqldpwncqszvftbrmjlhg", Some(6)),
            ("nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg", Some(10)),
            ("zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw", Some(11)),
        ];

        for (s, e) in cases {
            assert_eq!(marker_end::<4, _>(s.bytes()), e);
        }
    }

    #[test]
    fn start_of_message() {
        let cases = vec![
            ("mjqjpqmgbljsphdztnvjfqwrcgsmlb", Some(19)),
            ("bvwbjplbgvbhsrlpgdmjqwftvncz", Some(23)),
            ("nppdvjthqldpwncqszvftbrmjlhg", Some(23)),
            ("nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg", Some(29)),
            ("zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw", Some(26)),
        ];

        for (s, e) in cases {
            assert_eq!(marker_end::<14, _>(s.bytes()), e);
        }
    }
}
