use anyhow::anyhow;
use futures_util::{future::BoxFuture, FutureExt};
use k256::ecdsa::SigningKey;
use rand::rngs::OsRng;

use super::Phase;
use crate::{error, messages, utils};
use messages::MessageType;

pub struct SelectPrivateKeyPhase {
    pub private_key: Option<SigningKey>,
}

impl SelectPrivateKeyPhase {
    pub fn new(private_key: Option<SigningKey>) -> Self {
        Self { private_key }
    }
}

#[derive(Clone, PartialEq, Eq)]
enum PrivateKeyInputKind {
    Manual,
    Generate,
}

impl Phase for SelectPrivateKeyPhase {
    fn run<'a>(&'a mut self) -> BoxFuture<'a, Result<(), error::AppError>> {
        async {
            if self.private_key.is_none() {
                match cliclack::select(MessageType::NoPrivateKey)
                    .items(&[
                        (
                            PrivateKeyInputKind::Manual,
                            MessageType::PrivateKeyInputExistingSelection,
                            "",
                        ),
                        (
                            PrivateKeyInputKind::Generate,
                            MessageType::PrivateKeyGenerateNewSelection,
                            "",
                        ),
                    ])
                    .initial_value(PrivateKeyInputKind::Manual)
                    .interact()?
                {
                    PrivateKeyInputKind::Manual => {
                        let key: String = cliclack::input(MessageType::PrivateKeyInputManually)
                            .validate_interactively(|input: &String| {
                                validate_private_key_input(input, true)
                            })
                            .validate(|input: &String| validate_private_key_input(input, false))
                            .interact()?;
                        self.private_key = Some(SigningKey::from_slice(&hex::decode(
                            utils::skip_hex_prefix(&key),
                        )?)?);
                    }
                    PrivateKeyInputKind::Generate => {
                        self.private_key = Some(SigningKey::random(&mut OsRng));
                    }
                };
            }

            if let Some(private_key) = &self.private_key {
                cliclack::note(
                    "Private key check",
                    MessageType::PrivateKeyVerified {
                        address: utils::secp256k1_signing_key_to_eth_address(private_key),
                    },
                )?;
                Ok(())
            } else {
                Err(anyhow!("No private key specified").into())
            }
        }
        .boxed()
    }
}

fn validate_private_key_input(input: &str, interactive: bool) -> anyhow::Result<()> {
    let input = utils::skip_hex_prefix(input);

    let valid_length = if interactive {
        input.len() <= 64
    } else {
        input.len() == 64
    };
    if !valid_length {
        anyhow::bail!("{}", MessageType::PrivateKeyInvalidLength);
    }

    let invalid_format = input.chars().any(|c| !c.is_ascii_hexdigit());
    if invalid_format {
        anyhow::bail!("{}", MessageType::PrivateKeyInvalidFormat);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;

    #[test]
    fn test_validate_private_key_input() {
        assert!(validate_private_key_input("0x", true).is_ok());
        assert!(validate_private_key_input("0xab", true).is_ok());
        assert!(validate_private_key_input(
            "0xabababababababababababababababababababababababababababababababab",
            true
        )
        .is_ok());
        assert!(validate_private_key_input(
            "abababababababababababababababababababababababababababababababab",
            true
        )
        .is_ok());
        assert_matches!(
            validate_private_key_input("ababababababababababababababababababababababababababababababababc", true),
            Err(e) if e.to_string().starts_with(&MessageType::PrivateKeyInvalidLength.to_string()));
        assert_matches!(
            validate_private_key_input("abababababababababababababababababababababababababababababababc", false),
            Err(e) if e.to_string().starts_with(&MessageType::PrivateKeyInvalidLength.to_string()));
        assert_matches!(
            validate_private_key_input("abababababababababababababababababababababababababababababababax", false),
            Err(e) if e.to_string().starts_with(&MessageType::PrivateKeyInvalidFormat.to_string()));
    }
}
