#!/bin/bash

# Print a preseed.cfg, using sensitive secrets imported from preeseed-variables.sh
source preseed-variables.sh

echo "\
##### Language/Regional options #####
AUTO_SETUP_LOCALE=en_GB.UTF-8
AUTO_SETUP_KEYBOARD_LAYOUT=gb
AUTO_SETUP_TIMEZONE=Europe/London

##### Network options #####
AUTO_SETUP_NET_ETHERNET_ENABLED=0
AUTO_SETUP_NET_WIFI_ENABLED=1
AUTO_SETUP_NET_WIFI_COUNTRY_CODE=GB
AUTO_SETUP_NET_HOSTNAME=display-pi

# Enter your static network details below, if applicable.
AUTO_SETUP_NET_USESTATIC=0
AUTO_SETUP_DHCP_TO_STATIC=0

AUTO_SETUP_NET_ETH_FORCE_SPEED=0

# Delay service starts at boot until network is established: 0=disabled | 1=enabled
AUTO_SETUP_BOOT_WAIT_FOR_NETWORK=1

##### Misc options #####
# Swap space size to generate: 0 => disable | 1 => auto | 2 and up => size in MiB
AUTO_SETUP_SWAPFILE_SIZE=1
# Swap space location: "zram" => swap space on /dev/zram0 (auto-size = 50% of RAM size) | /path/to/file => swap file at location (auto-size = 2 GiB minus RAM size)
AUTO_SETUP_SWAPFILE_LOCATION=/var/swap

# Set to "1" to disable HDMI/video output and framebuffers on Raspberry Pi, to reduce power consumption and memory usage: Works on RPi only!
AUTO_SETUP_HEADLESS=0

# Unmask (enable) systemd-logind service (including dbus), which is masked by default on DietPi
AUTO_UNMASK_LOGIND=0

# Custom Script (post-networking and post-DietPi install)
# - Allows you to automatically execute a custom script at the end of DietPi install.
# - Option 0 = Copy your script to /boot/Automation_Custom_Script.sh and it will be executed automatically.
# - Option 1 = Host your script online, then use e.g. AUTO_SETUP_CUSTOM_SCRIPT_EXEC=https://myweb.com/myscript.sh and it will be downloaded and executed automatically.
# - Executed script log: /var/tmp/dietpi/logs/dietpi-automation_custom_script.log
AUTO_SETUP_CUSTOM_SCRIPT_EXEC=0

# Restore a DietPi-Backup on first boot: 0 => disable | 1 => interactive restore (show list of found backups) | 2 => non-interactive restore (restore first found backup)
# - Simply attach the drive/disk/stick which contains the backup. All attached drives will be mounted temporarily and searched automatically.
AUTO_SETUP_BACKUP_RESTORE=0

##### Software options #####
# SSH server choice: 0=none/custom | -1=Dropbear | -2=OpenSSH
AUTO_SETUP_SSH_SERVER_INDEX=-1

# SSH server pubkey
# - Public key(s) for "root" and "dietpi" users, which will be added to ~/.ssh/authorized_keys
# - Use the same setting multiple times for adding multiple keys.
# - See SOFTWARE_DISABLE_SSH_PASSWORD_LOGINS below for disabling SSH password logins.
AUTO_SETUP_SSH_PUBKEY=${SSH_PUBLIC_KEY}

# Logging mode choice: 0=none/custom | -1=RAMlog hourly clear | -2=RAMlog hourly save to disk + clear | -3=Rsyslog + Logrotate
AUTO_SETUP_LOGGING_INDEX=-1
# RAMlog max tmpfs size (MiB). 50 MiB should be fine for single use. 200+ MiB for heavy webserver access log etc.
AUTO_SETUP_RAMLOG_MAXSIZE=50

# Dependency preferences
# - DietPi-Software installs all dependencies for selected software options automatically, which can include a webserver for web applications, a desktop for GUI applications and one usually wants a web browser on desktops.
# - Especially for non-interactive first run installs (see AUTO_SETUP_AUTOMATED below), you may want to define which webserver, desktop and/or browser you want to have installed in such case. For interactive installs you will be always asked to pick one.
# - With below settings you can define your preference for non-interactive installs. However, it will only installed if any other selected software requires it, and an explicit webserver/desktop/browser selection overrides those settings:
# - Webserver preference: 0=Apache | -1=Nginx | -2=Lighttpd
AUTO_SETUP_WEB_SERVER_INDEX=0
# - Desktop preference: 0=LXDE | -1=Xfce | -2=MATE | -3=LXQt | -4=GNUstep
AUTO_SETUP_DESKTOP_INDEX=0
# - Browser preference: 0=None | -1=Firefox | -2=Chromium
AUTO_SETUP_BROWSER_INDEX=-1

