use alloy::primitives::Address;
use config::ConfigError;
use k256::ecdsa::SigningKey;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufWriter, Write},
    net::IpAddr,
    path::PathBuf,
};

use crate::{
    config::Network,
    error::AppError,
    utils::{
        self,
        config::{ConfigPath, JsonConfig},
    },
};

const DEFAULT_STATE_PATH: ConfigPath<'_> = ConfigPath::Relative {
    root: "./",
    path: "./state.json",
};

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub network: Option<Network>,
    #[serde(with = "utils::secp256k1_signing_key_opt_str")]
    pub private_key: Option<SigningKey>,
    #[serde(skip_deserializing)]
    pub address: Option<Address>,
    pub ip: Option<IpAddr>,
}

impl JsonConfig for State {
    type Type = Self;
    const DEFAULT_PATH: Option<&ConfigPath<'_>> = None;
}

impl State {
    pub fn path() -> PathBuf {
        match std::env::var("STORE_PATH").as_deref() {
            Ok(path) => PathBuf::from(&ConfigPath::Absolute { path }),
            Err(_) => PathBuf::from(&DEFAULT_STATE_PATH),
        }
    }

    pub fn read() -> Result<Self, AppError> {
        let res = Self::load_json(Self::path());

        if matches!(&res, Err(ConfigError::Foreign(e))
            if e.downcast_ref::<std::io::Error>().map(|e| e.kind())
                == Some(std::io::ErrorKind::NotFound))
        {
            return Ok(Self::default());
        }

        res.map(|mut state| {
            state.address = state
                .private_key
                .as_ref()
                .map(utils::secp256k1_signing_key_to_eth_address);
            state
        })
        .map_err(AppError::from)
    }

    pub fn write(&self) -> Result<(), AppError> {
        let file = File::create(Self::path())?;
        let mut writer = BufWriter::new(file);

        serde_json::to_writer_pretty(&mut writer, &self)?;

        writer.flush().map_err(AppError::from)
    }

    pub fn is_complete(&self) -> bool {
        self.network.is_some()
            && self.private_key.is_some()
            && self.address.is_some()
            && self.ip.is_some()
    }
}
