mod docker_compose_file;
pub mod keystore;
mod parity_config_file;
pub mod utils;

use anyhow::anyhow;
use ethereum_types::Address;
use futures::StreamExt;
use k256::ecdsa::SigningKey;
use rand::rngs::OsRng;
use serde::Deserialize;
use std::{
    net::IpAddr,
    path::{Path, PathBuf},
};

use crate::{config::Network, error::AppError, messages::MessageType, state::State};
use docker_compose_file::DockerComposeFile;
use parity_config_file::ParityConfigFile;

const DEFAULT_TEMPLATES_PATH: &str = "./setup_templates/";
const CHAIN_DESCRIPTION_FILE_NAME: &str = "./chain.json";
const DOCKER_FILE_NAME: &str = "./docker-compose.yml";
const PARITY_CONFIG_FILE_NAME: &str = "./parity_config.toml";
const PASSWORD_FILE_NAME: &str = "password.pwds";
const KEY_FILE_NAME: &str = "keyfile";

pub struct Setup {
    pub network: Network,
    pub address: Address,
    private_key: SigningKey,
    ip: IpAddr,
}

#[derive(Deserialize)]
struct Chainspec {
    name: String,
}

impl Setup {
    pub fn new(state: State) -> Result<Self, AppError> {
        let State {
            network: Some(network),
            address: Some(address),
            private_key: Some(private_key),
            ip: Some(ip),
        } = state
        else {
            return Err(anyhow!("State is incomplete").into());
        };

        Ok(Self {
            network,
            address,
            private_key,
            ip,
        })
    }

    fn templates_path() -> PathBuf {
        std::env::var("TEMPLATE_DIRECTORY")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./").join(DEFAULT_TEMPLATES_PATH))
    }

    pub async fn run(&self) -> Result<(), AppError> {
        let output_dir = crate::utils::output_dir();

        if !tokio::fs::try_exists(&output_dir).await? {
            tokio::fs::create_dir_all(&output_dir).await?;
        }

        let chainspec = self.download_and_save_chainspec_file(&output_dir).await?;

        let docker_template_file_path = Self::templates_path()
            .join("apollo")
            .join(&chainspec.name)
            .join(DOCKER_FILE_NAME);

        let parity_config_template_file_path = Self::templates_path()
            .join("apollo")
            .join(&chainspec.name)
            .join(PARITY_CONFIG_FILE_NAME);

        let docker_compose_file = DockerComposeFile::new(
            docker_template_file_path,
            output_dir.join(DOCKER_FILE_NAME),
            &chainspec.name,
            &self.network,
            &self.address,
        )
        .await?;
        docker_compose_file.save().await?;

        let parity_config_file = ParityConfigFile::new(
            parity_config_template_file_path,
            output_dir.join(PARITY_CONFIG_FILE_NAME),
            &self.address,
            &self.ip,
            &docker_compose_file.validator_version,
        )
        .await?;
        parity_config_file.save().await?;

        let random_password = utils::generate_password();
        tokio::fs::write(output_dir.join(PASSWORD_FILE_NAME), &random_password).await?;

        keystore::encrypt_key(
            &output_dir,
            &mut OsRng,
            self.private_key.to_bytes(),
            &random_password,
            Some(KEY_FILE_NAME),
        )
        .await?;

        let saved_key = eth_keystore::decrypt_key(output_dir.join(KEY_FILE_NAME), random_password);
        if !matches!(saved_key, Ok(private) if sha3::digest::generic_array::GenericArray::clone_from_slice(private.as_slice()) == self.private_key.to_bytes())
        {
            return Err(anyhow::anyhow!("Stored private key mismatch!").into());
        }

        cliclack::note("Setup status", MessageType::SetupCompleted).map_err(AppError::from)
    }

    async fn download_and_save_chainspec_file(
        &self,
        output_dir: &Path,
    ) -> Result<Chainspec, AppError> {
        let chain_spec_file_path = output_dir.join(CHAIN_DESCRIPTION_FILE_NAME);
        let mut chain_spec_file = tokio::fs::File::create(&chain_spec_file_path).await?;
        let mut chain_spec_stream = reqwest::get(&self.network.chainspec).await?.bytes_stream();
        while let Some(chunk) = chain_spec_stream.next().await {
            tokio::io::copy(&mut chunk?.as_ref(), &mut chain_spec_file).await?;
        }

        serde_json::from_slice::<Chainspec>(&tokio::fs::read(&chain_spec_file_path).await?)
            .map_err(AppError::from)
    }
}
