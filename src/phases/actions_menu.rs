use alloy::{
    consensus::BlockHeader,
    eips::{BlockId, BlockNumberOrTag},
    network::{primitives::HeaderResponse, BlockResponse, Network},
    primitives::U256,
    providers::{
        fillers::{FillProvider, TxFiller},
        Provider,
    },
    rpc::types::{BlockTransactionsKind, SyncStatus},
    transports::Transport,
};
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use futures_util::{future::BoxFuture, FutureExt};
use serde::Deserialize;
use std::path::PathBuf;

use super::Phase;
use crate::{
    error::{self, AppError},
    messages,
    state::State,
    utils::{self, debug_info::DebugInfo, exec},
};
use messages::MessageType;

const MAX_REMOTE_BLOCK_TIMESTAMP_AHEAD: u64 = 60;

pub struct ActionsMenuPhase<
    F,
    P: Provider<T, N> + Send + Sync + Clone,
    T: Transport + Clone,
    N: Network + Clone,
> where
    F: TxFiller<N>,
{
    provider_remote: FillProvider<F, P, T, N>,
    provider_local: FillProvider<F, P, T, N>,
    client: reqwest::Client,
    discord_webhook_url: String,
    pub quit: bool,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum DiscordResponse {
    Success(DiscordSuccessResponse),
    Error(DiscordError),
}

#[derive(Deserialize)]
struct DiscordSuccessResponse {
    timestamp: DateTime<Utc>,
}

#[derive(Deserialize)]
struct DiscordError {
    code: i64,
    message: String,
}

impl<F, P: Provider<T, N> + Send + Sync + Clone, T: Transport + Clone, N: Network + Clone>
    ActionsMenuPhase<F, P, T, N>
where
    F: TxFiller<N>,
{
    pub fn new(
        discord_webhook_url: String,
        provider_remote: FillProvider<F, P, T, N>,
        provider_local: FillProvider<F, P, T, N>,
    ) -> Self {
        Self {
            quit: false,
            discord_webhook_url,
            client: reqwest::Client::new(),
            provider_remote,
            provider_local,
        }
    }

    async fn check_sync(&self) -> Result<(), AppError> {
        match self.provider_local.syncing().await? {
            SyncStatus::Info(info) => {
                cliclack::note(
                    "Sync check",
                    MessageType::Syncing {
                        progress: info
                            .current_block
                            .saturating_sub(info.starting_block)
                            .saturating_mul(U256::from(100))
                            .checked_div(info.highest_block.saturating_sub(info.starting_block))
                            .unwrap_or(U256::from(100))
                            .try_into()
                            .map_err(anyhow::Error::from)?,
                    },
                )?;
            }
            SyncStatus::None => {
                cliclack::note("Sync check", MessageType::NotSyncing)?;
            }
        }

        Ok(())
    }

    async fn check_fork(&self) -> Result<MessageType, AppError> {
        let Some(block) = self
            .provider_local
            .get_block(
                BlockId::Number(BlockNumberOrTag::Latest),
                BlockTransactionsKind::Hashes,
            )
            .await?
        else {
            return Ok(MessageType::Forked);
        };

        let header = block.header();

        let Some(remote_block) = self
            .provider_remote
            .get_block(
                BlockId::Number(BlockNumberOrTag::Number(header.number())),
                BlockTransactionsKind::Hashes,
            )
            .await?
        else {
            return Ok(MessageType::Forked);
        };

        let Some(remote_latest_block) = self
            .provider_remote
            .get_block(
                BlockId::Number(BlockNumberOrTag::Latest),
                BlockTransactionsKind::Hashes,
            )
            .await?
        else {
            return Ok(MessageType::Forked);
        };

        if remote_block.header().hash() != header.hash()
            || !matches!(remote_latest_block.header().timestamp().checked_sub(header.timestamp()), Some(diff) if diff < MAX_REMOTE_BLOCK_TIMESTAMP_AHEAD)
        {
            return Ok(MessageType::Forked);
        }

        Ok(MessageType::NotForked)
    }

    async fn fix_fork(&self) -> Result<(), AppError> {
        cliclack::log::step(MessageType::FixForkStepFixing)?;

        exec::run_docker_compose_down()?;

        cliclack::log::step(MessageType::FixForkStepRemovingChains)?;

        tokio::fs::remove_dir_all(utils::output_dir().join("./chains"))
            .await
            .or_else(|e| {
                if e.kind() == tokio::io::ErrorKind::NotFound {
                    Ok(())
                } else {
                    Err(e)
                }
            })?;

        cliclack::log::step(MessageType::FixForkStepDownloadingBackup)?;

        exec::run_download_backup("https://backup.ambrosus.io/blockchain.tgz").await?;

        exec::run_docker_compose_up()?;

        cliclack::log::step(MessageType::FixForkStepFixed)?;

        Ok(())
    }

    async fn check_git_version(&self) -> MessageType {
        let (local, remote) = exec::get_git_commits().await;

        if local == remote {
            MessageType::GitVersionOk
        } else {
            MessageType::GitVersionOld { local, remote }
        }
    }

    async fn fix_git_version(&self) -> Result<(), AppError> {
        cliclack::log::step(MessageType::FixGitVersionStepUpdate)?;

        exec::run_update(PathBuf::from("./update.sh")).await?;

        std::process::exit(0)
    }

    async fn check(&self) -> Result<(), AppError> {
        self.check_sync().await?;

        let fork_status = self.check_fork().await?;
        cliclack::note("Fork check", &fork_status)?;
        if fork_status == MessageType::Forked
            && cliclack::confirm(MessageType::AskFixForkIssue).interact()?
        {
            self.fix_fork().await?;
        }

        let git_versiom_status = self.check_git_version().await;
        cliclack::note("Git version check", &git_versiom_status)?;
        if let MessageType::GitVersionOld { .. } = git_versiom_status {
            if cliclack::confirm(MessageType::AskFixGitVersionIssue).interact()? {
                self.fix_git_version().await?;
            }
        }

        Ok(())
    }

    async fn send_logs(&self) -> Result<(), AppError> {
        let State {
            ip: Some(ip_address),
            ..
        } = State::read()?
        else {
            return Err(anyhow!("IP configuration is missed in state").into());
        };

        let node_version = exec::get_node_version();
        let debug_info = DebugInfo::collect().await?;
        let title = format!("{:?}-{}", debug_info.address, debug_info.timestamp);

        let debug_info_payload = format!("{debug_info:?}");
        let form = reqwest::multipart::Form::new()
            .part(
                "payload_json",
                reqwest::multipart::Part::text(serde_json::to_string(
                    &serde_json::json!({
                        "content": format!("
                            node version: {node_version}
                            address: {ip_address}
                            network: {network_name}
                            ETH address: {eth_address:?}
                        ", network_name = debug_info.network_name(), eth_address = debug_info.address)
                    }),
                )?).mime_str("application/json")?,
            )
            .part(
                format!("Logs {title}"),
                reqwest::multipart::Part::stream(debug_info_payload).mime_str("text/plain")?
                    .file_name(format!("{title}.txt")),
            );

        let req = self
            .client
            .post(&self.discord_webhook_url)
            .multipart(form)
            .build()?;

        let text = self.client.execute(req).await?.text().await?;

        match serde_json::from_str::<DiscordResponse>(&text) {
            Ok(DiscordResponse::Success(DiscordSuccessResponse { timestamp })) => {
                cliclack::note("Send logs", MessageType::LogsReceivedAt { timestamp })?;
            }
            Ok(DiscordResponse::Error(DiscordError { code, message })) => {
                cliclack::note(
                    "Send logs",
                    MessageType::LogsSendError {
                        msg: format!("Code: {code} Message: {message}"),
                    },
                )?;
            }
            Err(_) => {
                cliclack::note(
                    "Send logs",
                    MessageType::LogsSendError {
                        msg: format!("Failed to parse: {text}"),
                    },
                )?;
            }
        }

        Ok(())
    }
}

impl<F, P: Provider<T, N> + Send + Sync + Clone, T: Transport + Clone, N: Network + Clone> Phase
    for ActionsMenuPhase<F, P, T, N>
where
    F: TxFiller<N>,
{
    fn run(&mut self) -> BoxFuture<'_, Result<(), error::AppError>> {
        async {
            let select = cliclack::select(MessageType::SelectActionMenu)
                .items(
                    &([
                        MessageType::LogsActionMenuItem,
                        MessageType::CheckActionMenuItem,
                        MessageType::QuitActionMenuItem,
                    ]
                    .into_iter()
                    .map(|item| {
                        let name = item.to_string();
                        (item, name, "")
                    })
                    .collect::<Vec<_>>()),
                )
                .interact()?;

            match select {
                MessageType::QuitActionMenuItem => {
                    self.quit = true;
                    Ok(())
                }
                MessageType::LogsActionMenuItem => self.send_logs().await.map_err(AppError::from),
                _ => self.check().await.map_err(AppError::from),
            }
        }
        .boxed()
    }
}
