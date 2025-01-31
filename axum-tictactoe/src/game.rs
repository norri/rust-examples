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
