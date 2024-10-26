use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::utils::config::{ConfigPath, JsonConfig};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub discord_webhook_url: String,
    pub networks: HashMap<String, Network>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Network {
    pub domain: String,
    pub rpc: reqwest::Url,
    pub chainspec: String,
    pub explorer_url: String,
    pub name: String,
}

impl JsonConfig for Config {
    type Type = Config;
    const DEFAULT_PATH: Option<&ConfigPath<'_>> = Some(&ConfigPath::Relative {
        root: "./",
        path: "./config/default.json",
    });
}
