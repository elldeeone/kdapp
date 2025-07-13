//! Web interface module
//! 
//! Provides HTTP API and WebSocket support for the NL interface.
//! Architecture inspired by kasperience's kaspa-auth implementation.

use anyhow::Result;
use axum::{
    Router,
    routing::{get, post},
    extract::State,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::services::ServeDir;
use tower_http::cors::CorsLayer;

use crate::{nlp, generation, deployment, session, wallet};

pub mod handlers;
pub mod websocket;

#[derive(Clone)]
pub struct AppState {
    pub nlp_processor: Arc<nlp::Processor>,
    pub code_generator: Arc<generation::Generator>,
    pub deployment_manager: Arc<deployment::Manager>,
    pub session_manager: Arc<session::Manager>,
    pub server_wallet: Option<Arc<wallet::ServerWallet>>,
}

pub async fn start_server(
    port: u16,
    nlp_processor: nlp::Processor,
    code_generator: generation::Generator,
    deployment_manager: deployment::Manager,
    session_manager: session::Manager,
    server_wallet: Option<wallet::ServerWallet>,
) -> Result<()> {
    let state = AppState {
        nlp_processor: Arc::new(nlp_processor),
        code_generator: Arc::new(code_generator),
        deployment_manager: Arc::new(deployment_manager),
        session_manager: Arc::new(session_manager),
        server_wallet: server_wallet.map(Arc::new),
    };

    let app = Router::new()
        // API routes
        .route("/api/generate", post(handlers::generate_episode))
        .route("/api/status/:id", get(handlers::get_status))
        .route("/api/episodes", get(handlers::list_episodes))
        
        // WebSocket for real-time updates
        .route("/ws/:episode_id", get(websocket::websocket_handler))
        
        // Serve static files
        .nest_service("/", ServeDir::new("src/web/static"))
        
        // Add CORS support
        .layer(CorsLayer::permissive())
        
        // Add state
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    log::info!("Web server listening on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}