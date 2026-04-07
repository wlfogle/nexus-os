[ Lynis 3.1.4 ]

################################################################################
  Lynis comes with ABSOLUTELY NO WARRANTY. This is free software, and you are
  welcome to redistribute it under the terms of the GNU General Public License.
  See the LICENSE file for details about using this software.

  2007-2024, CISOfy - https://cisofy.com/lynis/
  Enterprise support available (compliance, plugins, interface and tools)
################################################################################


[+] Initializing program
------------------------------------
  - Detecting OS...                                           [ DONE ]
  - Checking profiles...                                      [ DONE ]

  ---------------------------------------------------
  Program version:           3.1.4
  Operating system:          Linux
  Operating system name:     Debian
  Operating system version:  13
  Kernel version:            6.17.13
  Hardware platform:         x86_64
  Hostname:                  tiamat
  ---------------------------------------------------
  Profiles:                  /etc/lynis/default.prf
  Log file:                  /var/log/lynis.log
  Report file:               /var/log/lynis-report.dat
  Report version:            1.0
  Plugin directory:          /etc/lynis/plugins
  ---------------------------------------------------
  Auditor:                   [Not Specified]
  Language:                  en
  Test category:             all
  Test group:                all
  ---------------------------------------------------
  - Program update status...                                  [ NO UPDATE ]

[+] System tools
------------------------------------
  - Scanning available tools...
  - Checking system binaries...

[+] Plugins (phase 1)
------------------------------------
 Note: plugins have more extensive tests and may take several minutes to complete
  
  - Plugin: debian
    [
[+] Debian Tests
------------------------------------
  - Checking for system binaries that are required by Debian Tests...
    - Checking /bin...                                        [ FOUND ]
    - Checking /sbin...                                       [ FOUND ]
    - Checking /usr/bin...                                    [ FOUND ]
    - Checking /usr/sbin...                                   [ FOUND ]
    - Checking /usr/local/bin...                              [ FOUND ]
    - Checking /usr/local/sbin...                             [ FOUND ]
  - Authentication:
    - PAM (Pluggable Authentication Modules):
      - libpam-tmpdir                                         [ Not Installed ]
  - File System Checks:
    - DM-Crypt, Cryptsetup & Cryptmount:

  [WARNING]: Test DEB-0280 had a long execution: 17.365358 seconds

      - Checking / on /dev/sda3                               [ NOT ENCRYPTED ]
      - Checking /mnt/hdd on --- Logical volume --- LV Path /dev/pve/media-hdd LV Name media-hdd VG Name pve LV UUID nKVpET-UssX-tMov-qYcf-RGoE-3uNX-XgNlax LV Write Access read/write LV Creation host, time tiamat, 2026-03-14 10:05:51 -0900 LV Pool name data LV Status available # open 1 LV Size 1.46 TiB Mapped size 1.63% Current LE 384000 Segments 1 Allocation inherit Read ahead sectors auto - currently set to 4096 Block device 252:6 --- Segments --- Virtual extents 0 to 383999: Type thin Device ID 5  [ NOT ENCRYPTED ]
      - Checking /media/loufogle/73cf9511-0af0-4ac4-9d83-ee21eb17ff5d/models:/mnt/laptop-models on 192.168.12.172:/media/loufogle/73cf9511-0af0-4ac4-9d83-ee21eb17ff5d/models  [ NOT ENCRYPTED ]
      - Checking /tmp/.mount_ProxMeSzYNGO on ProxMenux-Monitor.AppImage  [ NOT ENCRYPTED ]
      - Checking /etc/pve on /dev/fuse                        [ NOT ENCRYPTED ]
  - Software:
    - apt-listbugs                                            [ Not Installed ]
    - apt-listchanges                                         [ Installed and enabled for apt ]
    - needrestart                                             [ Not Installed ]
    - fail2ban                                                [ Installed with jail.conf ]
]

