//! Cryptographic utilities

use anyhow::Result;

pub fn generate_random_bytes(len: usize) -> Result<Vec<u8>> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..len).map(|_| rng.gen()).collect();
    Ok(bytes)
}