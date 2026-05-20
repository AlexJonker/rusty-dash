#!/bin/bash
# This script is executed by systemd at boot time.

source /storage/settings.conf
export SLINT_BACKEND

if [ "$ENABLE_DASH" = "true" ]; then
    exec /usr/local/bin/rusty-dash # SLINT_BACKEND is set in settings.conf
else
    exec /sbin/agetty --autologin driver --noclear tty1 linux
fi