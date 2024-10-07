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
    /// IP Address parse error
    #[error("Invalid IP address: {0:#}")]
    IpAddress(#[from] std::net::AddrParseError),
    /// Reqwest crate error
    #[error("Http request failed: {0}")]
    ReqwestError(#[from] reqwest::Error),
    /// Generic
    #[error("{0:#}")]
    Anyhow(#[from] anyhow::Error),
}
