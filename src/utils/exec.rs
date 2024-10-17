use anyhow::anyhow;
use regex::Regex;
use std::{
    process::{Command, Output},
    str,
};

use crate::utils;

pub fn is_docker_installed() -> anyhow::Result<bool> {
    let docker_version_regexp =
        Regex::new(r"^Docker version ([0-9.\-a-z]+)(?:\+[^,]*)?, build ([0-9a-f]+)")?;

    let output = Command::new("docker").arg("-v").output()?;
    let stdout_str = str::from_utf8(&output.stdout)?;

    Ok(docker_version_regexp.find(stdout_str).is_some())
}

pub fn run_docker() -> anyhow::Result<()> {
    let output_dir = utils::output_dir();

    match Command::new("docker-compose")
        .current_dir(output_dir)
        .arg("up")
        .arg("-d")
        .output()?
    {
        Output { status, .. } if status.success() => Ok(()),
        Output { status, stderr, .. } => Err(anyhow!(
            "Run `docker-compose` failure ({:?}). Error: {:?}",
            status.code(),
            std::str::from_utf8(&stderr)
        )),
    }
}
