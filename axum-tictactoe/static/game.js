let ws;
let playerType;
let currentRoom;

// Connect to WebSocket server
function connect() {
    ws = new WebSocket('ws://localhost:3000/ws');
    
    ws.onopen = () => {
        console.log('Connected to server');
    };

    ws.onmessage = (event) => {
        const data = JSON.parse(event.data);
        handleServerMessage(data);
    };

    ws.onclose = () => {
        console.log('Disconnected from server');
        setTimeout(connect, 1000);
    };
}

function handleServerMessage(data) {
    const statusElement = document.getElementById('status');
    const board = document.getElementById('board');

    switch (data.type) {
        case 'room_created':
            currentRoom = data.room_id;
            playerType = data.player_type;
            document.getElementById('playerInfo').style.display = 'block';
            document.getElementById('playerType').textContent = data.player_type;
            document.getElementById('currentRoomDisplay').textContent = data.room_id;
            document.getElementById('playerType').className = '';
            document.getElementById('leaveRoom').style.display = 'inline-block';
            document.getElementById('board').classList.add('disabled');
            document.getElementById('createRoom').disabled = true;
            document.getElementById('joinRoom').disabled = true;
            document.getElementById('roomId').disabled = true;
            statusElement.textContent = `Created room ${data.room_id}. Waiting for opponent...`;
            break;

        case 'room_joined':
            currentRoom = data.room_id;
            playerType = data.player_type;
            document.getElementById('playerInfo').style.display = 'block';
            document.getElementById('currentRoomDisplay').textContent = data.room_id;
            document.getElementById('playerType').textContent = data.player_type;
            document.getElementById('playerType').className = '';
            document.getElementById('leaveRoom').style.display = 'inline-block';
            document.getElementById('board').classList.add('disabled');
            document.getElementById('createRoom').disabled = true;
            document.getElementById('joinRoom').disabled = true;
            document.getElementById('roomId').disabled = true;
            statusElement.textContent = `Joined room ${data.room_id}. Game is ready to start!`;
            break;

        case 'game_state':
            const cells = board.getElementsByClassName('cell');
            data.state.board.forEach((cell, index) => {
                cells[index].textContent = cell ? cell : '';
            });

            switch (data.state.status) {
                case 'InProgress':
                    statusElement.textContent = `Current turn: Player ${data.state.current_turn}`;
                    document.getElementById('board').classList.toggle('disabled', data.state.current_turn !== playerType);
                    break;
                case 'Draw':
                    statusElement.textContent = 'Game ended in a draw!';
                    break;
                default:
                    if (data.state.status.Won) {
                        statusElement.textContent = `Player ${data.state.status.Won} wins!`;
                    }
            }
            break;

        case 'player_left':
            if (data.room_id === currentRoom) {
                if (data.player_type === playerType) {
                    statusElement.textContent = 'You have left the room.';
                    currentRoom = null;
                    playerType = null;
                    document.getElementById('playerInfo').style.display = 'none';
                    document.getElementById('currentRoomDisplay').textContent = '';
                    document.getElementById('leaveRoom').style.display = 'none';
                    document.getElementById('board').classList.add('disabled');
                    Array.from(document.getElementsByClassName('cell')).forEach(cell => cell.textContent = '');
                } else {
                    statusElement.textContent = `Your opponent (Player ${data.player_type}) has left the room. Waiting for new player...`;
                    document.getElementById('board').classList.add('disabled');
                }
            }
            break;

        case 'error':
            statusElement.textContent = `Error: ${data.message}`;
            break;
    }
}

function createRoom() {
    const message = {
        type: 'create_room'
    };
    ws.send(JSON.stringify(message));
}

function joinRoom() {
    const roomId = document.getElementById('roomId').value;
    const message = {
        type: 'join_room',
        room_id: roomId
    };
    ws.send(JSON.stringify(message));
}

function makeMove(index) {
    if (!currentRoom || !playerType) return;
    
    const message = {
        type: 'make_move',
        room_id: currentRoom,
        position: index,
        player: playerType
    };
    ws.send(JSON.stringify(message));
}

// Initialize game when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    // Event Listeners
    document.getElementById('createRoom').addEventListener('click', createRoom);
    document.getElementById('joinRoom').addEventListener('click', joinRoom);
    document.getElementById('leaveRoom').addEventListener('click', () => {
        if (currentRoom && ws.readyState === WebSocket.OPEN) {
            ws.send(JSON.stringify({
                type: 'leave_room',
                room_id: currentRoom
            }));
            currentRoom = null;
            playerType = null;
            document.getElementById('playerInfo').style.display = 'none';
            document.getElementById('leaveRoom').style.display = 'none';
            document.getElementById('status').textContent = 'Welcome! Create or join a room to start playing.';
            const cells = document.getElementsByClassName('cell');
            Array.from(cells).forEach(cell => cell.textContent = '');
            document.getElementById('createRoom').disabled = false;
            document.getElementById('joinRoom').disabled = false;
            document.getElementById('roomId').disabled = false;
        }
    });
    document.querySelectorAll('.cell').forEach(cell => {
        cell.addEventListener('click', () => {
            const index = parseInt(cell.dataset.index);
            makeMove(index);
        });
    });

    // Initial connection
    connect();
});