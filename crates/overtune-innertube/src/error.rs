use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("unexpected status {0}")]
    Status(u16),
    #[error("unexpected response: {0}")]
    Parse(&'static str),
    #[error("not playable: {0}")]
    Unplayable(String),
}

pub type Result<T> = std::result::Result<T, Error>;
