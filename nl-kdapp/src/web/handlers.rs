//! HTTP request handlers

use axum::{
    extract::{State, Path},
    response::{Json, Html},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use rand::Rng;
use super::AppState;

#[derive(Deserialize)]
pub struct GenerateRequest {
    pub prompt: String,
}

#[derive(Serialize)]
pub struct GenerateResponse {
    pub status: String,
    pub episode_id: Option<String>,
    pub share_link: Option<String>,
    pub error: Option<String>,
    pub generated_code: Option<String>,
}

pub async fn generate_episode(
    State(state): State<AppState>,
    Json(request): Json<GenerateRequest>,
) -> Result<Json<GenerateResponse>, StatusCode> {
    // Process the prompt
    let game_request = match state.nlp_processor.process(&request.prompt).await {
        Ok(req) => req,
        Err(e) => {
            return Ok(Json(GenerateResponse {
                status: "error".to_string(),
                episode_id: None,
                share_link: None,
                error: Some(e.to_string()),
                generated_code: None,
            }));
        }
    };

    // Generate code
    let generated_code = match state.code_generator.generate(&game_request).await {
        Ok(code) => code,
        Err(e) => {
            return Ok(Json(GenerateResponse {
                status: "error".to_string(),
                episode_id: None,
                share_link: None,
                error: Some(e.to_string()),
                generated_code: None,
            }));
        }
    };

    // Create Episode on server instead of deploying
    // Generate Episode ID as u32 (matching kdapp framework)
    let episode_id_num: u32 = rand::thread_rng().gen();
    let episode_id = episode_id_num.to_string();
    let creator_session = "default"; // TODO: get from session
    
    let metadata = match state.episode_manager.create_episode(
        &game_request.game_type.to_string(),
        episode_id.clone(),
        creator_session.to_string(),
    ).await {
        Ok(meta) => meta,
        Err(e) => {
            return Ok(Json(GenerateResponse {
                status: "error".to_string(),
                episode_id: None,
                share_link: None,
                error: Some(e.to_string()),
                generated_code: None,
            }));
        }
    };
    
    // Initialize the Episode in kdapp Engine if we have the manager and wallet
    if let (Some(kdapp_manager), Some(server_wallet)) = (&state.kdapp_manager, &state.server_wallet) {
        match kdapp_manager.initialize_episode(episode_id_num, server_wallet).await {
            Ok(_) => log::info!("Initialized Episode {} in kdapp Engine", episode_id_num),
            Err(e) => log::error!("Failed to initialize Episode in kdapp Engine: {}", e),
        }
    }

    // For local development, we'll return a relative path that the frontend can interpret
    Ok(Json(GenerateResponse {
        status: "success".to_string(),
        episode_id: Some(episode_id.clone()),
        share_link: Some(format!("/app/{}", episode_id)),
        error: None,
        generated_code: Some(generated_code.episode_code.clone()),
    }))
}

pub async fn get_status(
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "episode_id": id,
        "status": "running",
        "players": 0,
    }))
}

pub async fn list_episodes(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.episode_manager.list_episodes().await {
        Ok(episodes) => Ok(Json(serde_json::json!({
            "episodes": episodes
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Deserialize)]
pub struct CommandRequest {
    pub command: serde_json::Value,
    pub session_id: String,
}

pub async fn execute_command(
    State(state): State<AppState>,
    Path(episode_id): Path<String>,
    Json(request): Json<CommandRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Check if we have command bridge
    let bridge = state.command_bridge
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    // Execute command
    match bridge.process_ui_command(
        &episode_id,
        request.command,
        None, // No player number from HTTP endpoint
    ).await {
        Ok(_) => Ok(Json(serde_json::json!({
            "status": "success"
        }))),
        Err(e) => Ok(Json(serde_json::json!({
            "status": "error",
            "error": e.to_string()
        }))),
    }
}

pub async fn get_episode_state(
    State(state): State<AppState>,
    Path(episode_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.episode_manager.get_episode(&episode_id).await {
        Ok(Some(metadata)) => Ok(Json(serde_json::json!({
            "episode_id": metadata.id,
            "type": metadata.episode_type,
            "created_at": metadata.created_at,
            "expires_at": metadata.expires_at,
            "remaining_seconds": metadata.remaining_seconds(),
            "participant_count": metadata.participant_count,
            "is_ephemeral": metadata.is_ephemeral,
        }))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn serve_app(
    State(state): State<AppState>,
    Path(episode_id): Path<String>,
) -> Result<Html<String>, StatusCode> {
    // Check if Episode exists
    match state.episode_manager.get_episode(&episode_id).await {
        Ok(Some(metadata)) => {
            // Generate dynamic HTML based on Episode type
            let html = generate_app_html(&metadata);
            Ok(Html(html))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

fn generate_app_html(metadata: &crate::runtime::EpisodeMetadata) -> String {
    format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{} - kdapp.fun</title>
    <link rel="stylesheet" href="/style.css">
</head>
<body>
    <div class="app-container">
        <header class="app-header">
            <h1>{}</h1>
            <div class="expiration-info">
                <span class="expires-label">Expires in:</span>
                <span class="countdown" data-expires="{}">{} hours</span>
            </div>
            <div class="participant-info">
                <span class="participant-count">{} participants</span>
                <button class="share-btn" onclick="shareApp()">Share</button>
            </div>
        </header>
        
        <main id="app-content" class="app-content" data-episode-id="{}" data-episode-type="{}">
            <!-- Dynamic content based on Episode type -->
            <div class="loading">Loading application...</div>
        </main>
    </div>
    
    <script src="/app.js"></script>
    <script>
        window.episodeId = '{}';
        window.episodeType = '{}';
        initializeApp();
    </script>
</body>
</html>
    "#,
        metadata.episode_type,
        metadata.episode_type,
        metadata.expires_at,
        metadata.remaining_seconds() / 3600,
        metadata.participant_count,
        metadata.id,
        metadata.episode_type,
        metadata.id,
        metadata.episode_type,
    )
}