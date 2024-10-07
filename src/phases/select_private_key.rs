use std::marker::PhantomData;

use anyhow::anyhow;
use cliclack::Validate;
use futures_util::{future::BoxFuture, FutureExt};
use k256::ecdsa::SigningKey;
use rand::rngs::OsRng;

use super::Phase;
use crate::{error, messages};
use messages::MessageType;

pub struct SelectPrivateKeyPhase {
    private_key: Option<SigningKey>,
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
            if self.private_key.is_some() {
                return Ok(());
            }

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
                            if input.len() > 64 {
                                Err(anyhow!("Private key invalid length (64 characters max)"))
                            } else if input.chars().any(|c| !c.is_ascii_hexdigit()) {
                                Err(anyhow!("Private key should be in hex form"))
                            } else {
                                Ok(())
                            }
                        })
                        .validate(|input: &String| {
                            if input.len() != 64 {
                                Err(anyhow!("Private key invalid length (64 characters max)"))
                            } else if input.chars().any(|c| !c.is_ascii_hexdigit()) {
                                Err(anyhow!("Private key should be in hex form"))
                            } else {
                                Ok(())
                            }
                        })
                        .interact()?;
                    self.private_key = Some(SigningKey::from_slice(&hex::decode(key)?)?);
                }
                PrivateKeyInputKind::Generate => {
                    self.private_key = Some(SigningKey::random(&mut OsRng));
                }
            };

            if self.private_key.is_some() {
                Ok(())
            } else {
                Err(anyhow!("No private key specified").into())
            }
        }
        .boxed()
    }
}
