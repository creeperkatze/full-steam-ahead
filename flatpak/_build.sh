#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")"
flatpak run org.flatpak.Builder --force-clean --user --disable-rofiles-fuse \
  build-dir dev.creeperkatze.full-steam-ahead.yml
