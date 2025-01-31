use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum Player {
    X,
    O,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GameStatus {
    InProgress,
    Won(Player),
    Draw,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameState {
    pub board: Vec<Option<Player>>,
    pub current_turn: Player,
    pub status: GameStatus,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            board: vec![None; 9],
            current_turn: Player::X,
            status: GameStatus::InProgress,
        }
    }

    pub fn update_status(&mut self) {
        // Check rows
        for i in (0..9).step_by(3) {
            if let (Some(player), Some(b), Some(c)) =
                (self.board[i], self.board[i + 1], self.board[i + 2])
            {
                if player == b && b == c {
                    self.status = GameStatus::Won(player);
                    return;
                }
            }
        }

        // Check columns
        for i in 0..3 {
            if let (Some(player), Some(b), Some(c)) =
                (self.board[i], self.board[i + 3], self.board[i + 6])
            {
                if player == b && b == c {
                    self.status = GameStatus::Won(player);
                    return;
                }
            }
        }

        // Check diagonals
        if let (Some(player), Some(b), Some(c)) = (self.board[0], self.board[4], self.board[8]) {
            if player == b && b == c {
                self.status = GameStatus::Won(player);
                return;
            }
        }
        if let (Some(player), Some(b), Some(c)) = (self.board[2], self.board[4], self.board[6]) {
            if player == b && b == c {
                self.status = GameStatus::Won(player);
                return;
            }
        }

        // Check for draw
        if self.board.iter().all(|cell| cell.is_some()) {
            self.status = GameStatus::Draw;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row_wins() {
        let mut game = GameState::new();
        
        // Test first row
        game.board = vec![
            Some(Player::X), Some(Player::X), Some(Player::X),
            None, None, None,
            None, None, None
        ];
        game.update_status();
        assert!(matches!(game.status, GameStatus::Won(Player::X)));

        // Test second row
        game.board = vec![
            None, None, None,
            Some(Player::O), Some(Player::O), Some(Player::O),
            None, None, None
        ];
        game.update_status();
        assert!(matches!(game.status, GameStatus::Won(Player::O)));

        // Test third row
        game.board = vec![
            None, None, None,
            None, None, None,
            Some(Player::X), Some(Player::X), Some(Player::X)
        ];
        game.update_status();
        assert!(matches!(game.status, GameStatus::Won(Player::X)));
    }

    #[test]
    fn test_column_wins() {
        let mut game = GameState::new();
        
        // Test first column
        game.board = vec![
            Some(Player::X), None, None,
            Some(Player::X), None, None,
            Some(Player::X), None, None
        ];
        game.update_status();
        assert!(matches!(game.status, GameStatus::Won(Player::X)));

        // Test second column
        game.board = vec![
            None, Some(Player::O), None,
            None, Some(Player::O), None,
            None, Some(Player::O), None
        ];
        game.update_status();
        assert!(matches!(game.status, GameStatus::Won(Player::O)));

        // Test third column
        game.board = vec![
            None, None, Some(Player::X),
            None, None, Some(Player::X),
            None, None, Some(Player::X)
        ];
        game.update_status();
        assert!(matches!(game.status, GameStatus::Won(Player::X)));
    }

    #[test]
    fn test_diagonal_wins() {
        let mut game = GameState::new();
        
        // Test main diagonal (top-left to bottom-right)
        game.board = vec![
            Some(Player::X), None, None,
            None, Some(Player::X), None,
            None, None, Some(Player::X)
        ];
        game.update_status();
        assert!(matches!(game.status, GameStatus::Won(Player::X)));

        // Test other diagonal (top-right to bottom-left)
        game.board = vec![
            None, None, Some(Player::O),
            None, Some(Player::O), None,
            Some(Player::O), None, None
        ];
        game.update_status();
        assert!(matches!(game.status, GameStatus::Won(Player::O)));
    }

    #[test]
    fn test_draw() {
        let mut game = GameState::new();
        game.board = vec![
            Some(Player::X), Some(Player::O), Some(Player::X),
            Some(Player::X), Some(Player::O), Some(Player::O),
            Some(Player::O), Some(Player::X), Some(Player::X)
        ];
        game.update_status();
        assert!(matches!(game.status, GameStatus::Draw));
    }

    #[test]
    fn test_in_progress() {
        let mut game = GameState::new();
        game.board = vec![
            Some(Player::X), Some(Player::O), None,
            Some(Player::X), None, None,
            None, None, None
        ];
        game.update_status();
        assert!(matches!(game.status, GameStatus::InProgress));
    }
}