[+] Boot and services
------------------------------------
  - Service Manager                                           [ systemd ]
  - Checking UEFI boot                                        [ ENABLED ]
  - Checking Secure Boot                                      [ DISABLED ]
  - Checking presence GRUB2                                   [ FOUND ]
    - Checking for password protection                        [ NONE ]
  - Check running services (systemctl)                        [ DONE ]
        Result: found 55 running services
  - Check enabled services at boot (systemctl)                [ DONE ]
        Result: found 64 enabled services
  - Check startup files (permissions)                         [ OK ]
  - Running 'systemd-analyze security'
      Unit name (exposure value) and predicate
      --------------------------------
    - chrony.service (value=3.5)                              [ PROTECTED ]
    - console-getty.service (value=9.6)                       [ UNSAFE ]
    - corosync.service (value=9.2)                            [ UNSAFE ]
    - cpufrequtils.service (value=9.6)                        [ UNSAFE ]
    - cron.service (value=9.6)                                [ UNSAFE ]
    - dbus.service (value=9.3)                                [ UNSAFE ]
    - dm-event.service (value=9.5)                            [ UNSAFE ]
    - dnsmasq.service (value=9.6)                             [ UNSAFE ]
    - emergency.service (value=9.5)                           [ UNSAFE ]
    - fail2ban.service (value=9.6)                            [ UNSAFE ]
    - frr.service (value=9.8)                                 [ UNSAFE ]
    - getty@tty1.service (value=9.6)                          [ UNSAFE ]
    - haveged.service (value=3.2)                             [ PROTECTED ]
    - iscsid.service (value=9.5)                              [ UNSAFE ]
    - ksmtuned.service (value=9.6)                            [ UNSAFE ]
    - lightdm.service (value=9.6)                             [ UNSAFE ]
    - loadcpufreq.service (value=9.6)                         [ UNSAFE ]
    - lvm2-lvmpolld.service (value=9.5)                       [ UNSAFE ]
    - lxc-monitord.service (value=9.6)                        [ UNSAFE ]
    - lxcfs.service (value=9.6)                               [ UNSAFE ]
    - lynis.service (value=9.6)                               [ UNSAFE ]
    - mdmonitor-oneshot.service (value=9.6)                   [ UNSAFE ]
    - netavark-dhcp-proxy.service (value=9.6)                 [ UNSAFE ]
    - nfs-blkmap.service (value=9.5)                          [ UNSAFE ]
    - novnc.service (value=9.6)                               [ UNSAFE ]
    - polkit.service (value=1.2)                              [ PROTECTED ]
    - postfix.service (value=3.9)                             [ PROTECTED ]
    - postfix@-.service (value=3.9)                           [ PROTECTED ]
    - proxmenux-monitor.service (value=9.6)                   [ UNSAFE ]
    - proxmox-firewall.service (value=9.6)                    [ UNSAFE ]
    - pve-cluster.service (value=9.5)                         [ UNSAFE ]
    - pve-container@100.service (value=9.6)                   [ UNSAFE ]
    - pve-container@101.service (value=9.6)                   [ UNSAFE ]
    - pve-container@102.service (value=9.6)                   [ UNSAFE ]
    - pve-container@103.service (value=9.6)                   [ UNSAFE ]
    - pve-container@104.service (value=9.6)                   [ UNSAFE ]
    - pve-container@105.service (value=9.6)                   [ UNSAFE ]
    - pve-container@106.service (value=9.6)                   [ UNSAFE ]
    - pve-container@107.service (value=9.6)                   [ UNSAFE ]
    - pve-container@210.service (value=9.6)                   [ UNSAFE ]
    - pve-container@212.service (value=9.6)                   [ UNSAFE ]
    - pve-container@214.service (value=9.6)                   [ UNSAFE ]
    - pve-container@215.service (value=9.6)                   [ UNSAFE ]
    - pve-container@230.service (value=9.6)                   [ UNSAFE ]
    - pve-container@231.service (value=9.6)                   [ UNSAFE ]
    - pve-container@240.service (value=9.6)                   [ UNSAFE ]
    - pve-container@242.service (value=9.6)                   [ UNSAFE ]
    - pve-container@900.service (value=9.6)                   [ UNSAFE ]
    - pve-firewall.service (value=9.5)                        [ UNSAFE ]
    - pve-ha-crm.service (value=9.6)                          [ UNSAFE ]
    - pve-ha-lrm.service (value=9.6)                          [ UNSAFE ]
    - pve-lxc-syscalld.service (value=9.6)                    [ UNSAFE ]
    - pvedaemon.service (value=9.6)                           [ UNSAFE ]
    - pvefw-logger.service (value=9.5)                        [ UNSAFE ]
    - pveproxy.service (value=9.6)                            [ UNSAFE ]
    - pvescheduler.service (value=9.6)                        [ UNSAFE ]
    - pvestatd.service (value=9.6)                            [ UNSAFE ]
    - qmeventd.service (value=9.6)                            [ UNSAFE ]
    - rc-local.service (value=9.6)                            [ UNSAFE ]
    - rescue.service (value=9.5)                              [ UNSAFE ]
    - rpc-gssd.service (value=9.5)                            [ UNSAFE ]
    - rpc-statd-notify.service (value=9.5)                    [ UNSAFE ]
    - rpc-svcgssd.service (value=9.5)                         [ UNSAFE ]
    - rrdcached.service (value=9.6)                           [ UNSAFE ]
    - smartmontools.service (value=9.6)                       [ UNSAFE ]
    - spiceproxy.service (value=9.6)                          [ UNSAFE ]
    - ssh.service (value=9.6)                                 [ UNSAFE ]
    - sshd@sshd-keygen.service (value=9.6)                    [ UNSAFE ]
    - systemd-ask-password-console.service (value=9.4)        [ UNSAFE ]
    - systemd-ask-password-wall.service (value=9.4)           [ UNSAFE ]
    - systemd-bsod.service (value=9.5)                        [ UNSAFE ]
    - systemd-hostnamed.service (value=1.7)                   [ PROTECTED ]
    - systemd-initctl.service (value=9.4)                     [ UNSAFE ]
    - systemd-journald.service (value=4.9)                    [ PROTECTED ]
    - systemd-logind.service (value=2.8)                      [ PROTECTED ]
    - systemd-networkd.service (value=2.9)                    [ PROTECTED ]
    - systemd-rfkill.service (value=9.4)                      [ UNSAFE ]
    - systemd-udevd.service (value=7.1)                       [ MEDIUM ]
    - tigervncserver@:1.service (value=9.6)                   [ UNSAFE ]
    - udisks2.service (value=9.6)                             [ UNSAFE ]
    - user@0.service (value=9.8)                              [ UNSAFE ]
    - uuidd.service (value=5.8)                               [ MEDIUM ]
    - watchdog-mux.service (value=9.6)                        [ UNSAFE ]
    - wpa_supplicant.service (value=9.6)                      [ UNSAFE ]
    - xrdp-sesman.service (value=9.6)                         [ UNSAFE ]
    - xrdp.service (value=8.0)                                [ EXPOSED ]
    - zfs-zed.service (value=9.6)                             [ UNSAFE ]

[+] Kernel
------------------------------------
  - Checking default runlevel                                 [ runlevel 5 ]
  - Checking CPU support (NX/PAE)
    CPU support: PAE and/or NoeXecute supported               [ FOUND ]
  - Checking kernel version and release                       [ DONE ]
  - Checking kernel type                                      [ DONE ]
  - Checking loaded kernel modules                            [ DONE ]
      Found 146 active modules
  - Checking Linux kernel configuration file                  [ FOUND ]
  - Checking default I/O kernel scheduler                     [ NOT FOUND ]
/usr/bin/grep: /etc/kernel-img.conf: No such file or directory
  - Checking core dumps configuration
    - configuration in systemd conf files                     [ DEFAULT ]
    - configuration in /etc/profile                           [ DEFAULT ]
    - 'hard' configuration in /etc/security/limits.conf       [ ENABLED ]
    - 'soft' configuration in /etc/security/limits.conf       [ DISABLED ]
    - Checking setuid core dumps configuration                [ DISABLED ]
  - Check if reboot is needed                                 [ NO ]

[+] Memory and Processes
------------------------------------
  - Checking /proc/meminfo                                    [ FOUND ]
  - Searching for dead/zombie processes                       [ NOT FOUND ]
  - Searching for IO waiting processes                        [ NOT FOUND ]
  - Search prelink tooling                                    [ NOT FOUND ]

[+] Users, Groups and Authentication
------------------------------------
  - Administrator accounts                                    [ OK ]
  - Unique UIDs                                               [ OK ]
  - Consistency of group files (grpck)                        [ OK ]
  - Unique group IDs                                          [ OK ]
  - Unique group names                                        [ OK ]
  - Password file consistency                                 [ OK ]
  - Password hashing methods                                  [ OK ]
  - Checking password hashing rounds                          [ DISABLED ]
  - Query system users (non daemons)                          [ DONE ]
  - NIS+ authentication support                               [ NOT ENABLED ]
  - NIS authentication support                                [ NOT ENABLED ]
  - Sudoers file(s)                                           [ FOUND ]
    - Permissions for directory: /etc/sudoers.d               [ WARNING ]
    - Permissions for: /etc/sudoers                           [ OK ]
    - Permissions for: /etc/sudoers.d/zfs                     [ OK ]
    - Permissions for: /etc/sudoers.d/README                  [ OK ]
  - PAM password strength tools                               [ SUGGESTION ]
  - PAM configuration files (pam.conf)                        [ FOUND ]
  - PAM configuration files (pam.d)                           [ FOUND ]
  - PAM modules                                               [ FOUND ]
  - LDAP module in PAM                                        [ NOT FOUND ]
  - Accounts without expire date                              [ SUGGESTION ]
  - Accounts without password                                 [ OK ]
  - Locked accounts                                           [ FOUND ]
  - Checking user password aging (minimum)                    [ DISABLED ]
  - User password aging (maximum)                             [ DISABLED ]
  - Checking expired passwords                                [ OK ]
  - Checking Linux single user mode authentication            [ OK ]
  - Determining default umask
    - umask (/etc/profile)                                    [ NOT FOUND ]
    - umask (/etc/login.defs)                                 [ SUGGESTION ]
  - LDAP authentication support                               [ NOT ENABLED ]
  - Logging failed login attempts                             [ DISABLED ]

