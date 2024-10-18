use chrono::{DateTime, Utc};
use ethereum_types::U256;
use futures_util::{future::BoxFuture, FutureExt};
use serde::Deserialize;
use web3::{
    types::{Block, BlockId, BlockNumber, SyncState},
    Transport, Web3,
};

use super::Phase;
use crate::{
    error::{self, AppError},
    messages,
    state::State,
    utils::{self, debug_info::DebugInfo},
};
use messages::MessageType;

pub struct ActionsMenuPhase<T: Transport + Send + Sync>
where
    <T as web3::Transport>::Out: Send,
{
    web3_client_remote: Web3<T>,
    web3_client_local: Web3<T>,
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

impl<T: Transport + Send + Sync> ActionsMenuPhase<T>
where
    <T as web3::Transport>::Out: Send,
{
    pub fn new(
        discord_webhook_url: String,
        web3_client_remote: Web3<T>,
        web3_client_local: Web3<T>,
    ) -> Self {
        Self {
            quit: false,
            discord_webhook_url,
            client: reqwest::Client::new(),
            web3_client_remote,
            web3_client_local,
        }
    }

    async fn check(&self) -> anyhow::Result<()> {
        let State {
            network: Some(network),
            ..
        } = State::read()?
        else {
            anyhow::bail!("Network configuration is missed in state")
        };

        cliclack::log::step(MessageType::Checking)?;

        match self.web3_client_local.eth().syncing().await? {
            SyncState::Syncing(state) => {
                cliclack::note(
                    "Sync check",
                    MessageType::Syncing {
                        progress: state
                            .current_block
                            .saturating_sub(state.starting_block)
                            .saturating_mul(U256::from(100))
                            .checked_div(state.highest_block.saturating_sub(state.starting_block))
                            .unwrap_or(U256::from(100))
                            .as_u64(),
                    },
                )?;
            }
            SyncState::NotSyncing => {
                cliclack::note("Sync check", MessageType::NotSyncing)?;
            }
        }

        let fork_status = match self
            .web3_client_local
            .eth()
            .block(BlockId::Number(BlockNumber::Latest))
            .await?
        {
            Some(Block {
                number: Some(block_number),
                hash,
                ..
            }) if matches!(
                self.web3_client_local.eth().block(BlockId::Number(BlockNumber::Number(block_number))).await?,
                Some(remote_block) if remote_block.hash == hash) =>
            {
                MessageType::NotForked
            }
            _ => MessageType::Forked,
        };

        cliclack::note("Fork check", fork_status)?;

        Ok(())
    }

    async fn send_logs(&self) -> anyhow::Result<()> {
        let State {
            ip: Some(ip_address),
            ..
        } = State::read()?
        else {
            anyhow::bail!("IP configuration is missed in state")
        };

        let node_version = utils::get_node_version();
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

        todo!()
    }
}

impl<T: Transport + Send + Sync> Phase for ActionsMenuPhase<T>
where
    <T as web3::Transport>::Out: Send,
{
    fn run<'a>(&'a mut self) -> BoxFuture<'a, Result<(), error::AppError>> {
        async {
            let select = cliclack::select(MessageType::SelectActionMenu)
                .items(
                    &([
                        MessageType::LogsActionMenuItem,
                        MessageType::CheckActionMenuItem,
                        MessageType::QuitActionMenuItem,
                    ]
                    .into_iter()
                    .filter_map(|item| {
                        let name = item.to_string();
                        Some((item, name, ""))
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
