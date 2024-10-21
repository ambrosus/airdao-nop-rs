use anyhow::anyhow;
use chrono::Utc;
use ethabi::Function;
use ethereum_types::{Address, U256};
use ethers::abi::{Detokenize, Tokenize};
use ethers_contract_derive::EthAbiType;
use futures_util::{future::BoxFuture, FutureExt};
use std::{collections::HashMap, time::Duration};
use web3::{types::CallRequest, Transport, Web3};

use super::Phase;
use crate::{config::Network, contract::EthContract, error, messages};
use messages::MessageType;

const DEPLOYMENTS_JSON: [(u64, &str); 3] = [
    (
        22040,
        include_str!("../../airdao-node-contracts/deployments/22040.json"),
    ),
    (
        16718,
        include_str!("../../airdao-node-contracts/deployments/16718.json"),
    ),
    (
        30746,
        include_str!("../../airdao-node-contracts/deployments/30746.json"),
    ),
];

pub struct CheckStatusPhase<T: Transport + Send + Sync>
where
    <T as web3::Transport>::Out: Send,
{
    web3_client: Web3<T>,
    contracts: HashMap<String, EthContract>,
    node_addr: Address,
    explorer_url: String,
}

impl<T: Transport + Send + Sync> CheckStatusPhase<T>
where
    <T as web3::Transport>::Out: Send,
{
    pub async fn new(
        web3_client: Web3<T>,
        network: &Network,
        node_addr: Address,
    ) -> anyhow::Result<Self> {
        let chain_id = web3_client.eth().chain_id().await?.as_u64();
        let mut deployments = DEPLOYMENTS_JSON
            .iter()
            .filter_map(|(chain_id, json_text)| {
                let contracts = serde_json::from_str::<HashMap<String, EthContract>>(json_text);
                if contracts.is_err() {
                    println!("{:?}", contracts);
                }
                Some((chain_id, contracts.ok()?))
            })
            .collect::<HashMap<_, _>>();
        Ok(Self {
            web3_client,
            contracts: deployments.remove(&chain_id).ok_or_else(|| {
                anyhow!(
                    "Unable to find deployment information for chain id `{}`",
                    chain_id
                )
            })?,
            node_addr,
            explorer_url: network.explorer_url.clone(),
        })
    }

    async fn query<R: Detokenize, P: Tokenize>(
        &self,
        contract: Address,
        function: &Function,
        params: P,
    ) -> anyhow::Result<R> {
        let input = function.encode_input(&params.into_tokens())?;

        let output = self
            .web3_client
            .eth()
            .call(
                CallRequest {
                    from: None,
                    to: Some(contract),
                    gas: None,
                    gas_price: None,
                    value: None,
                    data: Some(input.into()),
                    ..Default::default()
                },
                None,
            )
            .await?;

        let decoded = function.decode_output(&output.0)?;

        <R>::from_tokens(decoded).map_err(anyhow::Error::from)
    }

    #[allow(unused)]
    async fn query_by_fn_signature<R: Detokenize, P: Tokenize>(
        &self,
        contract: Address,
        signature: &str,
        params: P,
    ) -> anyhow::Result<R> {
        let eth_fn = ethers::abi::AbiParser::default().parse_function(signature)?;
        self.query(contract, &eth_fn, params).await
    }

    async fn get_stake(&self, node_addr: Address) -> anyhow::Result<Stake> {
        let contract = self
            .contracts
            .get("ServerNodesManager")
            .ok_or_else(|| anyhow!("Unable to find contract `ServerNodesManager` abi"))?;

        self.query(contract.address, contract.function("stakes")?, node_addr)
            .await
    }

    async fn get_withdraw_lock_id(&self, node_addr: Address) -> anyhow::Result<U256> {
        let contract = self
            .contracts
            .get("ServerNodesManager")
            .ok_or_else(|| anyhow!("Unable to find contract `ServerNodesManager` abi"))?;

        self.query(
            contract.address,
            contract.function("lockedWithdraws")?,
            node_addr,
        )
        .await
    }

    async fn get_withdraw_lock(&self, lock_id: U256) -> anyhow::Result<Lock> {
        let contract = self
            .contracts
            .get("LockKeeper")
            .ok_or_else(|| anyhow!("Unable to find contract `LockKeeper` abi"))?;

        self.query(contract.address, contract.function("getLock")?, lock_id)
            .await
    }

    async fn is_onboarded(&self, node_addr: Address) -> anyhow::Result<bool> {
        let contract = self
            .contracts
            .get("ValidatorSet")
            .ok_or_else(|| anyhow!("Unable to find contract `ValidatorSet` abi"))?;

        self.query::<U256, _>(
            contract.address,
            contract.function("getNodeStake")?,
            node_addr,
        )
        .await
        .map(|stake_val| !stake_val.is_zero())
    }

    async fn get_onboarding_delay(&self, node_addr: Address) -> anyhow::Result<U256> {
        let contract = self
            .contracts
            .get("ServerNodesManager")
            .ok_or_else(|| anyhow!("Unable to find contract `ServerNodesManager` abi"))?;

        self.query(
            contract.address,
            contract.function("onboardingDelay")?,
            node_addr,
        )
        .await
    }

    async fn get_apollo_info(&self, node_addr: Address) -> anyhow::Result<ApolloInfo> {
        let stake = self.get_stake(node_addr).await?;
        let lock_id = self.get_withdraw_lock_id(node_addr).await?;
        let _ = self.get_withdraw_lock(lock_id).await?;
        let is_onboarded = self.is_onboarded(node_addr).await?;

        Ok(ApolloInfo {
            apollo: stake,
            is_onboarded,
        })
    }

    async fn get_onboarding_waiting_time(
        &self,
        node_addr: Address,
        stake: &Stake,
    ) -> anyhow::Result<Duration> {
        let onboarding_delay = self.get_onboarding_delay(node_addr).await?;
        let now = Utc::now();
        let seconds_to_wait = onboarding_delay
            .as_u64()
            .saturating_sub(now.timestamp() as u64)
            .saturating_sub(stake.timestamp_stake.as_u64());

        Ok(Duration::from_secs(seconds_to_wait))
    }
}

