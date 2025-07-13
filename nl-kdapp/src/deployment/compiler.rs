//! Code compiler for generated Episodes

use anyhow::Result;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::process::Command;

pub struct Compiler {
    temp_dir: TempDir,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            temp_dir: TempDir::new().expect("Failed to create temp dir"),
        }
    }

    pub async fn compile(&self, id: &str, code: &str) -> Result<PathBuf> {
        // For POC, save code and pretend to compile
        // Real implementation will invoke rustc
        
        let file_path = self.temp_dir.path().join(format!("{}.rs", id));
        std::fs::write(&file_path, code)?;
        
        log::info!("Would compile: {:?}", file_path);
        
        Ok(file_path)
    }
}