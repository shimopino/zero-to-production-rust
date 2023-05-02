use anyhow::{bail, ensure, Context};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataStoreError {
    #[error("data store disconnected")]
    Disconnect(#[from] std::io::Error),
    #[error("the data for key `{0}` cis not available")]
    Redaction(String),
    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader { expected: String, found: String },
    #[error("unknown data store error")]
    Unknown,
}

#[derive(Debug, thiserror::Error)]
enum ApplicationError {
    #[error("Divided By 0.")]
    DivivedByZero,
    #[error("Arguments are negative")]
    NegativeNumber,
}

fn main() {
    if let Err(e) = run() {
        println!("Error: {}", e);
        for cause in e.chain().skip(1) {
            println!("Caused by {}", cause);
        }
    }

    if let Err(e) = calc(10, 0) {
        println!("Error: {}", e);
        for cause in e.chain().skip(1) {
            println!("Caused by {}", cause);
        }
    }

    let error = calc(10, -5).unwrap_err();
    assert!(error.is::<ApplicationError>());

    match error.downcast_ref::<ApplicationError>() {
        Some(ApplicationError::DivivedByZero) => {
            println!("Error is [DivivedByZero]")
        }
        Some(ApplicationError::NegativeNumber) => {
            println!("Error is [NegativeNumber]")
        }
        None => println!("not [ApplicationError]"),
    }
}

fn run() -> anyhow::Result<()> {
    let path = "./sample.txt";
    let data =
        std::fs::read_to_string(path).with_context(|| format!("Failed to read file: {}", path))?;
    println!("File contents: {}", data);
    Ok(())
}

fn calc(a: i32, b: i32) -> anyhow::Result<i32> {
    if b == 0 {
        bail!(ApplicationError::DivivedByZero)
    }
    ensure!(a < 0, ApplicationError::NegativeNumber);

    Ok(a + b)
}
