//! WebSocket handler for real-time application updates
//! Based on kasperience's kaspa-auth WebSocket patterns

use axum::{
    extract::{ws::{WebSocket, WebSocketUpgrade, Message}, Path, State},
    response::Response,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::{Serialize, Deserialize};
use tokio::sync::mpsc;
use super::AppState;
use crate::episodes::tictactoe::TTTState;

/// WebSocket message types
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    /// Client subscribes to Episode
    Subscribe { 
        episode_id: String,
        session_id: String,
    },
    
    /// Episode state update
    StateUpdate {
        episode_id: String,
        state: serde_json::Value,
        timestamp: u64,
    },
    
    /// Player action
    Action {
        episode_id: String,
        action: serde_json::Value,
    },
    
    /// Participant joined/left
    ParticipantUpdate {
        episode_id: String,
        participant_count: usize,
        event: String, // "joined" or "left"
    },
    
    /// Error message
    Error {
        message: String,
    },
    
    /// Episode expiration warning
    ExpirationWarning {
        episode_id: String,
        remaining_seconds: i64,
    },
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Path(episode_id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, episode_id, state))
}

async fn handle_socket(socket: WebSocket, episode_id: String, state: AppState) {
    log::info!("WebSocket connected for Episode: {}", episode_id);
    
    // Check if Episode exists
    let metadata = match state.episode_manager.get_episode(&episode_id).await {
        Ok(Some(meta)) => meta,
        Ok(None) => {
            log::warn!("WebSocket connection for non-existent Episode: {}", episode_id);
            return;
        }
        Err(e) => {
            log::error!("Error checking Episode: {}", e);
            return;
        }
    };
    
    // Split WebSocket into sender and receiver
    let (mut sender, mut receiver) = socket.split();
    
    // Create channel for sending messages to client
    let (tx, mut rx) = mpsc::channel::<WsMessage>(100);
    
    // Send initial state
    let _ = tx.send(WsMessage::StateUpdate {
        episode_id: episode_id.clone(),
        state: serde_json::json!({
            "type": metadata.episode_type,
            "created_at": metadata.created_at,
            "expires_at": metadata.expires_at,
            "participant_count": metadata.participant_count,
        }),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    }).await;
    
    // Send expiration warning if less than 1 hour remaining
    if metadata.remaining_seconds() < 3600 {
        let _ = tx.send(WsMessage::ExpirationWarning {
            episode_id: episode_id.clone(),
            remaining_seconds: metadata.remaining_seconds(),
        }).await;
    }
    
    // Subscribe to kdapp events if available
    let mut kdapp_event_task = if let Some(kdapp_manager) = &state.kdapp_manager {
        // First, send the current state if available
        if let Some(current_state) = kdapp_manager.get_episode_state(&episode_id).await {
            log::info!("Found stored state for Episode {}, sending to WebSocket", episode_id);
            if let Ok(ttt_state) = borsh::from_slice::<crate::episodes::tictactoe::TTTState>(&current_state) {
                send_tictactoe_state_update(&tx, &episode_id, &ttt_state).await;
            } else {
                log::error!("Failed to deserialize stored state for Episode {}", episode_id);
            }
        } else {
            log::warn!("No stored state found for Episode {}", episode_id);
        }
        
        if let Some(event_sender) = kdapp_manager.get_event_sender().await {
            let mut event_rx = event_sender.subscribe();
            let tx_clone = tx.clone();
            let episode_id_clone = episode_id.clone();
            
            Some(tokio::spawn(async move {
                log::debug!("Starting kdapp event listener for Episode {}", episode_id_clone);
                while let Ok(event) = event_rx.recv().await {
                    log::debug!("Received kdapp event: {:?}", event);
                    match event {
                        crate::kdapp_integration::EpisodeEvent::StateUpdate { episode_id: evt_id, state } => {
                            if evt_id == episode_id_clone {
                                // Deserialize TicTacToe state
                                if let Ok(ttt_state) = borsh::from_slice::<crate::episodes::tictactoe::TTTState>(&state) {
                                    send_tictactoe_state_update(&tx_clone, &episode_id_clone, &ttt_state).await;
                                }
                            }
                        }
                        crate::kdapp_integration::EpisodeEvent::GameOver { episode_id: evt_id, winner } => {
                            if evt_id == episode_id_clone {
                                log::info!("Game over for Episode {}: {:?}", episode_id_clone, winner);
                            }
                        }
                    }
                }
            }))
        } else {
            None
        }
    } else {
        None
    };
    
    // Spawn task to send messages to client
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });
    
    // Spawn task to receive messages from client
    let tx_clone = tx.clone();
    let state_clone = state.clone();
    let episode_id_clone = episode_id.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                // Parse and handle client message
                if let Ok(client_msg) = serde_json::from_str::<WsMessage>(&text) {
                    handle_client_message(client_msg, &state_clone, &tx_clone, &episode_id_clone).await;
                }
            }
        }
    });
    
    // Wait for any task to finish
    if let Some(mut kdapp_task) = kdapp_event_task {
        tokio::select! {
            _ = (&mut send_task) => {
                recv_task.abort();
                kdapp_task.abort();
            },
            _ = (&mut recv_task) => {
                send_task.abort();
                kdapp_task.abort();
            },
            _ = (&mut kdapp_task) => {
                send_task.abort();
                recv_task.abort();
            },
        }
    } else {
        tokio::select! {
            _ = (&mut send_task) => recv_task.abort(),
            _ = (&mut recv_task) => send_task.abort(),
        }
    }
    
    log::info!("WebSocket disconnected for Episode: {}", episode_id);
}

