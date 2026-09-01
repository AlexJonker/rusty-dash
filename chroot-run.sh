#!/bin/bash -e
set -e

pacman-key --init
pacman-key --populate archlinuxarm
pacman -Sy --noconfirm

ls /boot

# Uninstall old kernel
pacman -Rns --noconfirm linux-aarch64 uboot-raspberrypi

ls /boot

# Install rpi-linux kernel
pacman -S --noconfirm linux-rpi linux-rpi-headers
pacman -Syu --noconfirm

ls /boot

# Install sudo
pacman -S --noconfirm sudo

# makepkg refuses to run as root, create a temporary build user with passwordless sudo
useradd -m builder
echo "builder ALL=(ALL) NOPASSWD: ALL" >/etc/sudoers.d/builder

# Install everything
pacman -S --noconfirm --needed git base-devel

sudo -u builder bash -c '
  set -e
  cd ~
  git clone https://aur.archlinux.org/qt5-connectivity.git
  cd qt5-connectivity
  makepkg -si --noconfirm
  cd ~
'

pacman -S --noconfirm --needed \
	cmake \
	ninja \
	boost \
	boost-libs \
	libusb \
	protobuf \
	openssl \
	qt5-base \
	qt5-multimedia \
	rtaudio

pacman -S --noconfirm --needed \
	exfatprogs \
	networkmanager \
	labwc \
	quickshell \
	swaybg \
	foot \
	android-udev

# --- Build aasdk ---
ANDROID_AUTO_SRC="/root/android-auto"
BUILD_DIR="/root/build"
INSTALL_PREFIX="/opt/nowa"

mkdir -p "$BUILD_DIR"
chown -R builder:builder "$ANDROID_AUTO_SRC" "$BUILD_DIR"
chmod 755 /root

sudo -u builder bash -c "
  set -e
  cmake -S '$ANDROID_AUTO_SRC/aasdk' -B '$BUILD_DIR/aasdk' \
    -GNinja \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_INSTALL_PREFIX='$INSTALL_PREFIX'
  cmake --build '$BUILD_DIR/aasdk' -j\$(nproc)
"
# install needs root to write to /opt
cmake --install "$BUILD_DIR/aasdk"

# Refresh linker cache so the openauto build below can find libaasdk/libaasdk_proto
echo "$INSTALL_PREFIX/lib" >/etc/ld.so.conf.d/nowa.conf
ldconfig

# --- Build openauto ---
sudo -u builder bash -c "
  set -e
  cmake -S '$ANDROID_AUTO_SRC/openauto' -B '$BUILD_DIR/openauto' \
    -GNinja \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_INSTALL_PREFIX='$INSTALL_PREFIX' \
    -DAASDK_INCLUDE_DIRS='$INSTALL_PREFIX/include' \
    -DAASDK_PROTO_INCLUDE_DIRS='$INSTALL_PREFIX/include' \
    -DAASDK_LIBRARIES='$INSTALL_PREFIX/lib/libaasdk.so' \
    -DAASDK_PROTO_LIBRARIES='$INSTALL_PREFIX/lib/libaasdk_proto.so'
  cmake --build '$BUILD_DIR/openauto' -j\$(nproc)
"
cmake --install "$BUILD_DIR/openauto"

# Sanity check: fail the build now, not on first boot, if the binary can't resolve its libs
if ! ldd "$INSTALL_PREFIX/bin/autoapp" | grep -q "not found"; then
	echo "autoapp: all shared libraries resolved"
else
	echo "ERROR: autoapp has unresolved shared libraries:"
	ldd "$INSTALL_PREFIX/bin/autoapp" | grep "not found"
	exit 1
fi

# Clean up build tree and source checkout — not needed on the final image
rm -rf "$BUILD_DIR" "$ANDROID_AUTO_SRC"

pacman -Rns --noconfirm git cmake base-devel ninja boost
pacman -Scc --noconfirm || true
rm -rf /var/cache/pacman/pkg/*
rm -rf /var/cache/pacman/pkg/.[!.]*
rm -rf /var/cache/pacman/pkg/..?*

# Clean up the temporary build user and its sudo grant
userdel -r builder
rm /etc/sudoers.d/builder

systemctl enable NetworkManager

# Fix home dir permissions
chown -R alarm:alarm /home/alarm

# Generate en_US.UTF-8 locale
locale-gen

# mkdir -p /storage/music
# chmod -R 777 /storage
