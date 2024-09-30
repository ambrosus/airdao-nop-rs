use config::ConfigError;
use serde::de::DeserializeOwned;
use std::path::PathBuf;

pub trait JsonConfig {
    type Type: DeserializeOwned;
    const DEFAULT_PATH: Option<&str> = None;

    fn load_json(root: &str, path: &str) -> Result<Self::Type, ConfigError> {
        let root = PathBuf::from(root);

        let mut builder = config::Config::builder();

        if let Some(default_path) = Self::DEFAULT_PATH {
            // Load default set of configuration
            builder = builder.add_source(config::File::from(root.join(default_path)));
        }

        builder
            .add_source(config::File::from(root.join(path)).required(Self::DEFAULT_PATH.is_none()))
            .build()
            .and_then(|config| config.try_deserialize())
    }
}
