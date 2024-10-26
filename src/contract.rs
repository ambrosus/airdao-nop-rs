use alloy::{
    json_abi::{Function, JsonAbi as Contract},
    primitives::Address,
};
use anyhow::anyhow;
use serde::{de, Deserialize};

#[derive(Debug)]
pub struct EthContract {
    pub address: Address,
    pub inner: Contract,
}

impl EthContract {
    pub fn function(&self, name: &str) -> anyhow::Result<&Function> {
        self.inner
            .function(name)
            .and_then(|functions| functions.first())
            .ok_or_else(|| anyhow!("Function {name} not found in contract abi!"))
    }
}

impl<'de> Deserialize<'de> for EthContract {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Inner<'a> {
            address: Address,
            #[serde(borrow)]
            abi: Vec<&'a str>,
        }

        let inner = Inner::deserialize(deserializer)?;
        let contract_abi = Contract::parse(inner.abi.into_iter())
            .map_err(|e| de::Error::custom(format!("Failed to deserialize contract abi: {e:?}")))?;

        Ok(Self {
            address: inner.address,
            inner: contract_abi,
        })
    }
}
