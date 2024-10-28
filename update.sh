#!/usr/bin/env bash
set -e
cd "$( dirname "$(readlink -f "${BASH_SOURCE[0]}")" )"
if [[ -d /etc/cron.daily ]]; then
  rm -f /etc/cron.daily/airdao-nop-rs
  ln -fs $PWD/update.sh /etc/cron.daily/airdao-nop-rs
fi

cat > /etc/sysctl.d/10-airdao.conf <<-END
net.ipv6.conf.all.disable_ipv6=1
END
sysctl -p /etc/sysctl.d/10-airdao.conf

cd ~/airdao-nop-rs

CURRENT_VERSION=$(./airdao-nop-rs --version | cut -d ' ' -f2)
LATEST_VERSION=$(curl -s https://raw.githubusercontent.com/ambrosus/airdao-nop-rs/main/Cargo.toml | grep '^version' | sed -E 's/version = "(.*)"/\1/')

if [[ "$CURRENT_VERSION" != "$LATEST_VERSION" ]]; then    
    DEBIAN_VERSION=$(lsb_release -sr | cut -d '.' -f 1)
    UBUNTU_VERSION=$(lsb_release -sr | cut -d '.' -f 1)
    if (( DEBIAN_VERSION > 11 )) || (( UBUNTU_VERSION >= 22 )); then
        FILE_URL="https://github.com/ambrosus/airdao-nop-rs/releases/download/v$LATEST_VERSION/airdao-nop-rs-x86-64.zip"
    else
        FILE_URL="https://github.com/ambrosus/airdao-nop-rs/releases/download/v$LATEST_VERSION/airdao-nop-rs-x86-64-old.zip"
    fi

    curl -L -o airdao-nop-rs.zip "$FILE_URL"
    unzip -o airdao-nop-rs.zip
    chmod +x ./airdao-nop-rs

    ./airdao-nop-rs update
else
fi
