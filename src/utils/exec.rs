use anyhow::anyhow;
use futures::FutureExt;
use regex::Regex;
use std::{
    path::PathBuf,
    process::{Command, Output},
    str,
};

use crate::{error::AppError, utils};

pub fn is_docker_installed() -> Result<bool, AppError> {
    let docker_version_regexp =
        Regex::new(r"^Docker version ([0-9.\-a-z]+)(?:\+[^,]*)?, build ([0-9a-f]+)")?;

    let output = Command::new("docker").arg("-v").output()?;
    let stdout_str = str::from_utf8(&output.stdout)?;

    Ok(docker_version_regexp.find(stdout_str).is_some())
}

pub fn run_docker_compose_up() -> Result<(), AppError> {
    let output_dir = utils::output_dir();

    match Command::new("docker-compose")
        .current_dir(output_dir)
        .arg("--compatibility")
        .arg("up")
        .arg("-d")
        .output()?
    {
        Output { status, .. } if status.success() => Ok(()),
        Output { status, stderr, .. } => Err(anyhow!(
            "Run `docker-compose up -d` failure ({:?}). Error: {:?}",
            status.code(),
            std::str::from_utf8(&stderr)
        )
        .into()),
    }
}

pub fn run_docker_compose_down() -> Result<(), AppError> {
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
        )
        .into()),
    }
}

pub fn run_docker_compose_pull() -> Result<(), AppError> {
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
        )
        .into()),
    }
}

pub async fn run_download_backup(url: &str) -> Result<(), AppError> {
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
        )
        .into()),
    }
}

pub fn docker_compose_restart() -> Result<(), AppError> {
    run_docker_compose_down()?;
    run_docker_compose_pull()?;
    run_docker_compose_up()
}

pub async fn run_update(update_file: PathBuf) -> Result<(), AppError> {
    match tokio::process::Command::new(update_file).output().await? {
        Output { status, .. } if status.success() => Ok(()),
        Output { status, stderr, .. } => Err(anyhow!(
            "Run `./update.sh` failure ({:?}). Error: {:?}",
            status.code(),
            std::str::from_utf8(&stderr)
        )
        .into()),
    }
}

// pub async fn get_parity_container_ip() -> Result<IpAddr, AppError> {
//     match tokio::process::Command::new("docker")
//         .arg("inspect")
//         .arg("-f")
//         .arg("\"{{range.NetworkSettings.Networks}}{{.IPAddress}}{{end}}\"")
//         .arg("parity")
//         .output()
//         .await?
//     {
//         Output { status, stdout, .. } if status.success() => {
//             IpAddr::from_str(str::from_utf8(&stdout)?.trim().trim_matches('"')).map_err(AppError::from)
//         },
//         Output { status, stderr, .. } => Err(anyhow!(
//             "Run `docker inspect -f \"{{range.NetworkSettings.Networks}}{{.IPAddress}}{{end}}\" parity` failure ({:?}). Error: {:?}",
//             status.code(),
//             std::str::from_utf8(&stderr)
//         )
//         .into()),
//     }
// }

pub fn get_os_release() -> String {
    super::output_into_string(Command::new("cat").arg("/etc/os-release").output())
}

pub fn get_mem_info() -> String {
    super::output_into_string(Command::new("cat").arg("/proc/meminfo").output())
}

pub fn get_directory_contents(cwd: Option<PathBuf>) -> String {
    let mut cmd = Command::new("ls");

    if let Some(cwd) = cwd {
        cmd.current_dir(cwd);
    }

    super::output_into_string(cmd.arg("-la").output())
}

pub fn get_disk_block_info() -> String {
    super::output_into_string(Command::new("df").arg("-h").output())
}

pub fn get_disk_inodes_info() -> String {
    super::output_into_string(Command::new("df").arg("-i").output())
}

pub fn get_process_tree() -> String {
    super::output_into_string(Command::new("ps").arg("axjf").output())
}

pub fn get_memory_usage() -> String {
    super::output_into_string(Command::new("free").arg("-m").output())
}

pub fn get_docker_compose_logs() -> String {
    super::output_into_string(
        Command::new("docker-compose")
            .current_dir(super::output_dir())
            .arg("logs")
            .arg("--tail=500")
            .output(),
    )
}

#[allow(unused)]
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

    let local_head = super::output_into_string(
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
                _ => Err(super::output_into_string(branch_res)),
            }?;

            Ok(super::output_into_string(
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
    let res = Command::new("docker")
        .stdin(std::process::Stdio::piped())
        .arg("inspect")
        .arg("--format='{{ index .Config.Image }}'")
        .arg("parity")
        .output();

    let success = matches!(&res, Ok(Output { status, .. }) if status.success());
    let output = super::output_into_string(res);

    if success {
        let Some((_, version)) = output.trim().trim_matches('\'').split_once(':') else {
            return output;
        };

        version.to_owned()
    } else {
        output
    }
}
