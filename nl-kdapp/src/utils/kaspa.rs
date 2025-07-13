//! Kaspa network utilities

pub fn get_default_wrpc_url(network: &str) -> String {
    match network {
        "mainnet" => "wss://kaspa.aspectron.org/wrpc/mainnet".to_string(),
        "testnet-10" => "wss://kaspa.aspectron.org/wrpc/testnet-10".to_string(),
        _ => "wss://localhost:17110".to_string(),
    }
}