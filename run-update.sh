#!/usr/bin/env bash
set -e

git submodule update --init --recursive
cargo build --release
#TODO: run update
