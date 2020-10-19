use crate::errors::InvalidError;

#[derive(Debug)]
pub struct Board{
    words: Vec<String>,
    danger_index: u8,
    grey_indices: u32,
    team_one_indices: u32,
    team_two_indices: u32,
    unraveled_indices: u32,
}

impl Board{
    pub fn new(vocab: &Vec<String>) -> Result<Self, InvalidError>{
        if vocab.len() != 25{
            return Err(InvalidError::new("Vocab must be 25 words"));
        }
        let board = Board{
            words: vocab.iter().map(|x| {String::from(x)}).collect(),
            danger_index: 0,
            grey_indices: 0,
            team_one_indices: 0,
            team_two_indices: 0,
            unraveled_indices: 0,
        };
        Ok(board)
    }

    pub fn words(&self) -> &Vec<String>{
        &self.words
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_new_size() {
        let words: Vec<String> = (0..25).map(|x| {format!("word-{}", x)}).collect();
        let board = Board::new(&words).unwrap();
        assert_eq!(board.words().len(), 25);
    }

    #[test]
    fn test_board_new_wrong_sizes() {
        let words: Vec<String> = (0..10).map(|x| {format!("word-{}", x)}).collect();
        assert!(Board::new(&words).is_err(), "Board must error out on invalid sizes");
    }
}