#!/bin/bash

source preeseed-variables.sh

echo <<EOF
# Entry 0
# - WiFi SSID: required, case sensitive
aWIFI_SSID[0]='${INIT_WIFI_SSID}'
# - WiFi key: If no key/open, leave this blank
# - In case of WPA-PSK, alternatively enter the 64-digit hexadecimal key returned by wpa_passphrase
# - Please replace single quote characters ' in your key with '\''. No other escaping is required.
aWIFI_KEY[0]='${INIT_WIFI_PSK}'
# - Key type: NONE (no key/open) | WPA-PSK | WEP | WPA-EAP (then use settings below)
aWIFI_KEYMGR[0]='WPA-PSK'
# - WPA-EAP options: Only fill if WPA-EAP is set above
aWIFI_PROTO[0]=''
aWIFI_PAIRWISE[0]=''
aWIFI_AUTH_ALG[0]=''
aWIFI_EAP[0]=''
aWIFI_IDENTITY[0]=''
aWIFI_PASSWORD[0]=''
aWIFI_PHASE1[0]=''
aWIFI_PHASE2[0]=''
# - Path to the certificate file, e.g.: /boot/mycert.cer
aWIFI_CERT[0]=''

# Entry 1
# - WiFi SSID: required, case sensitive
aWIFI_SSID[1]='${DEST_WIFI_SSID}'
# - WiFi key: If no key/open, leave this blank
# - In case of WPA-PSK, alternatively enter the 64-digit hexadecimal key returned by wpa_passphrase
# - Please replace single quote characters ' in your key with '\''. No other escaping is required.
aWIFI_KEY[1]='${DEST_WIFI_PSK}'
# - Key type: NONE (no key/open) | WPA-PSK | WEP | WPA-EAP (then use settings below)
aWIFI_KEYMGR[1]='WPA-PSK'
# - WPA-EAP options: Only fill if WPA-EAP is set above
aWIFI_PROTO[1]=''
aWIFI_PAIRWISE[1]=''
aWIFI_AUTH_ALG[1]=''
aWIFI_EAP[1]=''
aWIFI_IDENTITY[1]=''
aWIFI_PASSWORD[1]=''
aWIFI_PHASE1[1]=''
aWIFI_PHASE2[1]=''
# - Path to the certificate file, e.g.: /boot/mycert.cer
aWIFI_CERT[1]=''
EOF