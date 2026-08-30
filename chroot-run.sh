#!/bin/bash -e
set -e

pacman-key --init
pacman-key --populate archlinuxarm
pacman -Sy --noconfirm

pacman -S --noconfirm --needed git rust base-devel

# makepkg refuses to run as root, create a temporary build user with passwordless sudo
useradd -m builder
echo "builder ALL=(ALL) NOPASSWD: ALL" > /etc/sudoers.d/builder

# Install stuff
sudo -u builder bash -c '
  cd ~
  git clone https://aur.archlinux.org/paru.git
  cd paru
  makepkg -si --noconfirm
  cd ~

  paru -S --noconfirm --needed \
    fastfetch \
    exfatprogs \
    networkmanager \
    labwc \
    quickshell \
    swaybg \
    foot

  # Clean system
  paru -Rns $(paru -Qtdq) --noconfirm
  paru -Sccd --noconfirm
'

# Clean up the temporary build user and its sudo grant
userdel -r builder
rm /etc/sudoers.d/builder

systemctl enable NetworkManager

# Fix home dir permissions
chown -R alarm:alarm /home/alarm

# mkdir -p /storage/music
# chmod -R 777 /storage
