use thiserror::Error;

#[derive(Debug, Error)]
pub enum UltralightError {
    #[error("Request error")]
    RequestError(#[from] reqwest::Error),
    #[error("Decompression error")]
    DecompressionError(#[from] sevenz_rust::Error),
    #[error("Failed to get environment variable")]
    EnvVarError(#[from] std::env::VarError),
    #[error("IO error")]
    IoError(#[from] std::io::Error),
}

pub type UltralightResult<T> = Result<T, UltralightError>;
