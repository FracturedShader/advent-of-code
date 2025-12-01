use std::fs::File;
use std::io::{BufReader, IsTerminal};

fn try_open_puzzle_file(puzzle_id: &str) -> std::io::Result<Option<File>> {
    if let Some(provided_path) = std::env::args().nth(3) {
        File::open(provided_path).map(Some)
    } else if let Ok(true) = std::fs::exists(format!("{puzzle_id}.txt")) {
        File::open(format!("{puzzle_id}.txt")).map(Some)
    } else if let Ok(true) = std::fs::exists(format!("data/{puzzle_id}.txt")) {
        File::open(format!("data/{puzzle_id}.txt")).map(Some)
    } else {
        Ok(None)
    }
}

pub fn puzzle_input(puzzle_id: &str) -> std::io::Result<Box<dyn std::io::BufRead>> {
    if let Some(file) = try_open_puzzle_file(puzzle_id)? {
        Ok(Box::new(BufReader::new(file)))
    } else {
        if std::io::stdin().is_terminal() {
            println!("Enter the puzzle input for {puzzle_id} followed by EOF (usually Ctrl-D):");
        }

        Ok(Box::new(std::io::stdin().lock()))
    }
}
