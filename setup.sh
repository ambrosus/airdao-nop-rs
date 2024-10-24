#!/bin/bash

# Change /etc/needrestart/needrestart.conf to skip confirmations for restarting required services
sed -i 's/^#\$nrconf{restart} = '\''i'\'';/$nrconf{restart} = '\''a'\'';/' /etc/needrestart/needrestart.conf

apt-get install -y \
    libssl-dev \
    pkg-config \
    ca-certificates \
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

git clone https://github.com/ambrosus/airdao-nop-rs.git
cd airdao-nop-rs || return

chmod +x update.sh
./update.sh

LATEST_TAG=$(curl -s https://api.github.com/repos/ambrosus/airdao-nop-rs/releases/latest | jq -r .tag_name)
DEBIAN_VERSION=$(lsb_release -sr)
UBUNTU_VERSION=$(lsb_release -sr)

if (( $(echo "$DEBIAN_VERSION > 11" | bc -l) )) || (( $(echo "$UBUNTU_VERSION >= 22" | bc -l) )); then
    FILE_URL="https://github.com/ambrosus/airdao-nop-rs/releases/download/$LATEST_TAG/airdao-nop-rs-x86-64.zip"
else
    FILE_URL="https://github.com/ambrosus/airdao-nop-rs/releases/download/$LATEST_TAG/airdao-nop-rs-x86-64-old.zip"
fi

curl -L -o airdao-nop-release.zip "$FILE_URL"

unzip airdao-nop-release.zip
rm airdao-nop-release.zip
chmod +x ./airdao-nop-rs

./airdao-nop-rs
