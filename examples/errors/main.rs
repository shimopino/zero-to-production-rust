use thiserror::Error;
use std::backtrace::Backtrace;

#[derive(Error, Debug)]
pub enum DataStoreError {
    #[error("data store disconnected")]
    Disconnect(#[from] std::io::Error),
    #[error("the data for key `{0}` is not available")]
    Redaction(String),
    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader {
        expected: String,
        found: String,
    },
    #[error("unknown data store error")]
    Unknown,
}

#[derive(Error, Debug)]
struct MyError {
    msg: String,
    backtrace: Backtrace,
}

fn main() {}