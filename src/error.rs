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
    /// SerDe JSON error
    #[error("SerDe JSON error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    /// Utf-8 parser error
    #[error("Utf-8 parse error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    /// YAML reader error
    #[error("YAML scan error: {0}")]
    YamlScanError(#[from] yaml_rust2::ScanError),
    /// Keystore error
    #[error("Keystore error: {0}")]
    KeystoreError(#[from] eth_keystore::KeystoreError),
    /// Config error
    #[error("Config error: {0}")]
    ConfigError(#[from] config::ConfigError),
    /// Generic
    #[error("{0:#}")]
    Anyhow(#[from] anyhow::Error),
}