[+] Kerberos
------------------------------------
  - Check for Kerberos KDC and principals                     [ NOT FOUND ]

[+] Shells
------------------------------------
  - Checking shells from /etc/shells
    Result: found 8 shells (valid shells: 8).
    - Session timeout settings/tools                          [ NONE ]
  - Checking default umask values
    - Checking default umask in /etc/bash.bashrc              [ NONE ]
    - Checking default umask in /etc/profile                  [ NONE ]

[+] File systems
------------------------------------
  - Checking mount points
    - Checking /home mount point                              [ SUGGESTION ]
    - Checking /tmp mount point                               [ OK ]
    - Checking /var mount point                               [ SUGGESTION ]
  - Checking LVM volume groups                                [ FOUND ]
    - Checking LVM volumes                                    [ FOUND ]
  - Query swap partitions (fstab)                             [ OK ]
  - Testing swap partitions                                   [ OK ]
  - Testing /proc mount (hidepid)                             [ SUGGESTION ]
  - Checking for old files in /tmp                            [ OK ]
  - Checking /tmp sticky bit                                  [ OK ]
  - Checking /var/tmp sticky bit                              [ OK ]
  - ACL support root file system                              [ ENABLED ]
  - Mount options of /                                        [ NON DEFAULT ]
  - Mount options of /dev                                     [ PARTIALLY HARDENED ]
  - Mount options of /dev/shm                                 [ PARTIALLY HARDENED ]
  - Mount options of /run                                     [ HARDENED ]
  - Mount options of /tmp                                     [ PARTIALLY HARDENED ]
  - Total without nodev:8 noexec:14 nosuid:6 ro or noexec (W^X): 12 of total 31
  - Checking Locate database                                  [ FOUND ]
  - Disable kernel support of some filesystems

[+] USB Devices
------------------------------------
  - Checking usb-storage driver (modprobe config)             [ NOT DISABLED ]
  - Checking USB devices authorization                        [ ENABLED ]
  - Checking USBGuard                                         [ NOT FOUND ]

[+] Storage
------------------------------------
  - Checking firewire ohci driver (modprobe config)           [ NOT DISABLED ]

[+] NFS
------------------------------------
  - Query rpc registered programs                             [ DONE ]
  - Query NFS versions                                        [ DONE ]
  - Query NFS protocols                                       [ DONE ]
  - Check running NFS daemon                                  [ NOT FOUND ]

[+] Name services
------------------------------------
  - Searching DNS domain name                                 [ FOUND ]
      Domain name: local
  - Checking /etc/hosts
    - Duplicate entries in hosts file                         [ NONE ]
    - Presence of configured hostname in /etc/hosts           [ FOUND ]
    - Hostname mapped to localhost                            [ NOT FOUND ]
    - Localhost mapping to IP address                         [ OK ]

[+] Ports and packages
------------------------------------
  - Searching package managers
    - Searching dpkg package manager                          [ FOUND ]
      - Querying package manager
    - Query unpurged packages                                 [ FOUND ]
  - Checking security repository in sources.list file         [ OK ]
  - Checking security repository in sources.list.d directory  [ OK ]
  - Checking APT package database                             [ OK ]
