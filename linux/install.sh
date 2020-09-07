#! /usr/bin/env bash
set -e
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

cd "$DIR/../src/ui/platune"
yarn linux-pack
sudo mkdir -p /opt/platune
sudo install -d /opt/platune
sudo install -m644 public/res/icon.png /usr/share/icons/platune.png
sudo install -m655 dist/Platune-0.1.0.AppImage /opt/platune/platune.AppImage
sudo install -m644 "$DIR/platune.desktop" /usr/share/applications/platune.desktop
cd ../../..
cargo build --release
sudo install -m655 target/release/platune /opt/platune/platune-server
