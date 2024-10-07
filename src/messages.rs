use ethereum_types::Address;
use std::net::IpAddr;
use strum_macros::Display;

#[derive(Display)]
pub enum MessageType<'a> {
    #[strum(serialize = "Which network do you want to be onboarded to?")]
    NetworkRequest,

    #[strum(serialize = "Network {network:?}")]
    NetworkSelected { network: &'a str },

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

    #[strum(
        serialize = "Please provide the IP address, which you will be using for your node. \nIs ${ip} correct?"
    )]
    NodeIpConfirmRequest { ip: IpAddr },

    #[strum(serialize = "Provide the IP address, which you will be using for your node")]
    NodeIpInputManually,

    #[strum(serialize = "{ip} is not a valid IP address")]
    NodeIpInvalidFormat { ip: &'a str },

    #[strum(serialize = "Node IP defined as {ip}")]
    NodeIpInfo { ip: &'a IpAddr },

    #[strum(
        serialize = "⛔ Docker is required, and was not found. Please verify your installation"
    )]
    DockerMissing,

    #[strum(serialize = "✅ Docker is installed")]
    DockerInstalled,
}