W: Target Packages (main/binary-amd64/Packages) is configured multiple times in /etc/apt/sources.list.d/debian.sources:2 and /etc/apt/sources.list.d/pecu-repos.list:6
W: Target Packages (main/binary-all/Packages) is configured multiple times in /etc/apt/sources.list.d/debian.sources:2 and /etc/apt/sources.list.d/pecu-repos.list:6
W: Target Packages (contrib/binary-amd64/Packages) is configured multiple times in /etc/apt/sources.list.d/debian.sources:2 and /etc/apt/sources.list.d/pecu-repos.list:6
W: Target Packages (contrib/binary-all/Packages) is configured multiple times in /etc/apt/sources.list.d/debian.sources:2 and /etc/apt/sources.list.d/pecu-repos.list:6
W: Target Packages (non-free-firmware/binary-amd64/Packages) is configured multiple times in /etc/apt/sources.list.d/debian.sources:2 and /etc/apt/sources.list.d/pecu-repos.list:6
W: Target Packages (non-free-firmware/binary-all/Packages) is configured multiple times in /etc/apt/sources.list.d/debian.sources:2 and /etc/apt/sources.list.d/pecu-repos.list:6
W: Target Packages (pve-no-subscription/binary-amd64/Packages) is configured multiple times in /etc/apt/sources.list.d/pecu-repos.list:7 and /etc/apt/sources.list.d/proxmox.sources:1
W: Target Packages (pve-no-subscription/binary-all/Packages) is configured multiple times in /etc/apt/sources.list.d/pecu-repos.list:7 and /etc/apt/sources.list.d/proxmox.sources:1
W: Target Packages (pve-no-subscription/binary-amd64/Packages) is configured multiple times in /etc/apt/sources.list.d/pecu-repos.list:7 and /etc/apt/sources.list.d/pve-public-repo.list:1
W: Target Packages (pve-no-subscription/binary-all/Packages) is configured multiple times in /etc/apt/sources.list.d/pecu-repos.list:7 and /etc/apt/sources.list.d/pve-public-repo.list:1
E: The repository 'https://security.debian.org/debian-security trixie/updates Release' does not have a Release file.
W: Target Packages (main/binary-amd64/Packages) is configured multiple times in /etc/apt/sources.list.d/debian.sources:1 and /etc/apt/sources.list.d/pecu-repos.list:5
W: Target Packages (main/binary-all/Packages) is configured multiple times in /etc/apt/sources.list.d/debian.sources:1 and /etc/apt/sources.list.d/pecu-repos.list:5
W: Target Packages (contrib/binary-amd64/Packages) is configured multiple times in /etc/apt/sources.list.d/debian.sources:1 and /etc/apt/sources.list.d/pecu-repos.list:5
W: Target Packages (contrib/binary-all/Packages) is configured multiple times in /etc/apt/sources.list.d/debian.sources:1 and /etc/apt/sources.list.d/pecu-repos.list:5
W: Target Packages (non-free-firmware/binary-amd64/Packages) is configured multiple times in /etc/apt/sources.list.d/debian.sources:1 and /etc/apt/sources.list.d/pecu-repos.list:5
W: Target Packages (non-free-firmware/binary-all/Packages) is configured multiple times in /etc/apt/sources.list.d/debian.sources:1 and /etc/apt/sources.list.d/pecu-repos.list:5
W: OpenPGP signature verification failed: https://packages.cisofy.com/community/lynis/deb stable InRelease: Sub-process /usr/bin/sqv returned an error code (1), error message is: Missing key 3E82FB7C68F57341349ED2C17A9A1D9D5B27C6D3, which is needed to verify signature.
E: The repository 'https://packages.cisofy.com/community/lynis/deb stable InRelease' is not signed.
W: Target Packages (main/binary-amd64/Packages) is configured multiple times in /etc/apt/sources.list.d/debian.sources:1 and /etc/apt/sources.list.d/pecu-repos.list:5
W: Target Packages (main/binary-all/Packages) is configured multiple times in /etc/apt/sources.list.d/debian.sources:1 and /etc/apt/sources.list.d/pecu-repos.list:5
W: Target Packages (contrib/binary-amd64/Packages) is configured multiple times in /etc/apt/sources.list.d/debian.sources:1 and /etc/apt/sources.list.d/pecu-repos.list:5
W: Target Packages (contrib/binary-all/Packages) is configured multiple times in /etc/apt/sources.list.d/debian.sources:1 and /etc/apt/sources.list.d/pecu-repos.list:5
W: Target Packages (non-free-firmware/binary-amd64/Packages) is configured multiple times in /etc/apt/sources.list.d/debian.sources:1 and /etc/apt/sources.list.d/pecu-repos.list:5
W: Target Packages (non-free-firmware/binary-all/Packages) is configured multiple times in /etc/apt/sources.list.d/debian.sources:1 and /etc/apt/sources.list.d/pecu-repos.list:5
W: Target Packages (main/binary-amd64/Packages) is configured multiple times in /etc/apt/sources.list.d/debian.sources:2 and /etc/apt/sources.list.d/pecu-repos.list:6
W: Target Packages (main/binary-all/Packages) is configured multiple times in /etc/apt/sources.list.d/debian.sources:2 and /etc/apt/sources.list.d/pecu-repos.list:6
W: Target Packages (contrib/binary-amd64/Packages) is configured multiple times in /etc/apt/sources.list.d/debian.sources:2 and /etc/apt/sources.list.d/pecu-repos.list:6
W: Target Packages (contrib/binary-all/Packages) is configured multiple times in /etc/apt/sources.list.d/debian.sources:2 and /etc/apt/sources.list.d/pecu-repos.list:6
W: Target Packages (non-free-firmware/binary-amd64/Packages) is configured multiple times in /etc/apt/sources.list.d/debian.sources:2 and /etc/apt/sources.list.d/pecu-repos.list:6
W: Target Packages (non-free-firmware/binary-all/Packages) is configured multiple times in /etc/apt/sources.list.d/debian.sources:2 and /etc/apt/sources.list.d/pecu-repos.list:6
W: Target Packages (pve-no-subscription/binary-amd64/Packages) is configured multiple times in /etc/apt/sources.list.d/pecu-repos.list:7 and /etc/apt/sources.list.d/proxmox.sources:1
W: Target Packages (pve-no-subscription/binary-all/Packages) is configured multiple times in /etc/apt/sources.list.d/pecu-repos.list:7 and /etc/apt/sources.list.d/proxmox.sources:1
W: Target Packages (pve-no-subscription/binary-amd64/Packages) is configured multiple times in /etc/apt/sources.list.d/pecu-repos.list:7 and /etc/apt/sources.list.d/pve-public-repo.list:1
W: Target Packages (pve-no-subscription/binary-all/Packages) is configured multiple times in /etc/apt/sources.list.d/pecu-repos.list:7 and /etc/apt/sources.list.d/pve-public-repo.list:1
  - Checking vulnerable packages (apt-get only)               [ DONE ]
  - Checking upgradeable packages                             [ SKIPPED ]
  - Checking package audit tool                               [ INSTALLED ]
    Found: apt-get
  - Toolkit for automatic upgrades                            [ NOT FOUND ]

[+] Networking
------------------------------------
  - Checking IPv6 configuration                               [ ENABLED ]
      Configuration method                                    [ AUTO ]
      IPv6 only                                               [ NO ]
  - Checking configured nameservers
    - Testing nameservers
        Nameserver: 8.8.8.8                                   [ OK ]
        Nameserver: 1.1.1.1                                   [ OK ]
    - Minimal of 2 responsive nameservers                     [ OK ]
  - Checking default gateway                                  [ DONE ]
  - Getting listening ports (TCP/UDP)                         [ DONE ]
  - Checking promiscuous interfaces                           [ WARNING ]
  - Checking waiting connections                              [ OK ]
  - Checking status DHCP client                               [ RUNNING ]
  - Checking for ARP monitoring software                      [ NOT FOUND ]
  - Uncommon network protocols                                [ 0 ]

[+] Printers and Spools
------------------------------------
  - Checking cups daemon                                      [ NOT FOUND ]
  - Checking lp daemon                                        [ NOT RUNNING ]

[+] Software: e-mail and messaging
------------------------------------
  - Postfix status                                            [ RUNNING ]
    - Postfix configuration                                   [ FOUND ]
      - Postfix banner                                        [ WARNING ]

[+] Software: firewalls
------------------------------------
  - Checking iptables kernel module                           [ FOUND ]
    - Checking iptables policies of chains                    [ FOUND ]
      - Chain INPUT (table: filter, target: ACCEPT)           [ ACCEPT ]
      - Chain INPUT (table: security, target: ACCEPT)         [ ACCEPT ]
    - Checking for empty ruleset                              [ WARNING ]
    - Checking for unused rules                               [ OK ]
  - Checking host based firewall                              [ ACTIVE ]

[+] Software: webserver
------------------------------------
  - Checking Apache                                           [ NOT FOUND ]
  - Checking nginx                                            [ NOT FOUND ]