# DietPi-Autostart: 0=Console | 7=Console autologin | 1=Kodi | 2=Desktop autologin | 16=Desktop | 4=OpenTyrian | 5=DietPi-CloudShell | 6=Amiberry fast boot | 8=Amiberry standard boot | 9=DDX-Rebirth | 10=CAVA Spectrum | 11=Chromium kiosk | 14=Custom script (background) | 17=Custom script (foreground)
# - This will be effective on 2nd boot, after first run update and installs have been done.
# - Related software titles must be installed either on first run installs or via AUTO_SETUP_AUTOMATED=1 + AUTO_SETUP_INSTALL_SOFTWARE_ID (see below).
AUTO_SETUP_AUTOSTART_TARGET_INDEX=2
# Autologin user name
# - This user must exist before first run installs, otherwise it will be reverted to root.
# - Applies to all autostart options but: 0, 6, 14 and 16
AUTO_SETUP_AUTOSTART_LOGIN_USER=root

##### Non-interactive first run setup #####
# On first login, run update, initial setup and software installs without any user input
# - Setting this to "1" is required for AUTO_SETUP_GLOBAL_PASSWORD and AUTO_SETUP_INSTALL_SOFTWARE_ID.
# - Setting this to "1" indicates that you accept the DietPi GPLv2 license, available at /boot/dietpi-LICENSE.txt, superseding AUTO_SETUP_ACCEPT_LICENSE.
AUTO_SETUP_AUTOMATED=1

# Global password to be applied for the system
# - Requires AUTO_SETUP_AUTOMATED=1
# - Affects "root" and "dietpi" users login passwords and is used by dietpi-software as default for software installs which require a password.
# - During first run setup, the password is removed from this file and instead encrypted and saved to root filesystem.
# - WARN: The default SSH server Dropbear does not support passwords over 100 characters.
# - WARN: We cannot guarantee that all software options can handle special characters like \"$.
AUTO_SETUP_GLOBAL_PASSWORD=${PASSWORD}

# Software to automatically install
# - Requires AUTO_SETUP_AUTOMATED=1
# - List of available software IDs: https://github.com/MichaIng/DietPi/wiki/DietPi-Software-list
# - Add as many entries as you wish, one each line.
# - DietPi will automatically install all dependencies, like ALSA/X11 for desktops etc.
# LXDE Desktop
AUTO_SETUP_INSTALL_SOFTWARE_ID=23
# Git
AUTO_SETUP_INSTALL_SOFTWARE_ID=17
# Dropbear
AUTO_SETUP_INSTALL_SOFTWARE_ID=104
# Neovim
AUTO_SETUP_INSTALL_SOFTWARE_ID=127
# Docker-compose
AUTO_SETUP_INSTALL_SOFTWARE_ID=134
# Docker
AUTO_SETUP_INSTALL_SOFTWARE_ID=162
# Wireguard
AUTO_SETUP_INSTALL_SOFTWARE_ID=172

#------------------------------------------------------------------------------------------------------
##### Misc DietPi program settings #####
#------------------------------------------------------------------------------------------------------
# DietPi-Survey: 1=opt in | 0=opt out | -1=ask on first call
# - https://dietpi.com/docs/dietpi_tools/#miscellaneous (see tab 'DietPi Survey')
SURVEY_OPTED_IN=0

#------------------------------------------------------------------------------------------------------
##### DietPi-Config settings #####
#------------------------------------------------------------------------------------------------------
# CPU Governor: schedutil | ondemand | interactive | conservative | powersave | performance
CONFIG_CPU_GOVERNOR=schedutil
# Ondemand Sampling Rate | Min value: 10000 microseconds (10 ms)
CONFIG_CPU_ONDEMAND_SAMPLE_RATE=25000
# Ondemand Sampling Down Factor: Sampling Rate * Down Factor / 1000 = ms (40 = 1000 ms when sampling rate is 25000)
CONFIG_CPU_ONDEMAND_SAMPLE_DOWNFACTOR=40
# Throttle Up Percentage: Percentage of average CPU usage during sampling rate at which CPU will be throttled up/down
CONFIG_CPU_USAGE_THROTTLE_UP=50

# CPU Frequency Limits: Disabled=disabled
# - Intel CPUs use a percentage value (%) from 0-100, e.g.: 55
# - All other devices must use a specific MHz value, e.g.: 1600
# - Has no effect on RPi, please set "arm_freq" and "arm_freq_min" in config.txt instead.
CONFIG_CPU_MAX_FREQ=Disabled
CONFIG_CPU_MIN_FREQ=Disabled