impl<T: Transport + Send + Sync> Phase for CheckStatusPhase<T>
where
    <T as web3::Transport>::Out: Send,
{
    fn run(&mut self) -> BoxFuture<'_, Result<(), error::AppError>> {
        async {
            match self.get_apollo_info(self.node_addr).await? {
                ApolloInfo {
                    is_onboarded: true, ..
                } => {
                    cliclack::note(
                        "Status check",
                        MessageType::NodeOnboarded {
                            explorer_url: &self.explorer_url,
                            node_addr: &self.node_addr,
                        },
                    )?;
                }
                info if !info.is_registered() => {
                    cliclack::note(
                        "Status check",
                        MessageType::NodeNotRegistered {
                            explorer_url: &self.explorer_url,
                        },
                    )?;
                }
                info => {
                    cliclack::note(
                        "Status check",
                        MessageType::NodeOnboarding {
                            time_to_wait: self
                                .get_onboarding_waiting_time(self.node_addr, &info.apollo)
                                .await?,
                        },
                    )?;
                }
            }

            Ok(())
        }
        .boxed()
    }
}

#[derive(Debug, EthAbiType)]
pub struct StakeInfo {
    pub amount: U256,
    pub staking_contract: Address,
    pub is_always_top: bool,
}

#[derive(Debug, EthAbiType)]
pub struct Stake {
    stake: U256,
    timestamp_stake: U256,
    owner_address: Address,
    rewards_address: Address,
}

#[derive(Debug, EthAbiType)]
pub struct Lock {
    locker: Address,
    receiver: Address,
    token: Address,
    first_unlock_time: u64,
    unlock_period: u64,
    total_claims: u64,
    times_claimed: u64,
    interval_amount: U256,
    description: String,
}

#[derive(Debug)]
pub struct ApolloInfo {
    apollo: Stake,
    // withdraw_lock: Option<WithdrawLock>,
    is_onboarded: bool,
}

impl ApolloInfo {
    fn is_registered(&self) -> bool {
        !self.apollo.stake.is_zero()
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct WithdrawLock {
    receiver: Address,
    amount: U256,
    unlock_time: u64,
}

impl From<Lock> for Option<WithdrawLock> {
    fn from(value: Lock) -> Self {
        if value.total_claims == 0 {
            None
        } else {
            Some(WithdrawLock {
                receiver: value.receiver,
                amount: value.interval_amount,
                unlock_time: value.first_unlock_time,
            })
        }
    }
}
