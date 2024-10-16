use std::collections::HashMap;

use anyhow::anyhow;
use ethabi::{Function, Token};
use ethereum_types::{Address, U256};
use ethers::abi::{AbiParser, Detokenize, Tokenize};
use ethers_contract_derive::EthAbiType;
use r#macro::include_json;
use serde::Deserialize;
use serde_json::json;
use web3::{types::CallRequest, Transport, Web3};

use crate::contract::EthContract;

const DEPLOYMENTS_JSON: [(u64, &str); 3] = [
    (
        22040,
        include_str!("../airdao-node-contracts/deployments/22040.json"),
    ),
    (
        16718,
        include_str!("../airdao-node-contracts/deployments/16718.json"),
    ),
    (
        30746,
        include_str!("../airdao-node-contracts/deployments/30746.json"),
    ),
];

pub struct ServerNode<T: Transport> {
    web3_client: Web3<T>,
    contracts: HashMap<String, EthContract>,
}

#[derive(Debug, EthAbiType)]
pub struct NodeStake {
    pub amount: U256,
    pub staking_contract: Address,
    pub is_always_top: bool,
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
    apollo: NodeStake,
    withdraw_lock: Option<WithdrawLock>,
    is_onboarded: bool,
}

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

impl<T: Transport> ServerNode<T> {
    pub async fn new(transport: T) -> anyhow::Result<Self> {
        let web3_client = Web3::new(transport);
        let chain_id = web3_client.eth().chain_id().await?.as_u64();
        let mut deployments = DEPLOYMENTS_JSON
            .iter()
            .filter_map(|(chain_id, json_text)| {
                let contracts =
                    serde_json::from_str::<HashMap<String, EthContract>>(json_text).ok()?;
                Some((chain_id, contracts))
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
        })
    }

    pub async fn check_status(&self, node_addr: Address) -> anyhow::Result<bool> {
        let stake = self.stakes(node_addr).await;
        println!("{stake:?}");
        todo!()
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

    async fn query_by_fn_signature<R: Detokenize, P: Tokenize>(
        &self,
        contract: Address,
        signature: &str,
        params: P,
    ) -> anyhow::Result<R> {
        let eth_fn = ethers::abi::AbiParser::default().parse_function(signature)?;
        self.query(contract, &eth_fn, params).await
    }

    async fn get_stake(&self, node_addr: Address) -> anyhow::Result<NodeStake> {
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

    async fn get_apollo_info(&self, node_addr: Address) -> anyhow::Result<ApolloInfo> {
        let stake = self.get_stake(node_addr).await?;
        let lock_id = self.get_withdraw_lock_id(node_addr).await?;
        let lock = self.get_withdraw_lock(lock_id).await?;
        let is_onboarded = self.is_onboarded(node_addr).await?;

        Ok(ApolloInfo {
            apollo: stake,
            withdraw_lock: lock.into(),
            is_onboarded,
        })
    }
}

#[cfg(test)]
mod tests {
    use ethabi::Function;
    use r#macro::include_json;
    use serde::{Deserialize, Serialize};
    use serde_json::{Deserializer, Serializer};
    use std::io::Write;

    #[test]
    fn de_function() {
        // let a = include_json!("/Volumes/untitled/airdao-nop-rs/airdao-node-contracts/deployments/22040.json");
        println!("{:?}", Function::deserialize(&mut Deserializer::from_str("function stakes(address) view returns (uint256 amount, address stakingContract, bool isAlwaysTop)")));

        let func = Function {
            name: "baz".to_owned(),
            inputs: vec![
                ethers::abi::Param {
                    name: "a".to_owned(),
                    kind: ethers::abi::ParamType::Uint(32),
                    internal_type: None,
                },
                ethers::abi::Param {
                    name: "b".to_owned(),
                    kind: ethers::abi::ParamType::Bool,
                    internal_type: None,
                },
            ],
            outputs: vec![],
            constant: None,
            state_mutability: ethers::abi::StateMutability::Payable,
        };

        let buf = Vec::new();
        let mut writer = std::io::BufWriter::new(buf);
        let mut ser = Serializer::new(writer.by_ref());
        func.serialize(&mut ser);
        println!(
            "{:?}",
            std::str::from_utf8(writer.into_inner().unwrap().as_slice())
        );

        println!("{:?}", ethers::abi::AbiParser::default().parse_function("function stakes(address) view returns (uint256 amount, address stakingContract, bool isAlwaysTop)"));
    }
}
