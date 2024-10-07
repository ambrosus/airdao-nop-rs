use config::ConfigError;
use serde::de::DeserializeOwned;
use std::path::PathBuf;

pub enum ConfigPath<'a> {
    Relative { root: &'a str, path: &'a str },
    Absolute { path: &'a str },
}

pub trait JsonConfig {
    type Type: DeserializeOwned;
    const DEFAULT_PATH: Option<&ConfigPath<'_>> = None;

    fn load_json(path: PathBuf) -> Result<Self::Type, ConfigError> {
        let mut builder = config::Config::builder();

        if let Some(default_path) = Self::DEFAULT_PATH {
            // Load default set of configuration
            builder = builder.add_source(config::File::from(PathBuf::from(default_path)));
        }

        builder
            .add_source(config::File::from(path).required(Self::DEFAULT_PATH.is_none()))
            .build()
            .and_then(|config| config.try_deserialize())
    }
}

impl From<&ConfigPath<'_>> for PathBuf {
    fn from(value: &ConfigPath<'_>) -> Self {
        match value {
            ConfigPath::Relative { root, path } => PathBuf::from(root).join(path),
            ConfigPath::Absolute { path } => PathBuf::from(path),
        }
    }
}
