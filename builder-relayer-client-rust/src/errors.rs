use thiserror::Error;

#[derive(Debug, Error)]
pub enum RelayClientError {
    #[error("signer is needed to interact with this endpoint!")]
    SignerUnavailable,
    #[error("safe already deployed!")]
    SafeDeployed,
    #[error("safe not deployed!")]
    SafeNotDeployed,
    #[error("http error: {0}")]
    Http(String),
    #[error("serialization error: {0}")]
    Serde(String),
    #[error("invalid network")]
    InvalidNetwork,
}

pub type Result<T> = std::result::Result<T, RelayClientError>;
