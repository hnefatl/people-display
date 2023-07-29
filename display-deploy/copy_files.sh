#!/bin/bash

if [[ $# -ne 1 ]] ; then
    echo "Usage copy_files.sh <sd device>"
    echo "The device must not be mounted, and must be in /dev/sda format."
    exit -1
fi
dev="$1"

if [ ! -f preseed-variables.sh ] ; then
    echo "A file called 'preseed-variables.sh' must exist, containing bash variable definitions for seeding into these files."
    echo "Required variables are INIT_WIFI_SSID, INIT_WIFI_PSK, DEST_WIFI_SSID, DEST_WIFI_PSK, PASSWORD"
    exit -1
fi

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
