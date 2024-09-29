use std::net::SocketAddr;

use strum_macros::Display;

#[derive(Display)]
pub enum MessageType {
    #[strum(serialize = "Which network do you want to be onboarded to?")]
    Network,

    #[strum(serialize = "No private key setup yet. What do you want to do?")]
    NoPrivateKey,

    #[strum(serialize = "Node IP defined as, {ip}")]
    NodeIpInfo { ip: SocketAddr },

    #[strum(
        serialize = "⛔ Docker is required, and was not found. Please verify your installation"
    )]
    DockerMissing,

    #[strum(serialize = "✅ Docker is installed")]
    DockerInstalled,
}
