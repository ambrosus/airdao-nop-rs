use config::ConfigError;
use ethereum_types::Address;
use ethers::core::k256::ecdsa::SigningKey;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufWriter, Write},
    net::IpAddr,
    path::PathBuf,
};

use crate::{
    config::Network,
    utils::{self, config::JsonConfig},
};

const DEFAULT_STATE_PATH: &str = "state.json";

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub network: Option<Network>,
    #[serde(with = "utils::secp256k1_signing_key_opt_str")]
    pub private_key: Option<SigningKey>,
    pub address: Option<Address>,
    pub ip: Option<IpAddr>,
}

impl JsonConfig for State {
    type Type = Self;
    const DEFAULT_PATH: Option<&str> = None;
}

impl State {
    fn path() -> String {
        std::env::var("STORE_PATH").unwrap_or_else(|_| DEFAULT_STATE_PATH.to_string())
    }

    pub fn read() -> anyhow::Result<Self> {
        let path = Self::path();
        let res = Self::load_json("./", &path);

        if matches!(&res, Err(ConfigError::Foreign(e))
            if e.downcast_ref::<std::io::Error>().map(|e| e.kind())
                == Some(std::io::ErrorKind::NotFound))
        {
            return Ok(Self::default());
        }

        res.map_err(anyhow::Error::from)
    }

    pub fn write(&self) -> anyhow::Result<()> {
        let path = PathBuf::from("./").join(Self::path());
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        serde_json::to_writer_pretty(&mut writer, &self)?;

        writer.flush().map_err(anyhow::Error::from)
    }
}
