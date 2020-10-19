use crate::errors::InvalidError;
use rand::seq::index::sample;
use rand::thread_rng;

fn bitmap_for_pos(pos_list: &[usize]) -> Result<u32, InvalidError> {
    let mut bm: u32 = 0;
    for pos in pos_list.iter() {
        if *pos > 31 {
            return Err(InvalidError::new(
                "bitmap_for_pos only takes member values 0 <= x < 32",
            ));
        }
        bm = bm | (1 << pos);
    }
    return Ok(bm);
}

#[derive(Debug)]
pub struct Board {
    words: Vec<String>,
    danger_index: u8,
    grey_indices: u32,
    team_one_indices: u32,
    team_two_indices: u32,
    unraveled_indices: u32,
}

impl Board {
    pub fn new(vocab: &Vec<String>) -> Result<Self, InvalidError> {
        if vocab.len() != 25 {
            return Err(InvalidError::new("Vocab must be 25 words"));
        }
        let mut rng = thread_rng();
        let num_grey = 7;
        let danger_and_grey = sample(&mut rng, vocab.len(), num_grey + 1).into_vec();
        let board = Board {
            words: vocab.iter().map(|x| String::from(x)).collect(),
            danger_index: danger_and_grey[0] as u8,
            grey_indices: bitmap_for_pos(&danger_and_grey[1..])?,
            team_one_indices: 0,
            team_two_indices: 0,
            unraveled_indices: 0,
        };
        Ok(board)
    }

    pub fn words(&self) -> &Vec<String> {
        &self.words
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_new_size() {
        let words: Vec<String> = (0..25).map(|x| format!("word-{}", x)).collect();
        let board = Board::new(&words).unwrap();
        assert_eq!(board.words().len(), 25);
    }

    #[test]
    fn test_board_new_wrong_sizes() {
        let words: Vec<String> = (0..10).map(|x| format!("word-{}", x)).collect();
        assert!(
            Board::new(&words).is_err(),
            "Board must error out on invalid sizes"
        );
    }

    #[test]
    fn test_bitmap_for_pos() {
        let test_cases: Vec<(Vec<usize>, u32)> = vec![
            (vec![], 0),
            (vec![0, 31], 0x80000001),
            ((0..32).collect(), 0xFFFFFFFF),
        ];
        for (i, test) in test_cases.iter().enumerate() {
            let (inp, out) = test;
            let ans = bitmap_for_pos(&inp[..]);
            assert!(ans.is_ok());
            let ans = ans.unwrap();
            assert_eq!(*out, ans, "Error in test: {}", i);
        }
    }

    #[test]
    fn test_bitmap_for_pos_error_cases() {
        let test_cases: Vec<Vec<usize>> = vec![vec![32], (0..33).collect()];
        for (i, test) in test_cases.iter().enumerate() {
            let ans = bitmap_for_pos(&test[..]);
            assert!(ans.is_err(), "Test: {} should have failed", i);
        }
    }
}
