#!/bin/bash

if [[ $# -ne 1 ]] ; then
    echo "Usage copy_files.sh <sd device>"
    echo "The device must not be mounted, and must be in /dev/sda format."
    exit -1
fi
dev="$1"

if [[ -f preseed-variables.sh ]] ; then
    source preseed-variables.sh
fi

[[ -z $INIT_WIFI_SSID ]] && echo "Required INIT_WIFI_SSID: SSID for WiFi network where the device is being configured." && exit -1
[[ -z $INIT_WIFI_PSK ]] && echo "Required INIT_WIFI_PSK: PSK for WiFi network where the device is being configured." && exit -1
[[ -z $DEST_WIFI_SSID ]] && echo "Required DEST_WIFI_SSID: SSID for WiFi network where the device will be installed." && exit -1
[[ -z $DEST_WIFI_PSK ]] && echo "Required DEST_WIFI_PSK: PSK for WiFi network where the device will be installed." && exit -1
[[ -z $PASSWORD ]] && echo "Required PASSWORD: User password to configure." && exit -1
[[ -z $SSH_PUBLIC_KEY ]] && echo "Required SSH_PUBLIC_KEY: Public key to add to .authorized_keys for remote connection, in 'ssh-ed25519 abc name@foo' format." && exit -1
[[ -z $WIREGUARD_CONFIG_FILE ]] && echo "Required WIREGUARD_CONFIG_FILE: Path to wireguard client configuration file to copy." && exit -1
[[ -z $DISPLAY_CONFIG_PATH ]] && echo "Config file for the display." && exit -1

function cleanup {
    umount "$root_dir/boot"
    umount "$root_dir"
    rmdir "$root_dir"
}

# Create a directory for mounting the device to.
root_dir="$(mktemp -d)" || exit 1
# Make sure to cleanup on exit
trap cleanup EXIT
# Mount all bits and pieces
mount "${dev}2" "$root_dir" || exit 1
mount "${dev}1" "$root_dir/boot" || exit 1

# Copy data over
bash dietpi.txt.sh > "$root_dir/boot/dietpi.txt" || exit 2
bash dietpi-wifi.txt.sh > "$root_dir/boot/dietpi-wifi.txt" || exit 2
cp post_install.sh "$root_dir/boot/Automation_Custom_Script.sh" || exit 2
mkdir -p "$root_dir/etc/wireguard" || exit 2
cp "$WIREGUARD_CONFIG_FILE" "$root_dir/etc/wireguard/wg0.conf" || exit 2

# Static files that don't need any variable expansions
cp -r files "$root_dir/files" || exit 2

cp "$DISPLAY_CONFIG_PATH" "$root_dir/files/display_config"
