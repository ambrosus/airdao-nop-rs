#!/bin/bash

# Change /etc/needrestart/needrestart.conf to skip confirmations for restarting required services
sed -i 's/^#\$nrconf{restart} = '\''i'\'';/$nrconf{restart} = '\''a'\'';/' /etc/needrestart/needrestart.conf

apt-get install -y \
    git \
    jq \
    unzip

if [ -f /etc/debian_version ]; then
    DISTRO=$(lsb_release -is)
    if [[ "$DISTRO" == "Debian" ]]; then

        if [ ! -d "/etc/apt/keyrings" ]; then
            sudo mkdir -p /etc/apt/keyrings
        fi

        curl -fsSL https://download.docker.com/linux/debian/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg
        echo \
          "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/debian \
          $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null

        apt-get update -y
        apt-get install -y \
            docker-ce \
            docker-ce-cli \
            containerd.io

        curl -L "https://github.com/docker/compose/releases/download/v2.21.0/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
        sudo chmod +x /usr/local/bin/docker-compose

    elif [[ "$DISTRO" == "Ubuntu" ]]; then

        if [ ! -d "/etc/apt/keyrings" ]; then
            sudo mkdir -p /etc/apt/keyrings
        fi

        curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg
        echo \
          "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu \
          $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null

        apt-get update -y
        apt-get install -y \
            docker-ce \
            docker-ce-cli \
            containerd.io

        curl -L "https://github.com/docker/compose/releases/download/v2.21.0/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
        sudo chmod +x /usr/local/bin/docker-compose

    else
        exit 1
    fi
else
    exit 1
fi

# Revert /etc/needrestart/needrestart.conf to original state after installing required packages
sed -i 's/^\$nrconf{restart} = '\''a'\'';/$nrconf{restart} = '\''i'\'';/' /etc/needrestart/needrestart.conf

LATEST_TAG=$(curl -s https://raw.githubusercontent.com/ambrosus/airdao-nop-rs/main/Cargo.toml | grep '^version' | sed -E 's/version = "(.*)"/\1/')
UBUNTU_MAJOR_VERSION=$(lsb_release -sr | cut -d '.' -f 1)
DEBIAN_MAJOR_VERSION=$(lsb_release -sr | cut -d '.' -f 1)

if (( DEBIAN_MAJOR_VERSION > 11 )) || (( UBUNTU_MAJOR_VERSION >= 22 )); then
    FILE_URL="https://github.com/ambrosus/airdao-nop-rs/releases/download/v$LATEST_TAG/airdao-nop-rs-x86-64.zip"
else
    FILE_URL="https://github.com/ambrosus/airdao-nop-rs/releases/download/v$LATEST_TAG/airdao-nop-rs-x86-64-old.zip"
fi

curl -L -o airdao-nop-release.zip "$FILE_URL"
unzip airdao-nop-release.zip
rm airdao-nop-release.zip

cd airdao-nop-rs || return

chmod +x update.sh
./update.sh

chmod +x ./airdao-nop-rs

./airdao-nop-rs
