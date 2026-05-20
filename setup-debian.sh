#!/bin/bash -e

set -euo pipefail

echo "=== rusty-dash Debian setup ==="

echo "[1/3] Installing runtime dependencies..."
sudo apt-get install -y --no-install-recommends \
    libfontconfig1 \
    libgbm1 \
    libxkbcommon0 \
    libudev1 \
    libseat1 \
    libinput10


echo "[2/3] Downloading binary..."

wget https://github.com/AlexJonker/rusty-dash/releases/latest/download/rusty-dash-amd64.xz
xz -d rusty-dash-amd64.xz

echo "[3/3] Installing binary and files..."
sudo mv rusty-dash-amd64 /usr/local/bin/rusty-dash
sudo chown root:root /usr/local/bin/rusty-dash
sudo chmod 755 /usr/local/bin/rusty-dash

sudo mkdir -p /storage/music
sudo cp -r src/storage/* /storage/
sudo chmod -R 777 /storage

sudo cp src/etc/systemd/system/startup.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable startup.service

echo ""
echo "=== Setup complete ==="
echo "Please reboot your system."