async fn handle_client_message(
    msg: WsMessage,
    state: &AppState,
    tx: &mpsc::Sender<WsMessage>,
    episode_id: &str,
) {
    match msg {
        WsMessage::Subscribe { .. } => {
            // Already subscribed by connecting
            log::debug!("Client subscribed to Episode: {}", episode_id);
        }
        
        WsMessage::Action { action, .. } => {
            log::debug!("Received action for Episode {}: {:?}", episode_id, action);
            
            // Extract player number from action if present
            let player_number = action.get("player")
                .and_then(|v| v.as_u64())
                .map(|n| n as u8);
            
            // Forward to command bridge if available
            if let Some(ref bridge) = state.command_bridge {
                match bridge.process_ui_command(episode_id, action.clone(), player_number).await {
                    Ok(()) => {
                        log::info!("Action processed for Episode {}", episode_id);
                        // Transaction will be detected by proxy and state updated
                    }
                    Err(e) => {
                        log::error!("Failed to process action: {}", e);
                        let _ = tx.send(WsMessage::Error {
                            message: format!("Failed to process action: {}", e),
                        }).await;
                    }
                }
            } else {
                log::warn!("No command bridge available - server wallet not configured");
                let _ = tx.send(WsMessage::Error {
                    message: "Server wallet not configured. Actions cannot be processed.".to_string(),
                }).await;
            }
        }
        
        _ => {
            // Other message types are server-to-client only
            log::warn!("Unexpected client message type");
        }
    }
}

// Helper function to send TicTacToe state update
async fn send_tictactoe_state_update(
    tx: &mpsc::Sender<WsMessage>,
    episode_id: &str,
    ttt_state: &TTTState,
) {
    log::info!("Sending TicTacToe state update for Episode {}", episode_id);
    // Convert board to JSON format
    let board: Vec<Vec<Option<String>>> = ttt_state.board.iter()
        .map(|row| row.iter()
            .map(|cell| cell.map(|pk| {
                if pk == ttt_state.first_player { "X".to_string() } else { "O".to_string() }
            }))
            .collect()
        )
        .collect();
    
    let current_player = match ttt_state.status {
        crate::episodes::tictactoe::TTTGameStatus::InProgress(pk) => {
            if pk == ttt_state.first_player { Some("X".to_string()) } else { Some("O".to_string()) }
        }
        _ => None,
    };
    
    let game_status = match ttt_state.status {
        crate::episodes::tictactoe::TTTGameStatus::InProgress(_) => "in_progress",
        crate::episodes::tictactoe::TTTGameStatus::Winner(_) => "winner",
        crate::episodes::tictactoe::TTTGameStatus::Draw => "draw",
    };
    
    let winner = match ttt_state.status {
        crate::episodes::tictactoe::TTTGameStatus::Winner(pk) => {
            Some(if pk == ttt_state.first_player { "X" } else { "O" })
        }
        _ => None,
    };
    
    // Count the moves to determine turn number
    let turn_count = ttt_state.board.iter()
        .flat_map(|row| row.iter())
        .filter(|cell| cell.is_some())
        .count() as u32;
    
    let json_state = serde_json::json!({
        "board": board,
        "currentPlayer": current_player,
        "status": game_status,
        "winner": winner,
        "turn": turn_count,
    });
    
    log::info!("WebSocket state update: {:?}", json_state);
    
    let msg = WsMessage::StateUpdate {
        episode_id: episode_id.to_string(),
        state: json_state,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };
    
    if let Err(e) = tx.send(msg).await {
        log::error!("Failed to send WebSocket message: {:?}", e);
    }
}