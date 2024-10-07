use anyhow::anyhow;
use futures_util::{future::BoxFuture, FutureExt};
use std::collections::HashMap;

use super::Phase;
use crate::{config::Network, error, messages};
use messages::MessageType;

pub struct SelectNetworkPhase<'a> {
    pub network: Option<&'a Network>,
    available_networks: &'a HashMap<String, Network>,
}

impl<'a> SelectNetworkPhase<'a> {
    pub fn new(
        network: Option<&'a Network>,
        available_networks: &'a HashMap<String, Network>,
    ) -> Self {
        Self {
            network,
            available_networks,
        }
    }
}

impl Phase for SelectNetworkPhase<'_> {
    fn run<'a>(&'a mut self) -> BoxFuture<'a, Result<(), error::AppError>> {
        async {
            let Some(initial_network) = self.available_networks.keys().next() else {
                return Err(anyhow!("No networks are defined").into());
            };

            if self
                .network
                .and_then(|network| {
                    if self.available_networks.get(&network.name) == self.network {
                        Some(network)
                    } else {
                        None
                    }
                })
                .is_none()
            {
                let selected = if self.available_networks.len() == 1 {
                    initial_network
                } else {
                    cliclack::select(MessageType::NetworkRequest)
                        .items(
                            &self
                                .available_networks
                                .iter()
                                .map(|(name, _)| (name, name, ""))
                                .collect::<Vec<_>>(),
                        )
                        .initial_value(initial_network)
                        .interact()?
                };

                self.network = self.available_networks.get(selected);
            }

            if let Some(network) = self.network {
                cliclack::note(
                    "Network check",
                    MessageType::NetworkSelected {
                        network: &network.name,
                    },
                )?;
                Ok(())
            } else {
                Err(anyhow!("No network selected").into())
            }
        }
        .boxed()
    }
}
