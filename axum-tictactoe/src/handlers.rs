use crate::game::{GameState, Player};
use crate::room::{GameRoom, GameRooms};
use crate::websocket::ServerMessage;
use axum::extract::ws::{Message, WebSocket};
use futures_util::stream::SplitSink;
use futures_util::SinkExt;
use rand::Rng;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

pub async fn handle_create_room(
    sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    game_rooms: &GameRooms,
) -> Option<Player> {
    let room_id = format!("{:04}", rand::thread_rng().gen_range(0..10000));
    let mut rooms = game_rooms.write().await;
    info!("Creating new room with ID: {}", room_id);

    let (tx, _rx) = tokio::sync::broadcast::channel(100);
    rooms.insert(
        room_id.clone(),
        GameRoom {
            game_state: GameState::new(),
            players: vec![Player::X],
            tx,
        },
    );

    let response = ServerMessage::RoomCreated {
        room_id: room_id.clone(),
        player_type: Player::X,
    };
    let _ = send_message(&sender, &response).await;
    info!("Player X joined room: {}", room_id);

    if let Some(room) = rooms.get(&room_id) {
        let mut rx = room.tx.subscribe();

        let sender_clone = sender.clone();
        tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                if let Ok(state_msg) = serde_json::from_str::<ServerMessage>(&msg) {
                    if let Err(e) = send_message(&sender_clone, &state_msg).await {
                        error!("Failed to send state message: {}", e);
                        break;
                    }
                }
            }
        });
    }

    Some(Player::X)
}

pub async fn handle_join_room(
    sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    game_rooms: &GameRooms,
    room_id: String,
) -> Option<Player> {
    let mut rooms = game_rooms.write().await;
    info!("Attempting to join room: {}", room_id);

    if let Some(room) = rooms.get_mut(&room_id) {
        if room.players.len() >= 2 {
            let _ = send_error(&sender, "Room is full".to_string()).await;
            return None;
        }
        room.players.push(Player::O);
        let response = ServerMessage::RoomJoined {
            room_id: room_id.clone(),
            player_type: Player::O,
        };
        let _ = send_message(&sender, &response).await;
        info!("Player O joined room: {}", room_id);

        let mut rx = room.tx.subscribe();
        let state_msg = ServerMessage::GameState {
            room_id: room_id.clone(),
            state: room.game_state.clone(),
        };
        let _ = room.tx.send(serde_json::to_string(&state_msg).unwrap());

        let sender_clone = sender.clone();
        tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                if let Ok(state_msg) = serde_json::from_str::<ServerMessage>(&msg) {
                    if let Err(e) = send_message(&sender_clone, &state_msg).await {
                        error!("Failed to send state message: {}", e);
                        break;
                    }
                }
            }
        });

        Some(Player::O)
    } else {
        let _ = send_error(&sender, "Room not found".to_string()).await;
        None
    }
}

pub async fn handle_leave_room(
    game_rooms: &GameRooms,
    room_id: String,
    current_player: Option<Player>,
) -> Option<Player> {
    let mut rooms = game_rooms.write().await;
    if let Some(room) = rooms.get_mut(&room_id) {
        if let Some(player_type) = current_player {
            room.players.retain(|&p| p != player_type);
            info!("Player {:?} left room: {}", player_type, room_id);
            let leave_msg = ServerMessage::PlayerLeft {
                room_id: room_id.clone(),
                player_type,
            };
            let _ = room.tx.send(serde_json::to_string(&leave_msg).unwrap());

            if room.players.is_empty() {
                info!("Room {} is empty, removing", room_id);
                rooms.remove(&room_id);
            }
            return None;
        }
    }
    current_player
}

pub async fn handle_make_move(
    sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    game_rooms: &GameRooms,
    room_id: String,
    position: usize,
    player: Player,
) {
    let mut rooms = game_rooms.write().await;

    if let Some(room) = rooms.get_mut(&room_id) {
        match room.make_move(position, player) {
            Ok(_) => {
                info!(
                    "Player {:?} made move at position {} in room {}",
                    player, position, room_id
                );
                let state_msg = ServerMessage::GameState {
                    room_id: room_id.clone(),
                    state: room.game_state.clone(),
                };
                let _ = room.tx.send(serde_json::to_string(&state_msg).unwrap());
            }
            Err(err_msg) => {
                warn!(
                    "Invalid move attempt - Room: {}, Player: {:?}, Position: {}, Error: {}",
                    room_id, player, position, err_msg
                );
                let _ = send_error(&sender, err_msg.to_string()).await;
            }
        }
    }
}

pub async fn handle_disconnection(player: Option<Player>, game_rooms: &GameRooms) {
    if let Some(player) = player {
        info!("Player {:?} disconnected", player);
        let mut rooms = game_rooms.write().await;
        let mut rooms_to_remove = Vec::new();
        let mut leave_messages = Vec::new();

        for (room_id, room) in rooms.iter_mut() {
            if !room.players.contains(&player) {
                continue;
            }

            room.players.retain(|&p| p != player);
            leave_messages.push((
                room_id.clone(),
                ServerMessage::PlayerLeft {
                    room_id: room_id.clone(),
                    player_type: player,
                },
            ));

            if room.players.is_empty() {
                rooms_to_remove.push(room_id.clone());
            }
        }

        for (room_id, leave_msg) in leave_messages {
            if let Some(room) = rooms.get(&room_id) {
                let _ = room.tx.send(serde_json::to_string(&leave_msg).unwrap());
            }
        }
        for room_id in rooms_to_remove {
            info!("Removing empty room: {}", room_id);
            rooms.remove(&room_id);
        }
    }
}

async fn send_message(
    sender: &Arc<Mutex<SplitSink<WebSocket, Message>>>,
    message: &ServerMessage,
) -> Result<(), Box<dyn std::error::Error>> {
    let msg_str = serde_json::to_string(message)?;
    sender.lock().await.send(Message::Text(msg_str)).await?;
    Ok(())
}

async fn send_error(
    sender: &Arc<Mutex<SplitSink<WebSocket, Message>>>,
    msg_str: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let error_msg = ServerMessage::Error { message: msg_str };
    send_message(sender, &error_msg).await
}
