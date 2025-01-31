# Axum Tic-Tac-Toe

A real-time multiplayer Tic-Tac-Toe game implemented using Rust's Axum web framework and WebSockets for live game updates.

## Features

- Real-time multiplayer gameplay using WebSocket
- Server-side game state management

## Technical Stack

- **Backend**: Rust with Axum web framework
- **Frontend**: HTML, JavaScript, and CSS
- **Communication**: WebSocket for real-time updates

## Project Structure

```
├── src/
│   ├── game.rs      # Game logic and state management
│   ├── room.rs      # Room management for multiplayer
│   ├── websocket.rs # WebSocket handling
│   └── main.rs      # Server setup and routing
├── static/
│   ├── game.js     # Frontend game logic
│   ├── index.html  # Game interface
│   └── styles.css  # Game styling
```

## Getting Started

### Prerequisites

- Rust and Cargo installed on your system

### Installation

1. Clone the repository
2. Navigate to the project directory
3. Run the server:
   ```bash
   cargo run
   ```
4. Open your browser and visit `http://localhost:3000`

## How to Play

1. Open the game in two different browser windows
2. The first player will be "X" and the second player will be "O"
3. Players take turns clicking on empty squares to place their mark
4. The game automatically detects wins and draws
5. The game state is synchronized between all players in real-time
