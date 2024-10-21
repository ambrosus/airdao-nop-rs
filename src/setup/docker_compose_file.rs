use anyhow::anyhow;
use ethereum_types::Address;
use std::path::PathBuf;
use yaml_rust2::{Yaml, YamlLoader};

use crate::{config::Network, error::AppError};

use super::utils;

pub struct DockerComposeFile {
    pub validator_version: String,
    output_file_path: PathBuf,
    content: String,
}

impl DockerComposeFile {
    pub async fn new(
        input_file_path: PathBuf,
        output_file_path: PathBuf,
        network_name: &str,
        network: &Network,
        address: &Address,
    ) -> Result<Self, AppError> {
        let bytes = tokio::fs::read(&input_file_path).await?;
        let raw_text = std::str::from_utf8(&bytes)?;
        let yaml_nodes = YamlLoader::load_from_str(raw_text)?;
        let template = raw_text.to_owned();

        for node in yaml_nodes {
            if let Some(Yaml::String(image)) =
                utils::yaml_find_hash_node(&node, "services.parity.image")
            {
                let Some((_, version)) = image.split_once(":") else {
                    continue;
                };

                return Ok(DockerComposeFile {
                    validator_version: format!("Apollo {version}"),
                    output_file_path,
                    content: template
                        .replace("<ENTER_YOUR_ADDRESS_HERE>", &format!("{:?}", address))
                        .replace("<ENTER_NETWORK_NAME_HERE>", network_name)
                        .replace("<ENTER_DOMAIN_HERE>", &network.domain),
                });
            }
        }

        Err(
            anyhow!("Validator version not found in docker template file '{input_file_path:?}'")
                .into(),
        )
    }

    pub async fn save(&self) -> Result<(), AppError> {
        tokio::fs::write(&self.output_file_path, &self.content)
            .await
            .map_err(AppError::from)
    }
}
