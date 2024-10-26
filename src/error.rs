#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),
    /// Hex decode error
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
    Reqwest(#[from] reqwest::Error),
    /// SerDe JSON error
    #[error("SerDe JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    /// Utf-8 parser error
    #[error("Utf-8 parse error: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    /// YAML reader error
    #[error("YAML scan error: {0}")]
    YamlScan(#[from] yaml_rust2::ScanError),
    /// Keystore error
    #[error("Keystore error: {0}")]
    Keystore(#[from] eth_keystore::KeystoreError),
    /// Config error
    #[error("Config error: {0}")]
    Config(#[from] config::ConfigError),
    /// Regex error
    #[error("RegExp error: {0}")]
    Regexp(#[from] regex::Error),
    /// Alloy abi error
    #[error("Alloy abi error: {0}")]
    AlloyAbi(#[from] alloy::dyn_abi::Error),
    /// Alloy types error
    #[error("Alloy types error: {0}")]
    AlloyTypes(#[from] alloy::sol_types::Error),
    /// Alloy rpc transport error
    #[error("Alloy rpc transport error: {0}")]
    AlloyRpcTransport(#[from] alloy::transports::TransportError),
    /// Alloy contract error
    #[error("Alloy contract error: {0}")]
    AlloyContract(#[from] alloy::contract::Error),
    /// Url parse error
    #[error("Url parse error: {0}")]
    UrlParse(#[from] url::ParseError),
    /// Generic
    #[error("{0:#}")]
    Anyhow(#[from] anyhow::Error),
}
