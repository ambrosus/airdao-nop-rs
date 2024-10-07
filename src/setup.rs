use anyhow::anyhow;
use futures::StreamExt;
use std::{fs, path::PathBuf};

use crate::{error::AppError, state::State};

pub struct Setup {
    state: State,
}

const DEFAULT_OUTPUT_PATH: &str = "./output/";
const CHAIN_DESCRIPTION_FILE_NAME: &str = "./chain.json";

impl Setup {
    pub fn new(state: State) -> Self {
        Self { state }
    }

    fn output_path() -> PathBuf {
        std::env::var("OUTPUT_DIRECTORY")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./").join(DEFAULT_OUTPUT_PATH.to_string()))
    }

    pub async fn run(&self) -> Result<(), AppError> {
        let output_path = Self::output_path();

        if !fs::exists(&output_path)? {
            fs::create_dir_all(&output_path)?;
        }

        self.download_chainspec_file(&output_path).await?;

        todo!()
    }

    async fn download_chainspec_file(&self, output_dir: &PathBuf) -> Result<(), AppError> {
        let chainspec_url = self
            .state
            .network
            .as_ref()
            .map(|network| &network.chainspec)
            .ok_or(anyhow!("Chainspec url not found"))?;
        let mut chain_spec_file =
            tokio::fs::File::create(output_dir.join(CHAIN_DESCRIPTION_FILE_NAME)).await?;
        let mut chain_spec_stream = reqwest::get(chainspec_url).await?.bytes_stream();
        while let Some(chunk) = chain_spec_stream.next().await {
            tokio::io::copy(&mut chunk?.as_ref(), &mut chain_spec_file).await?;
        }

        Ok(())
    }
}
