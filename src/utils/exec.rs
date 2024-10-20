use anyhow::anyhow;
use regex::Regex;
use std::{
    path::PathBuf,
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

pub fn run_docker_compose_up() -> anyhow::Result<()> {
    let output_dir = utils::output_dir();

    match Command::new("docker-compose")
        .current_dir(output_dir)
        .arg("up")
        .arg("-d")
        .output()?
    {
        Output { status, .. } if status.success() => Ok(()),
        Output { status, stderr, .. } => Err(anyhow!(
            "Run `docker-compose up -d` failure ({:?}). Error: {:?}",
            status.code(),
            std::str::from_utf8(&stderr)
        )),
    }
}

pub fn run_docker_compose_down() -> anyhow::Result<()> {
    let output_dir = utils::output_dir();

    match Command::new("docker-compose")
        .current_dir(output_dir)
        .arg("down")
        .output()?
    {
        Output { status, .. } if status.success() => Ok(()),
        Output { status, stderr, .. } => Err(anyhow!(
            "Run `docker-compose down` failure ({:?}). Error: {:?}",
            status.code(),
            std::str::from_utf8(&stderr)
        )),
    }
}

pub fn run_docker_compose_pull() -> anyhow::Result<()> {
    let output_dir = utils::output_dir();

    match Command::new("docker-compose")
        .current_dir(output_dir)
        .arg("pull")
        .output()?
    {
        Output { status, .. } if status.success() => Ok(()),
        Output { status, stderr, .. } => Err(anyhow!(
            "Run `docker-compose pull` failure ({:?}). Error: {:?}",
            status.code(),
            std::str::from_utf8(&stderr)
        )),
    }
}

pub async fn run_download_backup(url: &str) -> anyhow::Result<()> {
    let output_dir = utils::output_dir();

    match tokio::process::Command::new("curl")
        .current_dir(output_dir)
        .arg("-s")
        .arg(url)
        .arg("|")
        .arg("tar")
        .arg("zxpf")
        .arg("-")
        .output()
        .await?
    {
        Output { status, .. } if status.success() => Ok(()),
        Output { status, stderr, .. } => Err(anyhow!(
            "Run `curl -s {} | tar zxpf -` failure ({:?}). Error: {:?}",
            url,
            status.code(),
            std::str::from_utf8(&stderr)
        )),
    }
}

pub fn docker_compose_restart() -> anyhow::Result<()> {
    run_docker_compose_down()?;
    run_docker_compose_pull()?;
    run_docker_compose_up()
}

pub async fn run_update(update_file: PathBuf) -> anyhow::Result<()> {
    match tokio::process::Command::new(update_file).output().await? {
        Output { status, .. } if status.success() => Ok(()),
        Output { status, stderr, .. } => Err(anyhow!(
            "Run `./update.sh` failure ({:?}). Error: {:?}",
            status.code(),
            std::str::from_utf8(&stderr)
        )),
    }
}
