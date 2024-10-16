use ethabi::{Contract, Function};
use ethereum_types::Address;
use ethers::abi::HumanReadableParser;
use serde::Deserialize;

#[derive(Debug)]
pub struct EthContract {
    pub address: Address,
    inner: ethers::abi::Contract,
}

impl EthContract {
    pub fn function(&self, name: &str) -> anyhow::Result<&Function> {
        self.inner.function(name).map_err(anyhow::Error::from)
    }
}

impl<'de> Deserialize<'de> for EthContract {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Inner {
            address: Address,
            abi: Vec<String>,
        }

        let inner = Inner::deserialize(deserializer)?;
        let mut contract_abi = Contract::default();

        // Workaround due to issues in ethers parser. Ignore errors
        inner.abi.iter().for_each(|input| {
            if input.starts_with("constructor ") {
                contract_abi.constructor = HumanReadableParser::parse_constructor(input).ok();
            } else if input.starts_with("function ") {
                if let Ok(func) = HumanReadableParser::parse_function(input) {
                    contract_abi
                        .functions
                        .entry(func.name.clone())
                        .or_default()
                        .push(func);
                }
            } else if input.starts_with("event ") {
                if let Ok(event) = HumanReadableParser::parse_event(input) {
                    contract_abi
                        .events
                        .entry(event.name.clone())
                        .or_default()
                        .push(event);
                }
            } else if input.starts_with("error ") {
                if let Ok(error) = HumanReadableParser::parse_error(input) {
                    contract_abi
                        .errors
                        .entry(error.name.clone())
                        .or_default()
                        .push(error);
                }
            } else if input.starts_with("fallback(") {
                contract_abi.fallback = true;
            } else if input.starts_with("receive(") {
                contract_abi.receive = true;
            }
        });

        Ok(Self {
            address: inner.address,
            inner: contract_abi,
        })
    }
}