[+] SSH Support
------------------------------------
  - Checking running SSH daemon                               [ FOUND ]
    - Searching SSH configuration                             [ FOUND ]
    - OpenSSH option: AllowTcpForwarding                      [ SUGGESTION ]
    - OpenSSH option: ClientAliveCountMax                     [ SUGGESTION ]
    - OpenSSH option: ClientAliveInterval                     [ OK ]
    - OpenSSH option: FingerprintHash                         [ OK ]
    - OpenSSH option: GatewayPorts                            [ OK ]
    - OpenSSH option: IgnoreRhosts                            [ OK ]
    - OpenSSH option: LoginGraceTime                          [ OK ]
    - OpenSSH option: LogLevel                                [ SUGGESTION ]
    - OpenSSH option: MaxAuthTries                            [ SUGGESTION ]
    - OpenSSH option: MaxSessions                             [ SUGGESTION ]
    - OpenSSH option: PermitRootLogin                         [ SUGGESTION ]
    - OpenSSH option: PermitUserEnvironment                   [ OK ]
    - OpenSSH option: PermitTunnel                            [ OK ]
    - OpenSSH option: Port                                    [ SUGGESTION ]
    - OpenSSH option: PrintLastLog                            [ OK ]
    - OpenSSH option: StrictModes                             [ OK ]
    - OpenSSH option: TCPKeepAlive                            [ SUGGESTION ]
    - OpenSSH option: UseDNS                                  [ OK ]
    - OpenSSH option: X11Forwarding                           [ SUGGESTION ]
    - OpenSSH option: AllowAgentForwarding                    [ SUGGESTION ]
    - OpenSSH option: AllowUsers                              [ NOT FOUND ]
    - OpenSSH option: AllowGroups                             [ NOT FOUND ]

[+] SNMP Support
------------------------------------
  - Checking running SNMP daemon                              [ NOT FOUND ]

[+] Databases
------------------------------------
  - PostgreSQL processes status                               [ FOUND ]

[+] LDAP Services
------------------------------------
  - Checking OpenLDAP instance                                [ NOT FOUND ]

[+] PHP
------------------------------------
  - Checking PHP                                              [ NOT FOUND ]

[+] Squid Support
------------------------------------
  - Checking running Squid daemon                             [ NOT FOUND ]

[+] Logging and files
------------------------------------
  - Checking for a running log daemon                         [ OK ]
    - Checking Syslog-NG status                               [ NOT FOUND ]
    - Checking systemd journal status                         [ FOUND ]
    - Checking Metalog status                                 [ NOT FOUND ]
    - Checking RSyslog status                                 [ NOT FOUND ]
    - Checking RFC 3195 daemon status                         [ NOT FOUND ]
    - Checking minilogd instances                             [ NOT FOUND ]
    - Checking wazuh-agent daemon status                      [ NOT FOUND ]
  - Checking logrotate presence                               [ OK ]
  - Checking remote logging                                   [ NOT ENABLED ]
  - Checking log directories (static list)                    [ DONE ]
  - Checking open log files                                   [ DONE ]
  - Checking deleted files in use                             [ FILES FOUND ]

[+] Insecure services
------------------------------------
  - Installed inetd package                                   [ NOT FOUND ]
  - Installed xinetd package                                  [ OK ]
    - xinetd status                                           [ NOT ACTIVE ]
  - Installed rsh client package                              [ OK ]
  - Installed rsh server package                              [ OK ]
  - Installed telnet client package                           [ OK ]
  - Installed telnet server package                           [ NOT FOUND ]
  - Checking NIS client installation                          [ OK ]
  - Checking NIS server installation                          [ OK ]
  - Checking TFTP client installation                         [ OK ]
  - Checking TFTP server installation                         [ OK ]

[+] Banners and identification
------------------------------------
  - /etc/issue                                                [ FOUND ]
    - /etc/issue contents                                     [ WEAK ]
  - /etc/issue.net                                            [ FOUND ]
    - /etc/issue.net contents                                 [ WEAK ]

[+] Scheduled tasks
------------------------------------
  - Checking crontab and cronjob files                        [ DONE ]

[+] Accounting
------------------------------------
  - Checking accounting information                           [ NOT FOUND ]
  - Checking sysstat accounting data                          [ NOT FOUND ]
  - Checking auditd                                           [ NOT FOUND ]

[+] Time and Synchronization
------------------------------------
  - NTP daemon found: chronyd                                 [ FOUND ]
  - Checking for a running NTP daemon or client               [ OK ]

[+] Cryptography
------------------------------------
  - Checking for expired SSL certificates [0/152]             [ NONE ]

  [WARNING]: Test CRYP-7902 had a long execution: 14.814156 seconds

  - Found 0 encrypted and 1 unencrypted swap devices in use.  [ OK ]
  - Kernel entropy is sufficient                              [ YES ]
  - HW RNG & rngd                                             [ NO ]
  - SW prng                                                   [ YES ]
  MOR-bit set                                                 [ YES ]

[+] Virtualization
------------------------------------

[+] Containers
------------------------------------
    - Docker
      - Docker daemon                                         [ RUNNING ]

[+] Security frameworks
------------------------------------
  - Checking presence AppArmor                                [ FOUND ]
    - Checking AppArmor status                                [ ENABLED ]
        Found 84 unconfined processes
  - Checking presence SELinux                                 [ NOT FOUND ]
  - Checking presence TOMOYO Linux                            [ NOT FOUND ]
  - Checking presence grsecurity                              [ NOT FOUND ]
  - Checking for implemented MAC framework                    [ OK ]

[+] Software: file integrity
------------------------------------
  - Checking file integrity tools
  - dm-integrity (status)                                     [ DISABLED ]
  - dm-verity (status)                                        [ DISABLED ]
  - Checking presence integrity tool                          [ NOT FOUND ]

[+] Software: System tooling
------------------------------------
  - Checking automation tooling
  - Automation tooling                                        [ NOT FOUND ]
  - Checking presence of Fail2ban                             [ FOUND ]
2026-03-20 08:18:52,655 fail2ban.jailreader     [264648]: WARNING Have not found any log file for proxmox jail
    - Checking Fail2ban jails                                 [ ENABLED ]
  - Checking for IDS/IPS tooling                              [ FOUND ]

[+] Software: Malware
------------------------------------
  - Malware software components                               [ NOT FOUND ]

[+] File Permissions
------------------------------------
  - Starting file permissions check
    File: /boot/grub/grub.cfg                                 [ OK ]
    File: /etc/crontab                                        [ SUGGESTION ]
    File: /etc/group                                          [ OK ]
    File: /etc/group-                                         [ OK ]
    File: /etc/hosts.allow                                    [ OK ]
    File: /etc/hosts.deny                                     [ OK ]
    File: /etc/issue                                          [ OK ]
    File: /etc/issue.net                                      [ OK ]
    File: /etc/motd                                           [ OK ]
    File: /etc/passwd                                         [ OK ]
    File: /etc/passwd-                                        [ OK ]
    File: /etc/ssh/sshd_config                                [ SUGGESTION ]
    Directory: /root/.ssh                                     [ OK ]
    Directory: /etc/cron.d                                    [ SUGGESTION ]
    Directory: /etc/cron.daily                                [ SUGGESTION ]
    Directory: /etc/cron.hourly                               [ SUGGESTION ]
    Directory: /etc/cron.weekly                               [ SUGGESTION ]
    Directory: /etc/cron.monthly                              [ SUGGESTION ]

