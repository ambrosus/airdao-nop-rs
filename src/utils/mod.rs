pub mod config;
pub mod debug_info;
pub mod exec;
pub mod logger;

use backtrace::Backtrace;
use ethereum_types::{Address, H160};
use futures::FutureExt;
use log::error;
use serde::{de, Deserialize};
use sha3::{Digest, Keccak256};
use std::{
    panic,
    path::PathBuf,
    process::{Command, Output},
    thread,
};

const DEFAULT_OUTPUT_DIRECTORY: &str = "./output";

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

pub fn secp256k1_signing_key_to_eth_address(key: &k256::ecdsa::SigningKey) -> Address {
    get_eth_address(key.verifying_key().to_encoded_point(false).as_bytes())
}

pub fn output_dir() -> PathBuf {
    PathBuf::from(
        std::env::var("OUTPUT_DIRECTORY")
            .as_deref()
            .unwrap_or(DEFAULT_OUTPUT_DIRECTORY),
    )
}

pub fn output_into_string(output: Result<Output, std::io::Error>) -> String {
    match output {
        Ok(Output { status, stdout, .. }) if status.success() => {
            match std::str::from_utf8(&stdout) {
                Ok(text) => text.to_owned(),
                Err(e) => format!("Format error: {e:?}. Raw: {stdout:?}"),
            }
        }
        Ok(Output { status, stderr, .. }) => format!(
            "Status: {}. Error: {}",
            status.code().unwrap_or_default(),
            std::str::from_utf8(&stderr).unwrap_or_default()
        ),
        Err(e) => {
            format!("I/O error: {e:?}")
        }
    }
}

pub fn get_os_release() -> String {
    output_into_string(Command::new("cat").arg("/etc/os-release").output())
}

pub fn get_mem_info() -> String {
    output_into_string(Command::new("cat").arg("/proc/meminfo").output())
}

pub fn get_directory_contents(cwd: Option<PathBuf>) -> String {
    let mut cmd = Command::new("ls");

    if let Some(cwd) = cwd {
        cmd.current_dir(cwd);
    }

    output_into_string(cmd.arg("-la").output())
}

pub fn get_disk_block_info() -> String {
    output_into_string(Command::new("df").arg("-h").output())
}

pub fn get_disk_inodes_info() -> String {
    output_into_string(Command::new("df").arg("-i").output())
}

pub fn get_process_tree() -> String {
    output_into_string(Command::new("ps").arg("axjf").output())
}

pub fn get_memory_usage() -> String {
    output_into_string(Command::new("free").arg("-m").output())
}

pub fn get_docker_compose_logs() -> String {
    output_into_string(
        Command::new("docker-compose")
            .current_dir(output_dir())
            .arg("logs")
            .arg("--tail=500")
            .output(),
    )
}

pub async fn get_git_commits() -> (String, String) {
    if let Ok(mut child) = tokio::process::Command::new("git")
        .arg("fetch")
        .arg("origin")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        let _ = child.wait().await;
    }

    let local_head = output_into_string(
        tokio::process::Command::new("git")
            .arg("rev-parse")
            .arg("HEAD")
            .output()
            .await,
    );

    // Try to acquire active branch name and then remote head commit for that branch
    match tokio::process::Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .output()
        .then(|branch_res| async move {
            let branch = match branch_res.as_ref() {
                Ok(Output { status, stdout, .. }) if status.success() => {
                    std::str::from_utf8(stdout)
                        .map_err(|e| format!("Format error: {e:?}. Raw: {stdout:?}"))
                }
                _ => Err(output_into_string(branch_res)),
            }?;

            Ok(output_into_string(
                tokio::process::Command::new("git")
                    .arg("rev-parse")
                    .arg(["origin/", branch.trim()].concat())
                    .output()
                    .await,
            ))
        })
        .await
    {
        Ok(remote_head) => (local_head, remote_head),
        Err(err) => (local_head, err),
    }
}

pub fn get_node_version() -> String {
    output_into_string(
        Command::new("docker")
            .arg("inspect")
            .arg("--format='{{ index .Config.Image }}'")
            .arg("parity")
            .arg("|")
            .arg("cut")
            .arg("-d':'")
            .arg("-f2")
            .output(),
    )
}

pub fn is_update_run() -> bool {
    match std::env::var("RUN_UPDATE").as_deref() {
        Ok("true") => true,
        _ => false,
    }
}
