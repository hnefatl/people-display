#!/bin/bash

# Set display resolution for next boot.
sed -i 's/framebuffer_width=.*/framebuffer_width=800/' /boot/config.txt
sed -i 's/framebuffer_height=.*/framebuffer_height=480/' /boot/config.txt
# Force HDMI to 800x480 30fps with 15:9 aspect ratio, since there's some flicker at 60fps.
# Doesn't work correctly, maybe a clock/timing issue? Need to reread display specs.
#echo "hdmi_cvt=800 480 30 6" >> /boot/config.txt

# Configure background and remove desktop icon clutter
# These need to be done post-install rather than during the initial copy, because LXDE hasn't been installed at that point.
sed -i 's|wallpaper=.*|wallpaper=/files/background.png|g' /root/.config/pcmanfm/LXDE/desktop-items-0.conf
rm "$root_dir/root/Desktop/*"

# Enable wireguard on boot, but don't worry about starting it now - post-install means about to reboot anyway.
systemctl enable wg-quick@wg0

# TODO: Configure systemctl.
