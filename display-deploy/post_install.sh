#!/bin/bash

# Assume environment variables from preseed-variables.sh are present.
# TODO: figure out how to copy this file over with the preseed config.

# Install the destination location's WiFi information.
/usr/bin/wpa_passphrase "${DEST_WIFI_SSID}" "${DEST_WIFI_PSK}" >> /etc/wpa_supplicant.conf

# Install docker (many steps...)
apt-get update
apt-get install ca-certificates curl gnupg

install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/debian/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg
chmod a+r /etc/apt/keyrings/docker.gpg

echo \
  "deb [arch="$(dpkg --print-architecture)" signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/debian \
  "$(. /etc/os-release && echo "$VERSION_CODENAME")" stable" > \
  /etc/apt/sources.list.d/docker.list

sudo apt-get update
sudo apt-get install docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin

# Copy over docker-compose file
# TODO

# Configure systemctl services
# TODO