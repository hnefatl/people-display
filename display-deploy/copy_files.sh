#!/bin/bash

if [[ $# -ne 1 ]] ; 
    echo "Usage copy_files.sh <sd device>"
    echo "The device must not be mounted, and must be in /dev/sda format."
fi
dev="$1"

# Create a directory for mounting the device to.
root_dir="$(mktemp -d)"
# Mount all bits and pieces
mount "${dev}1" "$root_dir"
mount "${dev}2" "$root_dir/boot"

# Copy data over
bash dietpi.txt.sh > "$root_dir/boot/dietpi.txt"
bash dietpi-wifi.txt.sh > "$root_dir/boot/dietpi-wifi.txt"
cp post_install.sh "$root_dir/Automation_Custom_Script.sh"

# Clean up
umount "$root_dir/boot"
umount "$root_dir"
rmdir "$root_dir"
