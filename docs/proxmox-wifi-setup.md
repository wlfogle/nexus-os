To connect Proxmox to Wi-Fi via the command line, you need to install `wpasupplicant`, create a configuration file with your Wi-Fi credentials, and edit the `/etc/network/interfaces` file to set up the Wi-Fi interface. After that, restart the networking service to apply the changes.

Connecting Proxmox to Wi-Fi via Command Line
To connect your Proxmox server to a Wi-Fi network using the command line, follow these steps:

Prerequisites
Ensure you have a wired Ethernet connection temporarily to install necessary packages.
Identify your Wi-Fi interface name (usually wlan0, wlp2s0, etc.) using the command:

ip a

Step 1: Install Required Packages
Connect to your Proxmox server via SSH or directly.
Update your package list and install wpasupplicant:

apt update
apt install wpasupplicant

Step 2: Configure Wi-Fi Settings
Create a configuration file for wpa_supplicant:

wpa_passphrase [Your_SSID] [Your_Password] > /etc/wpa_supplicant/wpa_supplicant.conf

Edit the /etc/network/interfaces file to include your Wi-Fi configuration. Open the file with a text editor:

nano /etc/network/interfaces

Add the following lines, replacing [wifi_interface] with your actual Wi-Fi interface name:

auto [wifi_interface]
iface [wifi_interface] inet dhcp
wpa-conf /etc/wpa_supplicant/wpa_supplicant.conf

Step 3: Enable IP Forwarding and NAT
To allow VMs to access the internet, enable IP forwarding by adding the following lines to the same file:

post-up echo 1 > /proc/sys/net/ipv4/ip_forward
post-up iptables -t nat -A POSTROUTING -o [wifi_interface] -j MASQUERADE

Step 4: Restart Networking
Restart the networking service to apply changes:

systemctl restart networking

Verify the connection by pinging an external site:

ping google.com

Important Notes
Proxmox does not officially support Wi-Fi configurations, and using Wi-Fi may lead to connectivity issues, especially for VMs.
For better performance and reliability, a wired connection is recommended whenever possible.
