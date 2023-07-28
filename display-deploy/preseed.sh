#!/bin/bash

# Print a preseed.cfg, using sensitive secrets imported from preeseed-variables.sh
source preeseed-variables.sh

echo <<EOF
# For use with Debian 12 installation media, for a reproducible linux installation.

d-i debian-installer/locale string en_GB

# Make sure wifi firmware is loaded
d-i hw-detect/load_firmware boolean true

# Random thing copied from debian preseed example file.
d-i netcfg/wireless_wep string
# Initial wifi details: for the wifi network near where the image is being build.
# See \`DEST_WIFI_*\` for wifi network where the device will be installed.
d-i netcfg/wireless_essid ${INIT_WIFI_SSID}
d-i netcfg/wireless_essid_again ${INIT_WIFI_SSID}
d-i netcfg/wireless_security_type WPA/WPA2 PSK
d-i netcfg/wireless_wpa ${INIT_WIFI_PSK}
d-i netcfg/choose_interface select auto
d-i netcfg/get_hostname string display-pi
d-i netcfg/get_domain string display-pi-domain

# Automatically partition the entire disk.
d-i partman-auto/init_automatically_partition
# No encryption or LVM.
d-i partman-auto/method string regular
d-i partman-auto/choose_recipe select All files in one partition (recommended for new users)
# Write without interactive confirmation
d-i partman/confirm_write_new_label boolean true
d-i partman/choose_partition select Finish partitioning and write changes to disk
d-i partman/confirm boolean true

# Timezones
d-i clock-setup/utc boolean true
d-i time/zone string Europe/London

# User accounts
d-i passwd/root-password password ${ROOT_PASSWORD}
d-i passwd/root-password-again password ${ROOT_PASSWORD}
d-i passwd/user-fullname string Pi
d-i passwd/username string pi
d-i passwd/user-password password ${USER_PASSWORD}
d-i passwd/user-password-again password ${USER_PASSWORD}

# Assume only OS (since we wiped the disk earlier)
d-i grub-installer/only_debian boolean true

# Install packages
tasksel tasksel/first multiselect standard, desktop
d-i pkgsel/include string openssh-server build-essential

# Copy over and run a post-installation script, run just before rebooting.
d-i preseed/late_command string \
  cp /cdrom/files/post_install.sh /target/root;\
  in-target /bin/bash /target/root/post_install.sh

# Install and reboot.
d-i finish-install/reboot_in_progress note
EOF