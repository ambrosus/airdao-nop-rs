use chrono::{DateTime, Utc};
use ethereum_types::Address;
use std::{net::IpAddr, time::Duration};
use strum_macros::Display;

#[derive(Display, Clone, PartialEq, Eq)]
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

    #[strum(serialize = "✅ Private key verified. Your address is {address:?}")]
    PrivateKeyVerified { address: Address },

    #[strum(
        serialize = "Please provide the IP address, which you will be using for your node. \nIs {ip} correct?"
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

    #[strum(serialize = "Starting docker containers... 🐳")]
    DockerStarting,

    #[strum(serialize = "🎉 Your node is launched! 🎉")]
    DockerStarted,

    #[strum(serialize = "🎉 Your node configuration is ready 🎉")]
    SetupCompleted,

    #[strum(
        serialize = "Your node is not registered in the network. Register here: {explorer_url}/explorer/node-setup/"
    )]
    NodeNotRegistered { explorer_url: &'a str },

    #[strum(
        serialize = "Node registered and onboarded to the network🎉. You can check it here: {explorer_url}/explorer/apollo/{node_addr:?}"
    )]
    NodeOnboarded {
        explorer_url: &'a str,
        node_addr: &'a Address,
    },

    #[strum(
        serialize = "Please wait until your node is onboarded to the network, Left: {time_to_wait:?}"
    )]
    NodeOnboarding { time_to_wait: Duration },

    #[strum(serialize = "You can now perform one of the following actions")]
    SelectActionMenu,

    #[strum(serialize = "📁 Send debug information to AirDao support team")]
    LogsActionMenuItem,

    #[strum(serialize = "🔍 Try to find and fix issues with your node setup")]
    CheckActionMenuItem,

    #[strum(serialize = "👋 Quit NOP")]
    QuitActionMenuItem,

    #[strum(serialize = "Checking...")]
    Checking,

    #[strum(serialize = "Syncing {progress}%... please wait")]
    Syncing { progress: u64 },

    #[strum(serialize = "Sync: OK")]
    NotSyncing,

    #[strum(serialize = "Fork: OK")]
    NotForked,

    #[strum(serialize = "Fork: Parity has forked...")]
    Forked,

    #[strum(serialize = "Logs received at {timestamp:?}")]
    LogsReceivedAt { timestamp: DateTime<Utc> },

    #[strum(serialize = "Failed send logs. {msg}")]
    LogsSendError { msg: String },
}
