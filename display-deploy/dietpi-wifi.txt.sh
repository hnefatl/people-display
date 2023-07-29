#!/bin/bash

source preseed-variables.sh

echo "\
# Entry 0
# - WiFi SSID: required, case sensitive
aWIFI_SSID[0]='${INIT_WIFI_SSID}'
# - WiFi key: If no key/open, leave this blank
# - In case of WPA-PSK, alternatively enter the 64-digit hexadecimal key returned by wpa_passphrase
# - Please replace single quote characters ' in your key with '\''. No other escaping is required.
aWIFI_KEY[0]='${INIT_WIFI_PSK}'
# - Key type: NONE (no key/open) | WPA-PSK | WEP | WPA-EAP (then use settings below)
aWIFI_KEYMGR[0]='WPA-PSK'

# Entry 1
# - WiFi SSID: required, case sensitive
aWIFI_SSID[1]='${DEST_WIFI_SSID}'
# - WiFi key: If no key/open, leave this blank
# - In case of WPA-PSK, alternatively enter the 64-digit hexadecimal key returned by wpa_passphrase
# - Please replace single quote characters ' in your key with '\''. No other escaping is required.
aWIFI_KEY[1]='${DEST_WIFI_PSK}'
# - Key type: NONE (no key/open) | WPA-PSK | WEP | WPA-EAP (then use settings below)
aWIFI_KEYMGR[1]='WPA-PSK'
"