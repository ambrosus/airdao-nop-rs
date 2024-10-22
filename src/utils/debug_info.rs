use chrono::Utc;
use ethereum_types::Address;
use std::{fmt::write, path::PathBuf};

use super::exec;
use crate::{config::Network, error::AppError, state::State};

pub struct DebugInfo {
    network: Option<Network>,
    pub address: Address,
    pub timestamp: i64,
    cwd: Option<PathBuf>,
    os_release: String,
    memory_info: String,
    directory_contents: String,
    output_directory_contents: String,
    disk_block_info: String,
    disk_inodes_info: String,
    process_tree: String,
    memory_usage: String,
    compose_logs: String,
    local_head: String,
    remote_head: String,
}

impl DebugInfo {
    pub async fn collect() -> Result<Self, AppError> {
        let State {
            network,
            private_key,
            ..
        } = State::read()?;

        let (local_head, remote_head) = exec::get_git_commits().await;

        Ok(Self {
            network,
            address: private_key
                .as_ref()
                .map(super::secp256k1_signing_key_to_eth_address)
                .unwrap_or_default(),
            timestamp: Utc::now().timestamp(),
            cwd: std::env::current_dir().ok(),
            os_release: exec::get_os_release(),
            memory_info: exec::get_mem_info(),
            directory_contents: exec::get_directory_contents(None),
            output_directory_contents: exec::get_directory_contents(Some(super::output_dir())),
            disk_block_info: exec::get_disk_block_info(),
            disk_inodes_info: exec::get_disk_inodes_info(),
            process_tree: exec::get_process_tree(),
            memory_usage: exec::get_memory_usage(),
            compose_logs: exec::get_docker_compose_logs(),
            local_head,
            remote_head,
        })
    }

    pub fn network_name(&self) -> &str {
        self.network
            .as_ref()
            .map(|network| network.name.as_str())
            .unwrap_or_default()
    }
}

impl std::fmt::Debug for DebugInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write(
            f,
            format_args!(
                "
                    Address: {:?}
                    Network: {}
                    Timestamp: {}

                    Network details: {}
                    Working directory: {:?}
                    OS Release: {}
                    Memory Info: {}
                    Directory Contents: {}
                    Output Directory Contents: {}
                    Disk Block Info: {}
                    Disk Inodes Info: {}
                    Process Tree: {}
                    Memory Usage: {}
                    Docker logs: {}
                    Local Git Head: {}
                    Remote Git Head: {}
                ",
                self.address,
                self.network_name(),
                self.timestamp,
                self.network
                    .as_ref()
                    .and_then(|network| serde_json::to_string_pretty(network).ok())
                    .unwrap_or_default(),
                self.cwd.clone().unwrap_or_default(),
                self.os_release,
                self.memory_info,
                self.directory_contents,
                self.output_directory_contents,
                self.disk_block_info,
                self.disk_inodes_info,
                self.process_tree,
                self.memory_usage,
                self.compose_logs,
                self.local_head,
                self.remote_head
            ),
        )
    }
}
