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
    /// Web3 error
    #[error("Web3 error: {0}")]
    Web3(#[from] web3::Error),
    /// Regex error
    #[error("RegExp error: {0}")]
    Regexp(#[from] regex::Error),
    /// Ethers crate parse error
    #[error("Ethers abi parse error: {0}")]
    EthersParse(#[from] ethers::abi::ParseError),
    /// Ethers crate abi error
    #[error("Ethers abi error: {0}")]
    EthersAbi(#[from] ethers::abi::Error),
    /// Ethers crate invalid output type error
    #[error("Ethers invalid output type error: {0}")]
    EthersOutputType(#[from] ethers::abi::InvalidOutputType),
    /// Generic
    #[error("{0:#}")]
    Anyhow(#[from] anyhow::Error),
}
