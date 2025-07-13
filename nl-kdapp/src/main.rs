//! Natural Language kdapp Interface
//! 
//! This application allows users to create kdapp Episodes through natural language prompts.
//! Architecture inspired by kasperience's kaspa-auth patterns.

use anyhow::Result;
use clap::Parser;
use log::info;

mod nlp;
mod generation;
mod web;
mod deployment;
mod templates;
mod session;
mod utils;

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
}

#[tokio::main]
async fn main() -> Result<()> {
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

    // Initialize subsystems
    let nlp_processor = nlp::Processor::new();
    let code_generator = generation::Generator::new();
    let deployment_manager = deployment::Manager::new(&args.network, args.wrpc_url);
    let session_manager = session::Manager::new();

    // Start web server
    web::start_server(
        args.port,
        nlp_processor,
        code_generator,
        deployment_manager,
        session_manager,
    ).await?;

    Ok(())
}