//! Web interface module
//! 
//! Provides HTTP API and WebSocket support for the NL interface.
//! Architecture inspired by kasperience's kaspa-auth implementation.

use anyhow::Result;
use axum::{
    Router,
    routing::{get, post},
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::services::ServeDir;
use tower_http::cors::CorsLayer;

use crate::{nlp, generation, deployment, session, wallet, runtime, bridge, kdapp_integration};

pub mod handlers;
pub mod websocket;

#[derive(Clone)]
pub struct AppState {
    pub nlp_processor: Arc<nlp::Processor>,
    pub code_generator: Arc<generation::Generator>,
    pub deployment_manager: Arc<deployment::Manager>,
    pub session_manager: Arc<session::Manager>,
    pub server_wallet: Option<Arc<wallet::ServerWallet>>,
    pub player1_wallet: Option<Arc<wallet::ServerWallet>>,
    pub player2_wallet: Option<Arc<wallet::ServerWallet>>,
    pub episode_manager: Arc<runtime::EpisodeManager>,
    pub command_bridge: Option<Arc<bridge::CommandBridge>>,
    pub kdapp_manager: Option<Arc<kdapp_integration::KdappManager>>,
}

pub async fn start_server(
    port: u16,
    nlp_processor: nlp::Processor,
    code_generator: generation::Generator,
    deployment_manager: deployment::Manager,
    session_manager: session::Manager,
    server_wallet: Option<wallet::ServerWallet>,
    player1_wallet: Option<wallet::ServerWallet>,
    player2_wallet: Option<wallet::ServerWallet>,
    episode_manager: runtime::EpisodeManager,
    kdapp_manager: Option<kdapp_integration::KdappManager>,
) -> Result<()> {
    let episode_manager = Arc::new(episode_manager);
    let server_wallet = server_wallet.map(Arc::new);
    let player1_wallet = player1_wallet.map(Arc::new);
    let player2_wallet = player2_wallet.map(Arc::new);
    
    // Create command bridge if we have a server wallet
    let command_bridge = if let Some(ref wallet) = server_wallet {
        Some(Arc::new(
            bridge::CommandBridge::new(
                episode_manager.clone(),
                wallet.clone(),
            ).with_player_wallets(
                player1_wallet.clone(),
                player2_wallet.clone(),
            )
        ))
    } else {
        None
    };
    
    let state = AppState {
        nlp_processor: Arc::new(nlp_processor),
        code_generator: Arc::new(code_generator),
        deployment_manager: Arc::new(deployment_manager),
        session_manager: Arc::new(session_manager),
        server_wallet,
        player1_wallet,
        player2_wallet,
        episode_manager,
        command_bridge,
        kdapp_manager: kdapp_manager.map(Arc::new),
    };

    let app = Router::new()
        // API routes
        .route("/api/generate", post(handlers::generate_episode))
        .route("/api/status/:id", get(handlers::get_status))
        .route("/api/episodes", get(handlers::list_episodes))
        
        // Episode interaction endpoints
        .route("/api/episode/:id/command", post(handlers::execute_command))
        .route("/api/episode/:id/state", get(handlers::get_episode_state))
        
        // WebSocket for real-time updates
        .route("/ws/:episode_id", get(websocket::websocket_handler))
        
        // Dynamic app interface
        .route("/app/:episode_id", get(handlers::serve_app))
        
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