# Disable Intel-based turbo/boost stepping. This flag should not be required, setting <100% MAX frequency should disable Turbo on Intel CPUs.
CONFIG_CPU_DISABLE_TURBO=0

#GPU Driver | Will also be applied during 1st run if set to a value other than 'None'
#   NB: x86_64 PC only!
#   Adds support for GUI/video hardware acceleration, OpenGL/GLES, Vulkan and VA-API
# - none | Default, No GPU
# - intel
# - nvidia
# - amd
# - custom | Manual driver install (DietPi will not make driver changes to your system)
CONFIG_GPU_DRIVER=none

# System-wide proxy settings
# - Do not modify, you must use dietpi-config > "Network Options: Adapters" to apply
CONFIG_PROXY_ADDRESS=MyProxyServer.com
CONFIG_PROXY_PORT=8080
CONFIG_PROXY_USERNAME=
CONFIG_PROXY_PASSWORD=

# Connection timeout in seconds for G_CHECK_NET and G_CHECK_URL. Increase if you have a "flaky" connection or slow DNS resolver.
# - Set this to "0" to allow unlimited time, however this is not recommended to avoid unlimited hanging background scripts, e.g. daily DietPi update check.
# - A negative or non-integer value will result in the default of 10 seconds.
CONFIG_G_CHECK_URL_TIMEOUT=10
# Connection attempts with above timeout each, before G_CHECK_NET and G_CHECK_URL give up and prompt an error.
# - Any value below "1" or a non-integer value will result in the default of 2 attempts.
CONFIG_G_CHECK_URL_ATTEMPTS=2
# General connection and DNS testing
# - IPv4 address to ping when checking network connectivity. Default: 9.9.9.9 (Quad9 DNS IP)
CONFIG_CHECK_CONNECTION_IP=9.9.9.9
# - IPv6 address to ping when checking network connectivity. Default: 2620:fe::fe (Quad9 DNS IP)
CONFIG_CHECK_CONNECTION_IPV6=2620:fe::fe
# - Domain to resolve when checking DNS resolver. Default: dns9.quad9.net (Quad9 DNS domain)
CONFIG_CHECK_DNS_DOMAIN=dns9.quad9.net

# Daily check for DietPi updates: 0=disable | 1=enable
# - Checks are done by downloading a file of only 7 bytes.
CONFIG_CHECK_DIETPI_UPDATES=1

# Daily check for APT package updates: 0=disable | 1=check only | 2=check and upgrade automatically
# - Upgrade logs can be found at: /var/tmp/dietpi/logs/dietpi-update_apt.log
CONFIG_CHECK_APT_UPDATES=2

# Network time sync: 0=disabled | 1=boot only | 2=boot + daily | 3=boot + hourly | 4=Daemon + Drift
CONFIG_NTP_MODE=2

# Serial Console: Set to 0 if you do not require serial console.
CONFIG_SERIAL_CONSOLE_ENABLE=1

# Sound card
CONFIG_SOUNDCARD=none

# LCD Panel addon
# - Do not modify, you must use dietpi-config to configure/set options
CONFIG_LCDPANEL=none

# IPv6
CONFIG_ENABLE_IPV6=0

# APT mirrors which are applied to /etc/apt/sources.list | Values here will also be applied during 1st run setup
# - Raspbian: https://www.raspbian.org/RaspbianMirrors
CONFIG_APT_RASPBIAN_MIRROR=http://raspbian.raspberrypi.org/raspbian/
# - Debian: https://www.debian.org/mirror/official#list
CONFIG_APT_DEBIAN_MIRROR=https://deb.debian.org/debian/

# NTP mirror, applied to /etc/ntp.conf
# - For a full list, please see: https://www.ntppool.org/zone/@
# - Please remove the initial integer and full stop from the value (removing "0."), eg: debian.pool.ntp.org
CONFIG_NTP_MIRROR=debian.pool.ntp.org

#------------------------------------------------------------------------------------------------------
##### DietPi-Software settings #####
#------------------------------------------------------------------------------------------------------
# SSH Server
# - Disable SSH password logins, e.g. when using pubkey authentication
#	0=Allow password logins for all users, including root
#	root=Disable password login for root user only
#	1=Disable password logins for all users, assure that you have a valid SSH key applied!
SOFTWARE_DISABLE_SSH_PASSWORD_LOGINS=1

# X.org
# - DPI 96(default) 120(+25%) 144(+50%) 168(+75%) 192(+100%)
SOFTWARE_XORG_DPI=96

#------------------------------------------------------------------------------------------------------
##### Settings, automatically added by dietpi-update #####
#------------------------------------------------------------------------------------------------------
"