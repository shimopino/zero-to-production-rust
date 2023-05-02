use thiserror::Error;
use anyhow::{Context};
use std::backtrace::Backtrace;

#[derive(Error, Debug)]
pub enum DataStoreError {
    #[error("data store disconnected")]
    Disconnect(#[from] std::io::Error),
    #[error("the data for key `{0}` cis not available")]
    Redaction(String),
    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader {
        expected: String,
        found: String,
    },
    #[error("unknown data store error")]
    Unknown,
}

fn main() {
    if let Err(e) = run() {
        println!("Error: {}", e);
        for cause in e.chain().skip(1) {
            println!("Caused by {}", cause);
        }
    }
}

fn run() -> anyhow::Result<()> {
    let path = "./sample.txt";
    let data = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path))?;
    println!("File contents: {}", data);
    Ok(())
}