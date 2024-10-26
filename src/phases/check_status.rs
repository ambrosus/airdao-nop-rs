use alloy::{
    contract::CallBuilder,
    dyn_abi::{DynSolValue, JsonAbiExt},
    json_abi::Function,
    primitives::{Address, U256},
    providers::{
        fillers::{FillProvider, TxFiller},
        Network, Provider,
    },
    sol_types::{sol, SolType, SolValue},
    transports::Transport,
};
use anyhow::anyhow;
use chrono::Utc;
use futures_util::{future::BoxFuture, FutureExt};
use std::{collections::HashMap, time::Duration};

use super::Phase;
use crate::{config, contract::EthContract, error::AppError, messages};
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

pub struct CheckStatusPhase<
    F,
    P: Provider<T, N> + Send + Sync + Clone,
    T: Transport + Clone,
    N: Network + Clone,
> where
    F: TxFiller<N>,
{
    provider: FillProvider<F, P, T, N>,
    contracts: HashMap<String, EthContract>,
    node_addr: Address,
    explorer_url: String,
}

impl<F, P: Provider<T, N> + Send + Sync + Clone, T: Transport + Clone, N: Network + Clone>
    CheckStatusPhase<F, P, T, N>
where
    F: TxFiller<N>,
{
    pub async fn new(
        provider: FillProvider<F, P, T, N>,
        network: &config::Network,
        node_addr: Address,
    ) -> Result<Self, AppError> {
        let chain_id = provider.get_chain_id().await?;
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
            provider,
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

    async fn query<R: SolValue>(
        &self,
        contract: Address,
        function: &Function,
        params: &[DynSolValue],
    ) -> Result<R, AppError>
    where
        R: From<<<R as SolValue>::SolType as SolType>::RustType>,
    {
        let input = function.abi_encode_input(params)?;

        let output = CallBuilder::new_raw(&self.provider, input.into())
            .to(contract)
            .call()
            .await?;

        <R>::abi_decode(&output.0, true).map_err(AppError::from)
    }

    // #[allow(unused)]
    // async fn query_by_fn_signature<R: Detokenize, P: Tokenize>(
    //     &self,
    //     contract: Address,
    //     signature: &str,
    //     params: P,
    // ) -> Result<R, AppError> {
    //     let eth_fn = ethers::abi::AbiParser::default().parse_function(signature)?;
    //     self.query(contract, &eth_fn, params).await
    // }

    async fn get_stake(&self, node_addr: Address) -> Result<Stake, AppError> {
        let contract = self
            .contracts
            .get("ServerNodesManager")
            .ok_or_else(|| anyhow!("Unable to find contract `ServerNodesManager` abi"))?;

        self.query(
            contract.address,
            contract.function("stakes")?,
            &[node_addr.into()],
        )
        .await
    }

    async fn get_withdraw_lock_id(&self, node_addr: Address) -> Result<U256, AppError> {
        let contract = self
            .contracts
            .get("ServerNodesManager")
            .ok_or_else(|| anyhow!("Unable to find contract `ServerNodesManager` abi"))?;

        self.query(
            contract.address,
            contract.function("lockedWithdraws")?,
            &[node_addr.into()],
        )
        .await
    }

    async fn get_withdraw_lock(&self, lock_id: U256) -> Result<Lock, AppError> {
        let contract = self
            .contracts
            .get("LockKeeper")
            .ok_or_else(|| anyhow!("Unable to find contract `LockKeeper` abi"))?;

        self.query(
            contract.address,
            contract.function("getLock")?,
            &[lock_id.into()],
        )
        .await
    }

    async fn is_onboarded(&self, node_addr: Address) -> Result<bool, AppError> {
        let contract = self
            .contracts
            .get("ValidatorSet")
            .ok_or_else(|| anyhow!("Unable to find contract `ValidatorSet` abi"))?;

        self.query::<U256>(
            contract.address,
            contract.function("getNodeStake")?,
            &[node_addr.into()],
        )
        .await
        .map(|stake_val| !stake_val.is_zero())
    }

    async fn get_onboarding_delay(&self, node_addr: Address) -> Result<U256, AppError> {
        let contract = self
            .contracts
            .get("ServerNodesManager")
            .ok_or_else(|| anyhow!("Unable to find contract `ServerNodesManager` abi"))?;

        self.query(
            contract.address,
            contract.function("onboardingDelay")?,
            &[node_addr.into()],
        )
        .await
    }

    async fn get_apollo_info(&self, node_addr: Address) -> Result<ApolloInfo, AppError> {
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
    ) -> Result<Duration, AppError> {
        let now = Utc::now();
        let seconds_to_wait = u64::try_from(
            self.get_onboarding_delay(node_addr)
                .await?
                .saturating_sub(stake.timestamp_stake),
        )
        .map_err(anyhow::Error::from)?
        .saturating_sub(now.timestamp() as u64);

        Ok(Duration::from_secs(seconds_to_wait))
    }
}

impl<F, P: Provider<T, N> + Send + Sync + Clone, T: Transport + Clone, N: Network + Clone> Phase
    for CheckStatusPhase<F, P, T, N>
where
    F: TxFiller<N>,
{
    fn run(&mut self) -> BoxFuture<'_, Result<(), AppError>> {
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

sol! {
    struct StakeInfo {
        uint256 amount;
        address staking_contract;
        bool is_always_top;
    }

    #[derive(Debug)]
    struct Stake {
        uint256 stake;
        uint256 timestamp_stake;
        address owner_address;
        address rewards_address;
    }

    struct Lock {
        address locker;
        address receiver;
        address token;
        uint64 first_unlock_time;
        uint64 unlock_period;
        uint64 total_claims;
        uint64 times_claimed;
        uint256 interval_amount;
        string description;
    }
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
