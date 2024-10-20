use anyhow::anyhow;
use futures_util::{future::BoxFuture, FutureExt};

use super::Phase;
use crate::{error, messages, utils::exec};
use messages::MessageType;

pub struct DockerAvailablePhase {}

impl Phase for DockerAvailablePhase {
    fn run(&mut self) -> BoxFuture<'_, Result<(), error::AppError>> {
        async {
            if exec::is_docker_installed()? {
                cliclack::note("Docker check", MessageType::DockerInstalled)?;
                Ok(())
            } else {
                Err(anyhow!("{}", MessageType::DockerMissing).into())
            }
        }
        .boxed()
    }
}
