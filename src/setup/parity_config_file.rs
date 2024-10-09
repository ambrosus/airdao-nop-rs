use ethereum_types::Address;
use std::{net::IpAddr, path::PathBuf};

use crate::error::AppError;

pub struct ParityConfigFile {
    output_file_path: PathBuf,
    content: String,
}

impl ParityConfigFile {
    pub async fn new(
        input_file_path: PathBuf,
        output_file_path: PathBuf,
        address: &Address,
        ip: &IpAddr,
        validator_version: &String,
    ) -> Result<Self, AppError> {
        let bytes = tokio::fs::read(&input_file_path).await?;
        let raw_text = std::str::from_utf8(&bytes)?;
        let template = raw_text.to_owned();

        Ok(Self {
            output_file_path,
            content: template
                .replace("<TYPE_YOUR_ADDRESS_HERE>", &format!("{:?}", address))
                .replace("<TYPE_YOUR_IP_HERE>", &ip.to_string())
                .replace("<TYPE_EXTRA_DATA_HERE>", validator_version),
        })
    }

    pub async fn save(&self) -> Result<(), AppError> {
        tokio::fs::write(&self.output_file_path, &self.content)
            .await
            .map_err(AppError::from)
    }
}
