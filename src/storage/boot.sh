#!/bin/bash
# This script is executed by systemd at boot time.
# This script gets run as root.

source /storage/settings.conf
export SLINT_BACKEND
export SLINT_SCALE_FACTOR

if [ "$ENABLE_DASH" = "true" ]; then
    exec /usr/local/bin/rusty-dash # SLINT_BACKEND and SLINT_SCALE_FACTOR is set in settings.conf
else
    systemctl start getty@tty1.service
fi