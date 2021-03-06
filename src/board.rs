use crate::errors::{InvalidError, InvalidMoveError};

use rand::prelude::*;
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

fn pos_from_bitmap(bitmap: &u32) -> Vec<usize> {
    let mut res: Vec<usize> = Vec::new();
    for i in 0..32 {
        if bitmap & (1 << i) != 0 {
            res.push(i);
        }
    }
    return res;
}

fn num_ones(num: &u32) -> u32 {
    let mut x: u32 = *num;
    let mut s = 0;
    for _ in 0..32 {
        s += x & 1;
        x = x >> 1;
    }
    return s;
}

fn is_bit_set(num: &u32, idx: usize) -> bool {
    let m = 1 << idx;
    num & m == m
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
        // shuffle words
        let mut rng = thread_rng();
        let mut indices: Vec<usize> = (0..vocab.len()).collect();
        indices.shuffle(&mut rng);
        // 1/3 partitions.
        let num_team_one = (vocab.len() / 3) as usize;
        let num_team_two = num_team_one;
        let num_grey: usize = vocab.len() - num_team_two - num_team_one;
        // get data.
        let grey = &indices[0..num_grey];
        let team_one = &indices[num_grey..(num_grey + num_team_one)];
        let team_two = &indices[(num_grey + num_team_one)..];
        //
        let board = Board {
            words: vocab.iter().map(|x| String::from(x)).collect(),
            danger_index: grey[0] as u8,
            grey_indices: bitmap_for_pos(&grey[1..])?,
            team_one_indices: bitmap_for_pos(&team_one[..])?,
            team_two_indices: bitmap_for_pos(&team_two[..])?,
            unraveled_indices: 0,
        };
        Ok(board)
    }

    pub fn words(&self) -> &Vec<String> {
        &self.words
    }

    pub fn get_grey_indices_list(&self) -> Vec<usize> {
        return pos_from_bitmap(&self.grey_indices);
    }

    pub fn get_team_one_indices_list(&self) -> Vec<usize> {
        return pos_from_bitmap(&self.team_one_indices);
    }

    pub fn get_team_two_indices_list(&self) -> Vec<usize> {
        return pos_from_bitmap(&self.team_two_indices);
    }

    pub fn danger_index(&self) -> u8 {
        self.danger_index
    }

    pub fn is_word_unravelled(&self, idx: usize) -> bool {
        is_bit_set(&self.unraveled_indices, idx)
    }

    pub fn unravel_word(&mut self, idx: usize) -> Result<(), InvalidMoveError> {
        if self.is_word_unravelled(idx) {
            return Err(InvalidMoveError::new(
                "Trying to unveil already known word.",
            ));
        }
        self.unraveled_indices = self.unraveled_indices | (1 << idx);
        Ok(())
    }

    pub fn has_hidden_tiles(&self) -> bool {
        !(((1 << self.danger_index) | self.unraveled_indices) == 0x01FFFFFF)
    }

    pub fn get_team_one_pending_size(&self) -> u32 {
        num_ones(&(self.team_one_indices ^ self.unraveled_indices & self.team_one_indices))
    }

    pub fn get_team_two_pending_size(&self) -> u32 {
        num_ones(&(self.team_two_indices ^ self.unraveled_indices & self.team_two_indices))
    }

    pub fn is_team_one_index(&self, idx: usize) -> bool {
        is_bit_set(&self.team_one_indices, idx)
    }

    pub fn is_team_two_index(&self, idx: usize) -> bool {
        is_bit_set(&self.team_two_indices, idx)
    }

    pub fn is_grey_index(&self, idx: usize) -> bool {
        is_bit_set(&self.grey_indices, idx)
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
        assert_eq!(num_ones(&board.grey_indices), 8);
        assert_eq!(num_ones(&board.team_one_indices), 8);
        assert_eq!(num_ones(&board.team_two_indices), 8);
    }

    #[test]
    fn test_board_vocab_matches_indices() {
        let words: Vec<String> = (0..25).map(|x| format!("word-{}", x)).collect();
        let board = Board::new(&words).unwrap();
        let grey_list = board.get_grey_indices_list();
        let team_one_list = board.get_team_one_indices_list();
        let team_two_list = board.get_team_two_indices_list();

        let mut seen = [false; 25];
        seen[board.danger_index as usize] = true;

        // make sure all inds are unique.
        for (name, list) in [
            ("grey_list", grey_list),
            ("team_one_list", team_one_list),
            ("team_two_list", team_two_list),
        ]
        .iter()
        {
            for (i, item) in list.iter().enumerate() {
                assert!(!seen[*item], "{} has dup item {} at {}", name, item, i);
                seen[*item] = true;
            }
        }

        // make sure all inds are used.
        for i in 0..seen.len() {
            assert!(seen[i], "{} index was not used anywhere", i);
        }

        // make sure vocab ordering is intact.
        for (i, word) in board.words.iter().enumerate() {
            assert_eq!(word, &words[i], "words dont match at: {}", i);
        }
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
    fn board_unravelling() {
        let words: Vec<String> = (0..25).map(|x| format!("word-{}", x)).collect();
        let mut board = Board::new(&words).unwrap();
        for i in 0..words.len() {
            let res = board.unravel_word(i);
            assert!(res.is_ok());
            assert!(board.is_word_unravelled(i));
            // unravelling again should throw.
            let res = board.unravel_word(i);
            assert!(res.is_err());
            assert!(board.is_word_unravelled(i));
        }
    }

    #[test]
    fn test_board_has_hidden_tiles() {
        let words: Vec<String> = (0..25).map(|x| format!("word-{}", x)).collect();
        let mut board = Board::new(&words).unwrap();
        assert!(board.has_hidden_tiles());
        for i in 0..words.len() {
            if i as u8 != board.danger_index {
                let res = board.unravel_word(i);
                assert!(res.is_ok());
            }
        }
        assert!(!board.has_hidden_tiles());
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

    #[test]
    fn test_num_ones() {
        // inp, out
        let test_cases: Vec<(u32, u32)> =
            vec![(0, 0), (1, 1), (2, 1), (5, 2), (0xFFFFFFFF, 32), (7, 3)];
        for (i, test) in test_cases.iter().enumerate() {
            let (inp, out) = test;
            let ans = num_ones(inp);
            assert_eq!(ans, *out, "Error in test: {}", i);
        }
    }

    #[test]
    fn test_board_get_team_pending_size() {
        let words: Vec<String> = (0..25).map(|x| format!("word-{}", x)).collect();
        let mut board = Board::new(&words).unwrap();
        let mut exp_t1_psize = 8;
        let mut exp_t2_psize = 8;
        assert_eq!(board.get_team_one_pending_size(), exp_t1_psize);
        assert_eq!(board.get_team_two_pending_size(), exp_t2_psize);

        for i in 0..words.len() {
            if i != board.danger_index as usize {
                let res = board.unravel_word(i);
                assert!(res.is_ok());
                if board.is_team_one_index(i) {
                    exp_t1_psize -= 1;
                    assert_eq!(
                        board.get_team_one_pending_size(),
                        exp_t1_psize,
                        "problem in idx: {}",
                        i
                    );
                } else if board.is_team_two_index(i) {
                    exp_t2_psize -= 1;
                    assert_eq!(
                        board.get_team_two_pending_size(),
                        exp_t2_psize,
                        "problem in idx: {}",
                        i
                    );
                } else {
                    assert!(board.is_grey_index(i));
                }
            }
        }
        assert_eq!(board.get_team_one_pending_size(), 0);
        assert_eq!(board.get_team_two_pending_size(), 0);
    }
}
