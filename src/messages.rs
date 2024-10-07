use ethereum_types::Address;
use std::net::SocketAddr;
use strum_macros::Display;

#[derive(Display)]
pub enum MessageType {
    #[strum(serialize = "Which network do you want to be onboarded to?")]
    Network,

    #[strum(serialize = "No private key setup yet. What do you want to do?")]
    NoPrivateKey,

    #[strum(serialize = "Input existing key manually")]
    PrivateKeyInputExistingSelection,

    #[strum(serialize = "Generate new key automatically")]
    PrivateKeyGenerateNewSelection,

    #[strum(serialize = "Please provide your private key (in hex form):")]
    PrivateKeyInputManually,

    #[strum(serialize = "Private key invalid length (64 hex characters max)")]
    PrivateKeyInvalidLength,

    #[strum(serialize = "Private key should be in hex form")]
    PrivateKeyInvalidFormat,

    #[strum(serialize = "✅ Private key verified. Your address is ${address:?}")]
    PrivateKeyVerified { address: Address },

    #[strum(serialize = "Node IP defined as, {ip}")]
    NodeIpInfo { ip: SocketAddr },

    #[strum(
        serialize = "⛔ Docker is required, and was not found. Please verify your installation"
    )]
    DockerMissing,

    #[strum(serialize = "✅ Docker is installed")]
    DockerInstalled,
}