[+] Home directories
------------------------------------
  - Permissions of home directories                           [ OK ]
  - Ownership of home directories                             [ OK ]
  - Checking shell history files                              [ OK ]

[+] Kernel Hardening
------------------------------------
  - Comparing sysctl key pairs with scan profile
    - dev.tty.ldisc_autoload (exp: 0)                         [ DIFFERENT ]
    - fs.protected_fifos (exp: 2)                             [ DIFFERENT ]
    - fs.protected_hardlinks (exp: 1)                         [ OK ]
    - fs.protected_regular (exp: 2)                           [ OK ]
    - fs.protected_symlinks (exp: 1)                          [ OK ]
    - fs.suid_dumpable (exp: 0)                               [ OK ]
    - kernel.core_uses_pid (exp: 1)                           [ OK ]
    - kernel.ctrl-alt-del (exp: 0)                            [ OK ]
    - kernel.dmesg_restrict (exp: 1)                          [ OK ]
    - kernel.kptr_restrict (exp: 2)                           [ DIFFERENT ]
    - kernel.modules_disabled (exp: 1)                        [ DIFFERENT ]
    - kernel.perf_event_paranoid (exp: 2 3 4)                 [ OK ]
    - kernel.randomize_va_space (exp: 2)                      [ OK ]
    - kernel.sysrq (exp: 0)                                   [ DIFFERENT ]
    - kernel.unprivileged_bpf_disabled (exp: 1)               [ DIFFERENT ]
    - kernel.yama.ptrace_scope (exp: 1 2 3)                   [ OK ]
    - net.core.bpf_jit_harden (exp: 2)                        [ DIFFERENT ]
    - net.ipv4.conf.all.accept_redirects (exp: 0)             [ OK ]
    - net.ipv4.conf.all.accept_source_route (exp: 0)          [ OK ]
    - net.ipv4.conf.all.bootp_relay (exp: 0)                  [ OK ]
    - net.ipv4.conf.all.forwarding (exp: 0)                   [ DIFFERENT ]
    - net.ipv4.conf.all.log_martians (exp: 1)                 [ DIFFERENT ]
    - net.ipv4.conf.all.mc_forwarding (exp: 0)                [ OK ]
    - net.ipv4.conf.all.proxy_arp (exp: 0)                    [ OK ]
    - net.ipv4.conf.all.rp_filter (exp: 1)                    [ DIFFERENT ]
    - net.ipv4.conf.all.send_redirects (exp: 0)               [ OK ]
    - net.ipv4.conf.default.accept_redirects (exp: 0)         [ OK ]
    - net.ipv4.conf.default.accept_source_route (exp: 0)      [ OK ]
    - net.ipv4.conf.default.log_martians (exp: 1)             [ DIFFERENT ]
    - net.ipv4.icmp_echo_ignore_broadcasts (exp: 1)           [ OK ]
    - net.ipv4.icmp_ignore_bogus_error_responses (exp: 1)     [ OK ]
    - net.ipv4.tcp_syncookies (exp: 1)                        [ OK ]
    - net.ipv4.tcp_timestamps (exp: 0 1)                      [ OK ]
    - net.ipv6.conf.all.accept_redirects (exp: 0)             [ DIFFERENT ]
    - net.ipv6.conf.all.accept_source_route (exp: 0)          [ OK ]
    - net.ipv6.conf.default.accept_redirects (exp: 0)         [ DIFFERENT ]
    - net.ipv6.conf.default.accept_source_route (exp: 0)      [ OK ]

[+] Hardening
------------------------------------
    - Installed compiler(s)                                   [ FOUND ]
    - Installed malware scanner                               [ NOT FOUND ]
    - Non-native binary formats                               [ FOUND ]

[+] Custom tests
------------------------------------
  - Running custom tests...                                   [ NONE ]

[+] Plugins (phase 2)
------------------------------------

