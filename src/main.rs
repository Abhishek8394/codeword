mod board;
mod errors;

use crate::board::Board;

fn main() {
    println!("Hello, world!");
    let words: Vec<String> = (0..25).map(|x| format!("word-{}", x)).collect();
    let board = Board::new(&words).unwrap();
    println!("{:?}", board);
}
