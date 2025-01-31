use crate::game::{GameState, Player};
use crate::handlers::{
    handle_create_room, handle_disconnection, handle_join_room, handle_leave_room, handle_make_move,
};
use crate::room::GameRooms;
use axum::extract::ws::{Message, WebSocket};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "create_room")]
    CreateRoom,
    #[serde(rename = "join_room")]
    JoinRoom { room_id: String },
    #[serde(rename = "leave_room")]
    LeaveRoom { room_id: String },
    #[serde(rename = "make_move")]
    MakeMove {
        room_id: String,
        position: usize,
        player: Player,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "room_created")]
    RoomCreated {
        room_id: String,
        player_type: Player,
    },
    #[serde(rename = "room_joined")]
    RoomJoined {
        room_id: String,
        player_type: Player,
    },
    #[serde(rename = "player_left")]
    PlayerLeft {
        room_id: String,
        player_type: Player,
    },
    #[serde(rename = "game_state")]
    GameState { room_id: String, state: GameState },
    #[serde(rename = "error")]
    Error { message: String },
}

pub async fn handle_socket(socket: WebSocket, game_rooms: GameRooms) {
    let (sender, mut receiver) = socket.split();
    let sender = Arc::new(Mutex::new(sender));
    info!("New WebSocket connection established");

    let mut current_player: Option<Player> = None;

    while let Some(Ok(message)) = receiver.next().await {
        if let Message::Text(text) = message {
            let client_msg: ClientMessage = match serde_json::from_str(&text) {
                Ok(msg) => msg,
                Err(e) => {
                    error!("Failed to parse client message: {}", e);
                    continue;
                }
            };

            let sender = sender.clone();
            match client_msg {
                ClientMessage::CreateRoom => {
                    current_player = handle_create_room(sender, &game_rooms).await;
                }

                ClientMessage::JoinRoom { room_id } => {
                    current_player = handle_join_room(sender, &game_rooms, room_id).await;
                }

                ClientMessage::LeaveRoom { room_id } => {
                    current_player = handle_leave_room(&game_rooms, room_id, current_player).await;
                }

                ClientMessage::MakeMove {
                    room_id,
                    position,
                    player,
                } => {
                    handle_make_move(sender, &game_rooms, room_id, position, player).await;
                }
            }
        }
    }

    handle_disconnection(current_player, &game_rooms).await;
}