================================================================================

  -[ Lynis 3.1.4 Results ]-

  Warnings (3):
  ----------------------------
  ! Found promiscuous interface [NETW-3015] 
    - Details  : enp4s0
    - Solution : Determine if this mode is required or whitelist interface in profile
      https://cisofy.com/lynis/controls/NETW-3015/

  ! Found some information disclosure in SMTP banner (OS or software name) [MAIL-8818] 
      https://cisofy.com/lynis/controls/MAIL-8818/

  ! iptables module(s) loaded, but no rules active [FIRE-4512] 
      https://cisofy.com/lynis/controls/FIRE-4512/

  Suggestions (52):
  ----------------------------
  * This release is more than 4 months old. Check the website or GitHub to see if there is an update available. [LYNIS] 
    - Related resources
      * Website: https://cisofy.com/lynis/controls/LYNIS/

  * Install libpam-tmpdir to set $TMP and $TMPDIR for PAM sessions [DEB-0280] 
    - Related resources
      * Website: https://cisofy.com/lynis/controls/DEB-0280/

  * Install apt-listbugs to display a list of critical bugs prior to each APT installation. [DEB-0810] 
    - Related resources
      * Website: https://cisofy.com/lynis/controls/DEB-0810/

  * Install needrestart, alternatively to debian-goodies, so that you can run needrestart after upgrades to determine which daemons are using old versions of libraries and need restarting. [DEB-0831] 
    - Related resources
      * Website: https://cisofy.com/lynis/controls/DEB-0831/

  * Copy /etc/fail2ban/jail.conf to jail.local to prevent it being changed by updates. [DEB-0880] 
    - Related resources
      * Website: https://cisofy.com/lynis/controls/DEB-0880/

  * Set a password on GRUB boot loader to prevent altering boot configuration (e.g. boot in single user mode without password) [BOOT-5122] 
    - Related resources
      * Website: https://cisofy.com/lynis/controls/BOOT-5122/

  * Determine runlevel and services at startup [BOOT-5180] 
    - Related resources
      * Website: https://cisofy.com/lynis/controls/BOOT-5180/

  * Consider hardening system services [BOOT-5264] 
    - Details  : Run '/usr/bin/systemd-analyze security SERVICE' for each service
    - Related resources
      * Article: Systemd features to secure service files: https://linux-audit.com/systemd/systemd-features-to-secure-units-and-services/
      * Website: https://cisofy.com/lynis/controls/BOOT-5264/

  * Determine why /vmlinuz or /boot/vmlinuz is missing on this Debian/Ubuntu system. [KRNL-5788] 
    - Details  : /vmlinuz or /boot/vmlinuz
    - Related resources
      * Website: https://cisofy.com/lynis/controls/KRNL-5788/

  * Configure password hashing rounds in /etc/login.defs [AUTH-9230] 
    - Related resources
      * Article: Linux password security: hashing rounds: https://linux-audit.com/authentication/configure-the-minimum-password-length-on-linux-systems/
      * Website: https://cisofy.com/lynis/controls/AUTH-9230/

  * Install a PAM module for password strength testing like pam_cracklib or pam_passwdqc or libpam-passwdqc [AUTH-9262] 
    - Related resources
      * Article: Configure minimum password length for Linux systems: https://linux-audit.com/configure-the-minimum-password-length-on-linux-systems/
      * Website: https://cisofy.com/lynis/controls/AUTH-9262/

  * When possible set expire dates for all password protected accounts [AUTH-9282] 
    - Related resources
      * Website: https://cisofy.com/lynis/controls/AUTH-9282/

  * Look at the locked accounts and consider removing them [AUTH-9284] 
    - Related resources
      * Website: https://cisofy.com/lynis/controls/AUTH-9284/

  * Configure minimum password age in /etc/login.defs [AUTH-9286] 
    - Related resources
      * Article: Configure minimum password length for Linux systems: https://linux-audit.com/configure-the-minimum-password-length-on-linux-systems/
      * Website: https://cisofy.com/lynis/controls/AUTH-9286/

  * Configure maximum password age in /etc/login.defs [AUTH-9286] 
    - Related resources
      * Article: Configure minimum password length for Linux systems: https://linux-audit.com/configure-the-minimum-password-length-on-linux-systems/
      * Website: https://cisofy.com/lynis/controls/AUTH-9286/

  * Default umask in /etc/login.defs could not be found and defaults usually to 022, which could be more strict like 027 [AUTH-9328] 
    - Related resources
      * Article: Set default file permissions on Linux with umask: https://linux-audit.com/filesystems/file-permissions/set-default-file-permissions-with-umask/
      * Website: https://cisofy.com/lynis/controls/AUTH-9328/

  * To decrease the impact of a full /home file system, place /home on a separate partition [FILE-6310] 
    - Related resources
      * Website: https://cisofy.com/lynis/controls/FILE-6310/

  * To decrease the impact of a full /var file system, place /var on a separate partition [FILE-6310] 
    - Related resources
      * Website: https://cisofy.com/lynis/controls/FILE-6310/

  * Disable drivers like USB storage when not used, to prevent unauthorized storage or data theft [USB-1000] 
    - Related resources
      * Website: https://cisofy.com/lynis/controls/USB-1000/

  * Disable drivers like firewire storage when not used, to prevent unauthorized storage or data theft [STRG-1846] 
    - Related resources
      * Website: https://cisofy.com/lynis/controls/STRG-1846/

  * Purge old/removed packages (2 found) with aptitude purge or dpkg --purge command. This will cleanup old configuration files, cron jobs and startup scripts. [PKGS-7346] 
    - Related resources
      * Website: https://cisofy.com/lynis/controls/PKGS-7346/

  * Install debsums utility for the verification of packages with known good database. [PKGS-7370] 
    - Related resources
      * Website: https://cisofy.com/lynis/controls/PKGS-7370/

  * Install package apt-show-versions for patch management purposes [PKGS-7394] 
    - Related resources
      * Website: https://cisofy.com/lynis/controls/PKGS-7394/

  * Consider using a tool to automatically apply upgrades [PKGS-7420] 
    - Related resources
      * Website: https://cisofy.com/lynis/controls/PKGS-7420/

  * Determine if protocol 'dccp' is really needed on this system [NETW-3200] 
    - Related resources
      * Website: https://cisofy.com/lynis/controls/NETW-3200/

  * Determine if protocol 'sctp' is really needed on this system [NETW-3200] 
    - Related resources
      * Website: https://cisofy.com/lynis/controls/NETW-3200/

  * Determine if protocol 'rds' is really needed on this system [NETW-3200] 
    - Related resources
      * Website: https://cisofy.com/lynis/controls/NETW-3200/

  * Determine if protocol 'tipc' is really needed on this system [NETW-3200] 
    - Related resources
      * Website: https://cisofy.com/lynis/controls/NETW-3200/

  * You are advised to hide the mail_name (option: smtpd_banner) from your postfix configuration. Use postconf -e or change your main.cf file (/etc/postfix/main.cf) [MAIL-8818] 
    - Related resources
      * Article: Postfix Hardening Guide for Security and Privacy: https://linux-audit.com/postfix-hardening-guide-for-security-and-privacy/
      * Website: https://cisofy.com/lynis/controls/MAIL-8818/

  * Consider hardening SSH configuration [SSH-7408] 
    - Details  : AllowTcpForwarding (set YES to NO)
    - Related resources
      * Article: OpenSSH security and hardening: https://linux-audit.com/ssh/audit-and-harden-your-ssh-configuration/
      * Website: https://cisofy.com/lynis/controls/SSH-7408/

  * Consider hardening SSH configuration [SSH-7408] 
    - Details  : ClientAliveCountMax (set 3 to 2)
    - Related resources
      * Article: OpenSSH security and hardening: https://linux-audit.com/ssh/audit-and-harden-your-ssh-configuration/
      * Website: https://cisofy.com/lynis/controls/SSH-7408/

  * Consider hardening SSH configuration [SSH-7408] 
    - Details  : LogLevel (set INFO to VERBOSE)
    - Related resources
      * Article: OpenSSH security and hardening: https://linux-audit.com/ssh/audit-and-harden-your-ssh-configuration/
      * Website: https://cisofy.com/lynis/controls/SSH-7408/

  * Consider hardening SSH configuration [SSH-7408] 
    - Details  : MaxAuthTries (set 6 to 3)
    - Related resources
      * Article: OpenSSH security and hardening: https://linux-audit.com/ssh/audit-and-harden-your-ssh-configuration/
      * Website: https://cisofy.com/lynis/controls/SSH-7408/

  * Consider hardening SSH configuration [SSH-7408] 
    - Details  : MaxSessions (set 10 to 2)
    - Related resources
      * Article: OpenSSH security and hardening: https://linux-audit.com/ssh/audit-and-harden-your-ssh-configuration/
      * Website: https://cisofy.com/lynis/controls/SSH-7408/

  * Consider hardening SSH configuration [SSH-7408] 
    - Details  : PermitRootLogin (set YES to (FORCED-COMMANDS-ONLY|NO|PROHIBIT-PASSWORD|WITHOUT-PASSWORD))
    - Related resources
      * Article: OpenSSH security and hardening: https://linux-audit.com/ssh/audit-and-harden-your-ssh-configuration/
      * Website: https://cisofy.com/lynis/controls/SSH-7408/

  * Consider hardening SSH configuration [SSH-7408] 
    - Details  : Port (set 22 to )
    - Related resources
      * Article: OpenSSH security and hardening: https://linux-audit.com/ssh/audit-and-harden-your-ssh-configuration/
      * Website: https://cisofy.com/lynis/controls/SSH-7408/

  * Consider hardening SSH configuration [SSH-7408] 
    - Details  : TCPKeepAlive (set YES to NO)
    - Related resources
      * Article: OpenSSH security and hardening: https://linux-audit.com/ssh/audit-and-harden-your-ssh-configuration/
      * Website: https://cisofy.com/lynis/controls/SSH-7408/

  * Consider hardening SSH configuration [SSH-7408] 
    - Details  : X11Forwarding (set YES to NO)
    - Related resources
      * Article: OpenSSH security and hardening: https://linux-audit.com/ssh/audit-and-harden-your-ssh-configuration/
      * Website: https://cisofy.com/lynis/controls/SSH-7408/

  * Consider hardening SSH configuration [SSH-7408] 
    - Details  : AllowAgentForwarding (set YES to NO)
    - Related resources
      * Article: OpenSSH security and hardening: https://linux-audit.com/ssh/audit-and-harden-your-ssh-configuration/
      * Website: https://cisofy.com/lynis/controls/SSH-7408/

  * Enable logging to an external logging host for archiving purposes and additional protection [LOGG-2154] 
    - Related resources
      * Website: https://cisofy.com/lynis/controls/LOGG-2154/

  * Check what deleted files are still in use and why. [LOGG-2190] 
    - Related resources
      * Website: https://cisofy.com/lynis/controls/LOGG-2190/

  * Add a legal banner to /etc/issue, to warn unauthorized users [BANN-7126] 
    - Related resources
      * Article: The real purpose of login banners: https://linux-audit.com/the-real-purpose-of-login-banners-on-linux/
      * Website: https://cisofy.com/lynis/controls/BANN-7126/

  * Add legal banner to /etc/issue.net, to warn unauthorized users [BANN-7130] 
    - Related resources
      * Article: The real purpose of login banners: https://linux-audit.com/the-real-purpose-of-login-banners-on-linux/
      * Website: https://cisofy.com/lynis/controls/BANN-7130/

  * Enable process accounting [ACCT-9622] 
    - Related resources
      * Website: https://cisofy.com/lynis/controls/ACCT-9622/

  * Enable sysstat to collect accounting (no results) [ACCT-9626] 
    - Related resources
      * Website: https://cisofy.com/lynis/controls/ACCT-9626/

  * Enable auditd to collect audit information [ACCT-9628] 
    - Related resources
      * Article: Linux audit framework 101: basic rules for configuration: https://linux-audit.com/linux-audit-framework/linux-audit-framework-101-basic-rules-for-configuration/
      * Article: Monitoring Linux file access, changes and data modifications: https://linux-audit.com/monitoring-linux-file-access-changes-and-modifications/
      * Website: https://cisofy.com/lynis/controls/ACCT-9628/

  * Install a file integrity tool to monitor changes to critical and sensitive files [FINT-4350] 
    - Related resources
      * Article: Monitoring Linux file access, changes and data modifications: https://linux-audit.com/monitoring-linux-file-access-changes-and-modifications/
      * Article: Monitor for file changes on Linux: https://linux-audit.com/monitor-for-file-system-changes-on-linux/
      * Website: https://cisofy.com/lynis/controls/FINT-4350/

  * Determine if automation tools are present for system management [TOOL-5002] 
    - Related resources
      * Website: https://cisofy.com/lynis/controls/TOOL-5002/

  * Consider restricting file permissions [FILE-7524] 
    - Details  : See screen output or log file
    - Solution : Use chmod to change file permissions
    - Related resources
      * Website: https://cisofy.com/lynis/controls/FILE-7524/

  * One or more sysctl values differ from the scan profile and could be tweaked [KRNL-6000] 
    - Solution : Change sysctl value or disable test (skip-test=KRNL-6000:<sysctl-key>)
    - Related resources
      * Article: Linux hardening with sysctl settings: https://linux-audit.com/linux-hardening-with-sysctl/
      * Article: Overview of sysctl options and values: https://linux-audit.com/kernel/sysctl/
      * Website: https://cisofy.com/lynis/controls/KRNL-6000/

  * Harden compilers like restricting access to root user only [HRDN-7222] 
    - Related resources
      * Article: Why remove compilers from your system?: https://linux-audit.com/software/why-remove-compilers-from-your-system/
      * Website: https://cisofy.com/lynis/controls/HRDN-7222/

  * Harden the system by installing at least one malware scanner, to perform periodic file system scans [HRDN-7230] 
    - Solution : Install a tool like rkhunter, chkrootkit, OSSEC, Wazuh
    - Related resources
      * Article: Antivirus for Linux: is it really needed?: https://linux-audit.com/malware/antivirus-for-linux-really-needed/
      * Article: Monitoring Linux Systems for Rootkits: https://linux-audit.com/monitoring-linux-systems-for-rootkits/
      * Website: https://cisofy.com/lynis/controls/HRDN-7230/

  Follow-up:
  ----------------------------
  - Show details of a test (lynis show details TEST-ID)
  - Check the logfile for all details (less /var/log/lynis.log)
  - Read security controls texts (https://cisofy.com)
  - Use --upload to upload data to central system (Lynis Enterprise users)

================================================================================

  Lynis security scan details:

  Hardening index : 63 [############        ]
  Tests performed : 272
  Plugins enabled : 1

  Components:
  - Firewall               [V]
  - Malware scanner        [X]

  Scan mode:
  Normal [V]  Forensics [ ]  Integration [ ]  Pentest [ ]

  Lynis modules:
  - Compliance status      [?]
  - Security audit         [V]
  - Vulnerability scan     [V]

  Files:
  - Test and debug information      : /var/log/lynis.log
  - Report data                     : /var/log/lynis-report.dat

================================================================================

  Lynis 3.1.4

  Auditing, system hardening, and compliance for UNIX-based systems
  (Linux, macOS, BSD, and others)

  2007-2024, CISOfy - https://cisofy.com/lynis/
  Enterprise support available (compliance, plugins, interface and tools)

================================================================================

  [TIP]: Enhance Lynis audits by adding your settings to custom.prf (see /etc/lynis/default.prf for all settings)
