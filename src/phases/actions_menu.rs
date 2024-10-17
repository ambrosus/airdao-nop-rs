use futures_util::{future::BoxFuture, FutureExt};

use super::Phase;
use crate::{error, messages, utils::debug_info::DebugInfo};
use messages::MessageType;

pub struct ActionsMenuPhase {
    client: reqwest::Client,
    discord_webhook_url: String,
    pub quit: bool,
}

impl ActionsMenuPhase {
    pub fn new(discord_webhook_url: String) -> Self {
        Self {
            quit: false,
            discord_webhook_url,
            client: reqwest::Client::new(),
        }
    }
}

impl Phase for ActionsMenuPhase {
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
                MessageType::LogsActionMenuItem => {
                    let debug_info = DebugInfo::collect().await?;
                    let title = format!("{:?}-{}", debug_info.address, debug_info.timestamp);

                    let req = self
                        .client
                        .post(&self.discord_webhook_url)
                        .json(&serde_json::json!({
                            "message": format!("Logs {title}"),
                            "fileName": format!("{title}.txt"),
                            "fileContent": format!("{debug_info:?}"),
                        }))
                        .build()?;

                    self.client.execute(req).await?;

                    Ok(())
                }
                _ => Ok(()),
            }
        }
        .boxed()
    }
}
