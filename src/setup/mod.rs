mod docker_compose;
pub mod utils;

use anyhow::anyhow;
use docker_compose::DockerComposeFile;
use ethereum_types::Address;
use futures::StreamExt;
use k256::ecdsa::SigningKey;
use serde::Deserialize;
use std::path::PathBuf;

use crate::{config::Network, error::AppError, state::State};

const DEFAULT_OUTPUT_PATH: &str = "./output/";
const DEFAULT_TEMPLATES_PATH: &str = "./setup_templates/";
const CHAIN_DESCRIPTION_FILE_NAME: &str = "./chain.json";
const DOCKER_FILE_NAME: &str = "./docker-compose.yml";

pub struct Setup {
    network: Network,
    address: Address,
    private_key: SigningKey,
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
            ..
        } = state
        else {
            return Err(anyhow!("State is incomplete").into());
        };

        Ok(Self {
            network,
            address,
            private_key,
        })
    }

    fn output_path() -> PathBuf {
        std::env::var("OUTPUT_DIRECTORY")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./").join(DEFAULT_OUTPUT_PATH.to_string()))
    }

    fn templates_path() -> PathBuf {
        std::env::var("TEMPLATE_DIRECTORY")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./").join(DEFAULT_TEMPLATES_PATH.to_string()))
    }

    pub async fn run(&self) -> Result<(), AppError> {
        let output_dir = Self::output_path();

        if !tokio::fs::try_exists(&output_dir).await? {
            tokio::fs::create_dir_all(&output_dir).await?;
        }

        let chainspec = self.download_and_save_chainspec_file(&output_dir).await?;

        let docker_template_file_path = Self::templates_path()
            .join("apollo")
            .join(&chainspec.name)
            .join(DOCKER_FILE_NAME);

        let docker_compose_file = DockerComposeFile::new(
            docker_template_file_path,
            output_dir.join(DOCKER_FILE_NAME),
            &self.network,
            &self.address,
        )
        .await?;
        docker_compose_file.save().await?;

        todo!()
    }

    async fn download_and_save_chainspec_file(
        &self,
        output_dir: &PathBuf,
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
