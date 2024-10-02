use config::ConfigError;
use ethereum_types::Address;
use ethers::core::k256::ecdsa::SigningKey;
use serde::Deserialize;
use std::net::IpAddr;

use crate::{
    config::Network,
    utils::{self, config::JsonConfig},
};

const DEFAULT_STATE_PATH: &str = "state.json";

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub network: Option<Network>,
    #[serde(deserialize_with = "utils::de_opt_secp256k1_signing_key")]
    pub private_key: Option<SigningKey>,
    pub address: Option<Address>,
    pub ip: Option<IpAddr>,
}

impl JsonConfig for State {
    type Type = Self;
    const DEFAULT_PATH: Option<&str> = None;
}

impl State {
    pub fn read() -> anyhow::Result<Self> {
        let path = std::env::var("STORE_PATH").unwrap_or_else(|_| DEFAULT_STATE_PATH.to_string());
        let res = Self::load_json("./", &path);

        if matches!(&res, Err(ConfigError::Foreign(e))
            if e.downcast_ref::<std::io::Error>().map(|e| e.kind())
                == Some(std::io::ErrorKind::NotFound))
        {
            return Ok(Self::default());
        }

        res.map_err(anyhow::Error::from)
    }
}
