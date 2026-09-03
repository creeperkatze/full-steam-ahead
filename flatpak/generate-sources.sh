#!/bin/bash
# Regenerates from the lockfiles.
set -euo pipefail
cd "$(dirname "$0")"

if [ ! -f flatpak-builder-tools/cargo/flatpak-cargo-generator.py ]; then
  echo "flatpak-builder-tools submodule not checked out - run:" >&2
  echo "  git submodule update --init flatpak/flatpak-builder-tools" >&2
  exit 1
fi

python3 -m pip install --quiet --user --break-system-packages toml tomlkit aiohttp pyyaml

python3 flatpak-builder-tools/cargo/flatpak-cargo-generator.py \
  ../src-tauri/Cargo.lock -o cargo-sources.json

( cd flatpak-builder-tools/node && python3 -m flatpak_node_generator \
    pnpm ../../../pnpm-lock.yaml -o ../../node-sources.json --pnpm-store-version v11 )
