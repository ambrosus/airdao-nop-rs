#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),
    /// Hex decode
    #[error("Decode from hex error: {0:#}")]
    FromHex(#[from] hex::FromHexError),
    /// Signature error
    #[error("Signature error: {0:#}")]
    Signature(#[from] k256::ecdsa::signature::Error),
    /// Generic
    #[error("{0:#}")]
    Anyhow(#[from] anyhow::Error),
}
