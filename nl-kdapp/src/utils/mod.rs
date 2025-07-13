//! Utility functions and helpers

use anyhow::Result;

pub mod crypto;
pub mod kaspa;

pub use crypto::generate_random_bytes;
pub use kaspa::get_default_wrpc_url;