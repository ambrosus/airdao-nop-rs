use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::utils::config::JsonConfig;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub networks: HashMap<String, Network>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Network {
    pub domain: String,
    pub rpc: String,
    pub chainspec: String,
    pub explorer_url: String,
    pub name: String,
}

impl JsonConfig for Config {
    type Type = Config;
    const DEFAULT_PATH: Option<&str> = Some("./config/default.json");
}
