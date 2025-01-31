use axum::{
    extract::ws::WebSocketUpgrade, extract::State, response::IntoResponse, routing::get, Router,
};
use room::GameRooms;
use tower_http::services::ServeDir;
use websocket::handle_socket;

mod game;
mod handlers;
mod room;
mod websocket;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Initialize game rooms
    let game_rooms: GameRooms = Default::default();

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .nest_service("/", ServeDir::new("static"))
        .with_state(game_rooms);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(game_rooms): State<GameRooms>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, game_rooms))
}
