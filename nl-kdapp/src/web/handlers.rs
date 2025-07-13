//! HTTP request handlers

use axum::{
    extract::{State, Path},
    response::Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
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

    // Deploy
    let deployment = match state.deployment_manager.deploy(&generated_code.episode_code).await {
        Ok(result) => result,
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

    Ok(Json(GenerateResponse {
        status: "success".to_string(),
        episode_id: Some(deployment.episode_id),
        share_link: Some(deployment.share_link),
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

pub async fn list_episodes() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "episodes": []
    }))
}