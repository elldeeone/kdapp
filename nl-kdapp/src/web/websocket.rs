//! WebSocket handler for real-time game updates
//! Based on kasperience's kaspa-auth WebSocket patterns

use axum::{
    extract::{ws::{WebSocket, WebSocketUpgrade}, Path, State},
    response::Response,
};
use super::AppState;

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Path(episode_id): Path<String>,
    State(_state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, episode_id))
}

async fn handle_socket(mut socket: WebSocket, episode_id: String) {
    log::info!("WebSocket connected for episode: {}", episode_id);
    
    // For POC, just accept connection
    // Real implementation will:
    // - Subscribe to episode events
    // - Forward game state updates
    // - Handle player commands
    
    while let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            log::debug!("Received WebSocket message: {:?}", msg);
        } else {
            break;
        }
    }
    
    log::info!("WebSocket disconnected for episode: {}", episode_id);
}