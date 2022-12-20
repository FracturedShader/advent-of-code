use md5::{Digest, Md5};
use std::io::BufRead;

fn find_base<F>(tester: F)
where
    F: Fn(&[u8]) -> bool + 'static,
{
    let base = "ckczppom";

    for i in 0.. {
        let mut hasher = Md5::new();
        let input = format!("{}{}", base, i);

        hasher.update(input);

        let result = hasher.finalize();

        if tester(&result) {
            println!("Hash success for: {}", i);
            break;
        }
    }
}

pub fn part_01(_reader: Option<impl BufRead>) {
    find_base(|result| result[0..2] == [0, 0] && ((result[2] & 0xF0) == 0));
}

pub fn part_02(_reader: Option<impl BufRead>) {
    find_base(|result| result[0..3] == [0, 0, 0]);
}
