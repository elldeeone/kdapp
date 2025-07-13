//! Natural Language kdapp Interface
//! 
//! This application allows users to create kdapp Episodes through natural language prompts.
//! Architecture inspired by kasperience's kaspa-auth patterns.

use anyhow::Result;
use clap::Parser;
use log::{info, warn};

mod nlp;
mod generation;
mod web;
mod deployment;
mod templates;
mod session;
mod utils;
mod wallet;
mod runtime;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to run the web server on
    #[arg(short, long, default_value = "3000")]
    port: u16,

    /// Kaspa network to deploy to
    #[arg(short, long, default_value = "testnet-10")]
    network: String,

    /// Kaspa wRPC URL
    #[arg(short, long)]
    wrpc_url: Option<String>,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// OpenRouter API key (can also use OPENROUTER_API_KEY env var)
    #[arg(long)]
    openrouter_api_key: Option<String>,

    /// LLM model to use (claude3-sonnet, claude3-opus, gpt4, llama3-70b, etc.)
    #[arg(long, default_value = "claude3-sonnet")]
    model: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if it exists
    dotenv::dotenv().ok();
    
    let args = Args::parse();

    // Initialize logging
    if args.debug {
        env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
    } else {
        env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    }

    info!("Starting Natural Language kdapp Interface");
    info!("Network: {}", args.network);
    info!("Port: {}", args.port);

    // Set OpenRouter API key if provided via CLI
    if let Some(api_key) = args.openrouter_api_key {
        std::env::set_var("OPENROUTER_API_KEY", api_key);
    }

    // Initialize NLP processor with selected model
    let nlp_processor = if std::env::var("OPENROUTER_API_KEY").is_ok() {
        let model = match args.model.as_str() {
            "claude3-opus" => nlp::Model::Claude3Opus,
            "claude3-sonnet" => nlp::Model::Claude3Sonnet,
            "gpt4" => nlp::Model::GPT4,
            "gpt4-turbo" => nlp::Model::GPT4Turbo,
            "llama3-70b" => nlp::Model::Llama3_70B,
            "mixtral" => nlp::Model::Mixtral8x7B,
            custom => nlp::Model::Custom(custom.to_string()),
        };
        
        match nlp::Processor::with_model(model) {
            Ok(processor) => {
                info!("Using OpenRouter with model: {}", args.model);
                processor
            }
            Err(e) => {
                warn!("Failed to initialize OpenRouter: {}. Falling back to simple parser.", e);
                nlp::Processor::new()
            }
        }
    } else {
        info!("No OpenRouter API key found. Using simple pattern matching.");
        info!("Set OPENROUTER_API_KEY or use --openrouter-api-key to enable AI parsing.");
        nlp::Processor::new()
    };
    let code_generator = generation::Generator::new();
    let deployment_manager = deployment::Manager::new(&args.network, args.wrpc_url.clone());
    let session_manager = session::Manager::new();
    
    // Initialize server wallet for POC
    let server_wallet = if args.network.contains("testnet") {
        match wallet::init_server_wallet().await {
            Ok(wallet) => {
                info!("Server wallet initialized for testnet POC");
                info!("Address: {}", wallet.address);
                Some(wallet)
            }
            Err(e) => {
                warn!("Failed to initialize server wallet: {}", e);
                warn!("Set SERVER_PRIVATE_KEY for funded transactions");
                None
            }
        }
    } else {
        None
    };

    // Start web server
    web::start_server(
        args.port,
        nlp_processor,
        code_generator,
        deployment_manager,
        session_manager,
        server_wallet,
    ).await?;

    Ok(())
}