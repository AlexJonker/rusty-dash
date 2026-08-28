#!/bin/bash -e
set -e

pacman-key --init
pacman-key --populate archlinuxarm
pacman -Sy --noconfirm

pacman -S --noconfirm --needed git rust base-devel

# makepkg refuses to run as root, create a temporary build user with passwordless sudo
useradd -m builder
echo "builder ALL=(ALL) NOPASSWD: ALL" >> /etc/sudoers

# Bootstrap paru (as builder, not root)
sudo -u builder bash -c '
  cd /home/builder
  git clone https://aur.archlinux.org/paru.git
  cd paru
  makepkg -si --noconfirm
  cd ..
  rm -fr paru

  paru -S --noconfirm --needed fastfetch exfatprogs quickshell

  paru -S --noconfirm --needed --mflags "--ignorearch" mangowm


  # Clean system
  paru -Rns $(paru -Qtdq) --noconfirm
  paru -Sccd --noconfirm
'

# Clean up the temporary build user and its sudo grant
userdel -r builder
sed -i '/builder ALL=(ALL) NOPASSWD: ALL/d' /etc/sudoers

# mkdir -p /storage/music
# chmod -R 777 /storage
