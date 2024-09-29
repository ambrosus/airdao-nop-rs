use regex::Regex;
use std::{process::Command, str};

const DEFAULT_OUTPUT_DIRECTORY: &str = "./output";

pub fn is_docker_installed() -> anyhow::Result<bool> {
    let docker_version_regexp = Regex::new(r"^Docker version ([0-9.\-a-z]+), build ([0-9a-f]+)")?;

    let output = Command::new("docker").arg("-v").output()?;
    let stdout_str = str::from_utf8(&output.stdout)?;

    Ok(docker_version_regexp.find(stdout_str).is_some())
}

pub fn run_docker() -> anyhow::Result<()> {
    let output_dir =
        std::env::var("OUTPUT_DIRECTORY").unwrap_or_else(|_| DEFAULT_OUTPUT_DIRECTORY.to_string());

    let _ = Command::new("docker-compose")
        .arg("up")
        .arg("-d")
        .arg("cwd")
        .arg(output_dir)
        .output()?;

    Ok(())
}
