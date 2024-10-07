pub mod config;
pub mod exec;
pub mod logger;

use backtrace::Backtrace;
use ethereum_types::H160;
use log::error;
use serde::{de, Deserialize};
use sha3::{Digest, Keccak256};
use std::{panic, thread};

pub fn set_heavy_panic() {
    panic::set_hook(Box::new(|panic_info| {
        let backtrace = Backtrace::new();

        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            error!("Panic occurred: {:?}", s);
        }

        // Get code location
        let location = panic_info.location().unwrap();

        // extract msg
        let msg = match panic_info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match panic_info.payload().downcast_ref::<String>() {
                Some(s) => &s[..],
                None => "Box<Any>",
            },
        };

        let handle = thread::current();
        let thread_name = handle.name().unwrap_or("<unnamed>");

        error!(
            "thread '{}' panicked at '{}', {}",
            thread_name, location, msg
        );

        error!("{:?}", backtrace);

        std::process::exit(1)
    }));
}

/// Deserializes private key in hex format into [`k256::SecretKey`]
pub fn de_secp256k1_signing_key<'de, D>(
    deserializer: D,
) -> Result<k256::ecdsa::SigningKey, D::Error>
where
    D: de::Deserializer<'de>,
{
    match de_opt_secp256k1_signing_key(deserializer)? {
        Some(key) => Ok(key),
        None => Err(serde::de::Error::custom("Missing signing key")),
    }
}

/// Deserializes optional private key in hex format into [`k256::SecretKey`]
pub fn de_opt_secp256k1_signing_key<'de, D>(
    deserializer: D,
) -> Result<Option<k256::ecdsa::SigningKey>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let Some(string) = Option::<String>::deserialize(deserializer)? else {
        return Ok(None);
    };

    let bytes = hex::decode(skip_hex_prefix(&string))
        .map_err(|err| de::Error::custom(format!("Not supported format: {}", err)))?;
    k256::ecdsa::SigningKey::from_slice(&bytes)
        .map_err(|err| de::Error::custom(format!("Not a private key: {}", err)))
        .map(Some)
}

/// Custom deserializer could be used to deserialize optional `DateTime` property to `None` if not exist
pub mod secp256k1_signing_key_opt_str {
    use serde::{Deserializer, Serialize, Serializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<k256::ecdsa::SigningKey>, D::Error>
    where
        D: Deserializer<'de>,
    {
        super::de_opt_secp256k1_signing_key(deserializer)
    }

    pub fn serialize<S>(
        value: &Option<k256::ecdsa::SigningKey>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        value
            .as_ref()
            .map(|key| ["0x", &hex::encode(key.to_bytes())].concat())
            .serialize(serializer)
    }
}

pub fn get_eth_address(uncompressed_public_key: &[u8]) -> H160 {
    H160::from_slice(
        &Keccak256::new_with_prefix(&uncompressed_public_key[1..])
            .finalize()
            .as_slice()[12..],
    )
}

pub fn skip_hex_prefix(input: &str) -> &str {
    match input.strip_prefix("0x") {
        Some(input) => input,
        None => input,
    }
}
