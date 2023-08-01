#!/bin/bash

# Set display resolution to 800x480 for next boot. This maybe requires 2 boots to take effect, not sure?
sed -i 's/#hdmi_group=.*/hdmi_group=2/' /boot/config.txt
sed -i 's/#hdmi_mode=.*/hdmi_mode=87/' /boot/config.txt
echo "hdmi_cvt=800 480 60 6 0 0 0" >> /boot/config.txt
sed -i 's/#hdmi_drive=.*/hdmi_drive=1/' /boot/config.txt

# Configure background
# This config file doesn't exist until LXDE runs for the first time, which is on first boot not post-install.
# Create the file with some reasonable values.
mkdir -p /root/.config/pcmanfm/LXDE
cat <<EOF > /root/.config/pcmanfm/LXDE/desktop-items-0.conf 
[*]
wallpaper_mode=stretch
wallpaper_common=1
wallpaper=/files/background.png
desktop_bg=#000000
desktop_fg=#ffffff
desktop_shadow=#333333
desktop_font=Sans 12
show_wm_menu=0
sort=mtime;ascending;
show_documents=0
show_trash=0
show_mounts=0
EOF

# Install resolvconf so we can resolve DNS names (required for installing wireguard)
apt-get install resolvconf
# Actually update resolv.conf so we can query DNS
resolvconf -u
# Install wireguard. Installing from the dietpi repository fails because it requires user input (for no good reason).
apt-get install wireguard-tools iptables qrencode

# Enable wireguard on boot, but don't worry about starting it now - post-install means about to reboot anyway.
# Wireguard will fail to start with `RTNETLINK answers: File exists` if an identical subnet is already provided by
# e.g. the current wifi network, but it'll work when rebooted on another network.
systemctl enable /files/wireguard.service

# TODO: Configure systemctl.
