use crate::game::{GameState, Player};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

pub type GameRooms = Arc<RwLock<HashMap<String, GameRoom>>>;

pub struct GameRoom {
    pub game_state: GameState,
    pub players: Vec<Player>,
    pub tx: broadcast::Sender<String>,
}

impl GameRoom {
    pub fn make_move(&mut self, position: usize, player: Player) -> Result<(), &'static str> {
        if self.players.len() < 2 {
            return Err("Waiting for opponent to join");
        }
        if !matches!(self.game_state.status, crate::game::GameStatus::InProgress) {
            return Err("Game has already ended");
        }
        if self.game_state.current_turn != player {
            return Err("Not your turn");
        }
        if position >= 9 {
            return Err("Invalid position");
        }
        if self.game_state.board[position].is_some() {
            return Err("Position already taken");
        }

        self.game_state.board[position] = Some(player);
        self.game_state.update_status();

        if matches!(self.game_state.status, crate::game::GameStatus::InProgress) {
            self.game_state.current_turn = match player {
                Player::X => Player::O,
                Player::O => Player::X,
            };
        }

        Ok(())
    }
}
