use std::{net::IpAddr, str::FromStr};

use anyhow::anyhow;
use futures_util::{future::BoxFuture, FutureExt};

use super::Phase;
use crate::{error, messages};
use messages::MessageType;

pub struct SelectNodeIP {
    pub node_ip: Option<IpAddr>,
}

impl SelectNodeIP {
    pub fn new(node_ip: Option<IpAddr>) -> Self {
        Self { node_ip }
    }
}

impl Phase for SelectNodeIP {
    fn run<'a>(&'a mut self) -> BoxFuture<'a, Result<(), error::AppError>> {
        async {
            if self.node_ip.is_none() {
                let my_ip = fetch_my_ip().await?;

                if cliclack::confirm(MessageType::NodeIpConfirmRequest { ip: my_ip }).interact()? {
                    self.node_ip = Some(my_ip);
                } else {
                    let ip_text: String = cliclack::input(MessageType::NodeIpInputManually)
                        .validate_interactively(|input: &String| validate_ip_input(input, true))
                        .validate(|input: &String| validate_ip_input(input, false))
                        .interact()?;
                    self.node_ip = Some(IpAddr::from_str(&ip_text)?);
                }
            }

            if let Some(ip) = &self.node_ip {
                cliclack::note("Node IP check", MessageType::NodeIpInfo { ip })?;
                Ok(())
            } else {
                Err(anyhow!("No IP specified for node").into())
            }
        }
        .boxed()
    }
}

async fn fetch_my_ip() -> anyhow::Result<IpAddr> {
    let res = reqwest::get("https://api.ipify.org/").await?;
    let text = res.text().await?;
    IpAddr::from_str(&text).map_err(anyhow::Error::from)
}

fn validate_ip_input(input: &str, interactive: bool) -> anyhow::Result<()> {
    let valid_length = if interactive {
        input.len() <= 15
    } else {
        input.len() >= 7 && input.len() <= 15
    };
    if !valid_length {
        anyhow::bail!("{}", MessageType::NodeIpInvalidFormat { ip: input });
    }

    let invalid_format = input.chars().any(|c| !c.is_ascii_digit() && c != '.');
    if invalid_format {
        anyhow::bail!("{}", MessageType::NodeIpInvalidFormat { ip: input });
    }

    if interactive {
        Ok(())
    } else {
        IpAddr::from_str(input)
            .map_err(|_| anyhow!("{}", MessageType::NodeIpInvalidFormat { ip: input }))
            .map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_private_key_input() {
        assert!(validate_ip_input("0", true).is_ok());
        assert!(validate_ip_input("0.", true).is_ok());
        assert!(validate_ip_input("a", true).is_err());
        assert!(validate_ip_input("0.0.0.0", true).is_ok());
        assert!(validate_ip_input("255.255.255.255", true).is_ok());
        assert!(validate_ip_input("255.255.255.2551", true).is_err());
        assert!(validate_ip_input("257.255.255.255", true).is_ok());
        assert!(validate_ip_input("257.255.255.255", false).is_err());
        assert!(validate_ip_input("257.255.0.0.", true).is_ok());
        assert!(validate_ip_input("257.255.0.0.", false).is_err());
    }
}
