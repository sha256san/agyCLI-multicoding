#!/bin/bash
set -e

# Initialize DBus session and unlock GNOME Keyring for headless Antigravity CLI Secret Service
if command -v dbus-launch >/dev/null 2>&1; then
    eval "$(dbus-launch --sh-syntax 2>/dev/null)" || true
fi

if command -v gnome-keyring-daemon >/dev/null 2>&1; then
    echo "" | gnome-keyring-daemon --unlock 2>/dev/null || true
    gnome-keyring-daemon --start --components=secrets,pkcs11,ssh 2>/dev/null || true
fi

export SSH_AUTH_SOCK="${SSH_AUTH_SOCK:-}"
exec "$@"
