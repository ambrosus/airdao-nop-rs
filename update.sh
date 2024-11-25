#!/usr/bin/env bash
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
LATEST_TAG=$(curl -s https://raw.githubusercontent.com/ambrosus/airdao-nop-rs/main/Cargo.toml | grep '^version' | sed -E 's/version = "(.*)"/\1/')
DIR_NAME="airdao-nop-rs"
ZIP_FILE="${DIR_NAME}.zip"

install_airdao() {
    cd ~

    curl -L -o "$ZIP_FILE" "$FILE_URL" || { echo "Failed to download file"; exit 1; }

    unzip -o "$ZIP_FILE" || { echo "Failed to unzip file"; exit 1; }
    rm "$ZIP_FILE"
    cd "$DIR_NAME"

    if cosign verify-blob --key airdao-nop-rs.pub --signature airdao-nop-rs.sig airdao-nop-rs; then
        echo -e "\033[0;32mVerified OK\033[0m"
    else
        rm -rf ~/airdao-nop-rs 
        echo -e "\033[0;31mError: Verification failed\033[0m"
        exit 1
    fi

    chmod +x ./airdao-nop-rs

    ./airdao-nop-rs update
}

if [[ "$CURRENT_VERSION" != "$LATEST_TAG" ]]; then
    DISTRO_NAME=$(lsb_release -i | cut -d ':' -f 2 | xargs)
    MAJOR_VERSION=$(lsb_release -sr | cut -d '.' -f 1)
    if [[ "$DISTRO_NAME" == "Ubuntu" ]]; then
        if (( MAJOR_VERSION >= 22 )); then
            FILE_URL="https://github.com/ambrosus/airdao-nop-rs/releases/download/v$LATEST_TAG/airdao-nop-rs-x86-64.zip"
        else
            FILE_URL="https://github.com/ambrosus/airdao-nop-rs/releases/download/v$LATEST_TAG/airdao-nop-rs-x86-64-old.zip"
        fi
    elif [[ "$DISTRO_NAME" == "Debian" ]]; then
        if (( MAJOR_VERSION > 11 )); then
            FILE_URL="https://github.com/ambrosus/airdao-nop-rs/releases/download/v$LATEST_TAG/airdao-nop-rs-x86-64.zip"
        else
            FILE_URL="https://github.com/ambrosus/airdao-nop-rs/releases/download/v$LATEST_TAG/airdao-nop-rs-x86-64-old.zip"
        fi
    else
        exit 1
    fi
    install_airdao
fi
