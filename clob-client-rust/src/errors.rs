use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClobError {
    #[error("Signer is needed to interact with this endpoint!")]
    L1AuthUnavailable,

    #[error("API Credentials are needed to interact with this endpoint!")]
    L2AuthNotAvailable,

    #[error("Builder API Credentials needed to interact with this endpoint!")]
    BuilderAuthNotAvailable,

    #[error("Builder key auth failed!")]
    BuilderAuthFailed,

    #[error("Other error: {0}")]
    Other(String),
}
