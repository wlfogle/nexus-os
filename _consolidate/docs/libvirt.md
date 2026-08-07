cockpit for windows

Cockpit is not natively available for Windows, but you can access it through a web browser on any operating system, including Windows, by connecting to a server where Cockpit is installed. You can also use a self-contained application for Linux, but for Windows users, the web interface is the primary method.
Anonymized by DuckDuckGo. Zero data retention for this chat. No AI training. Learn more

i want to add my windows vm to cockpit

You can’t add a Windows VM to Cockpit directly, because Cockpit’s “hosts” are Linux systems running Cockpit. To manage that VM from Cockpit, you have two practical options:

    Manage the Cockpit host (Linux)

    Install Cockpit on the Linux VM you want to manage.
    If you want VM management inside Cockpit, also install the Machines module (cockpit-machines) on that Cockpit server (or another Linux box that has libvirt).

    Access your Windows VM another way

    Use RDP/Windows tooling for the Windows VM, and use Cockpit only for the Linux machines (or for the hypervisor host running the Windows VM), where Cockpit + libvirt/VM tooling can show/manage guests.

If you tell me what your setup is (is your Cockpit server a Linux VM on the same hypervisor, and what hypervisor—libvirt/KVM, VMware, VirtualBox, Proxmox?), I’ll give the exact steps for that path.

it is a windows 11 vm

Cockpit can’t manage a Windows 11 VM directly. Install Cockpit on a Linux host (or a Linux VM) and then manage things from there.

Tell me which you want:

    Manage your Windows 11 VM using Cockpit (you’d need a Linux libvirt/KVM-style host that can “see” the VM), or
    Just view/manage the Windows VM while using Cockpit for the rest (you’d use Windows tools like RDP for the VM)?

Also, what’s running the VM—KVM/libvirt, VMware, VirtualBox, or something else?

proxmox vm

To add your Proxmox-hosted Windows 11 VM to Cockpit, you can’t “import” it into Cockpit unless Cockpit is running on a Linux host with libvirt/qemu that has access to the VMs. Cockpit doesn’t natively speak Proxmox’s management API.

What to do instead:

    Manage the VM from Proxmox (web UI / console).
    Use Cockpit only on a Linux libvirt host (e.g., where the underlying VMs are running) with the Virtual Machines module (cockpit-machines). Then Cockpit will list/manage those libvirt VMs automatically.

If you want, tell me:

    Is Cockpit running anywhere right now (which machine/IP)?
    Do you want Cockpit to manage Proxmox itself (host-level), or just the Windows guest?
    What hypervisor backend is Proxmox using for the VMs (KVM/libvirt — usually yes)?

https://192.168.12.204:9090/system

Since you’re using Proxmox, you can only get your Windows 11 guest VM to show up in Cockpit if Cockpit is installed on a Linux host that connects to the same libvirt/QEMU where those VMs live.

Right now, the URL you gave (https://192.168.12.204:9090/system) is Cockpit running on that box, so do this:

    Open Cockpit on 192.168.12.204 and check whether you have a Virtual Machines entry in the left menu.
    If not, install the virtual machines component on the Cockpit host (it must be a libvirt-capable Linux machine): cockpit-machines (and libvirtd if needed).
    If Virtual Machines exists but your Proxmox Windows VM is not listed, Cockpit is almost certainly not connected to Proxmox/libvirt (so it can’t “see” the VMs).

Answer these and I’ll tell you the exact path:

    Is Cockpit installed directly on the Proxmox host (192.168.12.204) or on a separate Linux VM/server?
    When you open Cockpit, do you see Virtual Machines in the left sidebar?

how connect cockpit to libvirt

To connect Cockpit to libvirt, Cockpit must be running on a Linux machine that has libvirt (qemu/libvirtd) installed and running. Then Cockpit’s “Virtual Machines” view will automatically talk to that local libvirt.

Do this on the Cockpit host:

    Install libvirt + Cockpit machines

bash

sudo apt update
sudo apt install -y libvirt-daemon-system libvirt-clients cockpit-machines

    Start/enable libvirtd

bash

sudo systemctl enable --now libvirtd
sudo systemctl status libvirtd

    Ensure your user can manage libvirt (pick one)

    Add to libvirt group (common):

bash

sudo usermod -aG libvirt $USER

Then log out/in.

    Or (if needed) use polkit prompts; simplest is just the group change.

    Open Cockpit and confirm “Virtual Machines” appears

    If it does, it should list libvirt guests.

If you’re trying to use the Proxmox host’s libvirt from a different machine (remote), you typically must configure libvirt remote access (e.g., TCP or SSH transport) and allow the connecting user/keys, then point Cockpit at that remote. Tell me your OS for the Cockpit host (Ubuntu/Debian/RHEL?) and whether Cockpit is running on Proxmox or on a separate server, and I’ll give the exact remote-connection steps.

cockpit is running on 4 systems, laptop .204, media stack .30, tiamat(proxmox host) .242, and bahamut .244

Which of those four machines is running Cockpit on port 9090 (the one you’re visiting)?

Cockpit will only show VMs in its Virtual Machines page if Cockpit is running on a Linux host that has libvirt installed and can reach the hypervisor’s libvirt (usually local libvirtd, or remote libvirt over SSH/TCP).

    If Cockpit is running on laptop .204: you must install libvirt + cockpit-machines there, and then you either:
        run the VMs there (local), or
        connect that Cockpit host to Proxmox/libvirt remotely.

    If you want Cockpit to manage the Proxmox guests: the easiest is to run/enable cockpit-machines on tiamat (.242) (the Proxmox host) so it can see the VMs locally.

Tell me:

    For each of those machines, do any of them run a package named cockpit-machines (or show “Virtual Machines” in Cockpit)?
    Which machine is the one you’re currently using in the browser (is it .204 or .30 or .242 or .244)?
    Do you want to manage Proxmox VMs from a remote Cockpit (from laptop/media stack), or just on the Proxmox host itself?

cockpit-machines on .204, and .30,browser running on laptop .204, remote cockpit on laptop

Great—right now Cockpit on .204 can see only libvirt available on .204. To see your Proxmox VMs on tiamat (.242), you have to connect .204’s Cockpit → tiamat’s libvirt (remote libvirt access).
Option A (recommended): run cockpit-machines on tiamat (.242)

So Cockpit runs next to libvirt.

    Install cockpit-machines on tiamat (.242)
    Then open Cockpit on tiamat and view/manage the VMs there.

Option B: connect .204 to tiamat’s libvirt remotely (remote libvirt over SSH)

On tiamat (.242) enable remote libvirt over SSH for qemu/libvirt:

    Ensure libvirt is running:

bash

sudo systemctl enable --now libvirtd
sudo systemctl status libvirtd

    Create a dedicated libvirt user (optional but clean), then allow it in polkit/permissions (often easiest via adding to libvirt group):

bash

sudo usermod -aG libvirt <your-user>

(log out/in)

    On tiamat (.242) make sure SSH works normally (keys, no password prompts if possible).

On .204 configure Cockpit machines to use the remote libvirt URI (usually via libvirt clients / environment).

    Install client bits:

bash

sudo apt install -y libvirt-clients

Then you’ll need to tell Cockpit/libvirt to connect to remote URI, typically one of:

    qemu+ssh://USER@tiamat/system or
    `qemu+ssh://USER@192.168.12.242/system

If you tell me the OS on .204 (Ubuntu/Debian/etc.) and on tiamat (.242), I’ll give the exact commands/steps to set the libvirt connection so the Virtual Machines page on .204 lists the Proxmox guests.

i don't think cockpit-machines will install on proxmox host

You’re right to be suspicious: on a Proxmox host, cockpit-machines usually isn’t a default/clean install, and whether it works depends on the Proxmox version and its packaged components. The more reliable path is:
Use Cockpit on .204 with remote libvirt over SSH

Cockpit’s “Virtual Machines” view works when the Cockpit host can talk to libvirt on the Proxmox node. That means you must make libvirt on tiamat (.242) reachable remotely, and then point .204’s libvirt tooling/Cockpit at it.
1) On tiamat (.242): ensure libvirt is accessible via libvirt over SSH

    Confirm libvirtd is running:

bash

sudo systemctl status libvirtd

    Ensure SSH access works for the user you’ll use from .204 (keys recommended):

bash

ssh <user>@tiamat

    Make sure that user has permission to query libvirt (commonly by being in the libvirt group):

bash

sudo usermod -aG libvirt <user>

Log out/in.
2) On .204: test remote libvirt connectivity first

Install libvirt clients (if you don’t have them):
bash

sudo apt install -y libvirt-clients

Then test:
bash

virsh -c qemu+ssh://<user>@tiamat/system list --all

If this returns your VM list, Cockpit should be able to do the same once configured to use that libvirt URI.
3) Configure Cockpit to use the remote libvirt URI

The exact UI/config differs by distro/version. The key is: set the libvirt connection Cockpit uses to:

    qemu+ssh://<user>@tiamat/system

If you tell me what OS is on .204 (Ubuntu/Debian/RHEL?) and what you see under Cockpit’s Virtual Machines page (does it show “No connections” / an error message?), I’ll give the exact click/command steps for that version.

Also: does your virsh -c qemu+ssh://<user>@tiamat/system list --all work?

 virsh -c qemu+ssh://root@tiamat/system list --all

error: failed to connect to the hypervisor
error: End of file while reading data: nc: invalid option -- 'U'
nc -h for help: Input/output error

That error pattern is almost always from a network/transport issue in the libvirt-over-SSH path (nc/transport getting the wrong options), or SSH isn’t reachable/usable for libvirt’s remote probe.

Do these in order:

    Confirm SSH connectivity from .204 to tiamat

bash

ssh root@tiamat

(If you can’t log in cleanly, fix SSH first.)

    Confirm you can reach tiamat over TCP 22 from .204

bash

nc -vz tiamat 22

    Try libvirt remote with the IP (not hostname) and verbose

bash

virsh -c 'qemu+ssh://root@192.168.12.242/system' -d 3 list --all

    If (3) still fails, libvirt is likely using nc in a way that’s incompatible. Check what nc is on .204:

bash

which nc
nc -h | head -n 20

    Quick workaround: ensure the ssh transport can run without libvirt needing netcat options Install the “netcat-openbsd” variant (common fix on Debian/Ubuntu):

bash

sudo apt update
sudo apt install -y netcat-openbsd

Then retry step (3).

If you paste outputs of (2) nc -vz, (3) virsh -d 3 (just the last ~20 lines), and (5) which nc + the first lines of nc -h, I’ll tell you the exact fix for your distro.

ssh tiamat
Last login: Sat Jun 27 15:16:52 2026 from 192.168.12.204
Welcome to fish, the friendly interactive shell
Type help for instructions on how to use fish
root@tiamat ~# 5u

You can SSH in, so the failure is specifically in the libvirt SSH transport (nc variant/options) on .204.

From .204, run these and paste the output:

bash

which nc
nc -h | head -n 30
nc -vz 192.168.12.242 22

    Then retry virsh with debug:

bash

virsh -c 'qemu+ssh://root@192.168.12.242/system' -d 3 list --all

Also, on .204, install a known-good netcat:
bash

sudo apt update
sudo apt install -y netcat-openbsd

Then run the virsh command again. If your .204 isn’t Debian/Ubuntu, tell me its OS and I’ll give the equivalent package/command.

loufogle@pop-os ~> which nc
                   nc -h | head -n 30
                   nc -vz 192.168.12.242 22

/usr/bin/nc
OpenBSD netcat (Debian patchlevel 1.218-4ubuntu1)
usage: nc [-46CDdFhklNnrStUuvZz] [-I length] [-i interval] [-M ttl]
          [-m minttl] [-O length] [-P proxy_username] [-p source_port]
          [-q seconds] [-s sourceaddr] [-T keyword] [-V rtable] [-W recvlimit]
          [-w timeout] [-X proxy_protocol] [-x proxy_address[:port]]
          [destination] [port]
        Command Summary:
                -4              Use IPv4
                -6              Use IPv6
                -b              Allow broadcast
                -C              Send CRLF as line-ending
                -D              Enable the debug socket option
                -d              Detach from stdin
                -F              Pass socket fd
                -h              This help text
                -I length       TCP receive buffer length
                -i interval     Delay interval for lines sent, ports scanned
                -k              Keep inbound sockets open for multiple connects
                -l              Listen mode, for inbound connects
                -M ttl          Outgoing TTL / Hop Limit
                -m minttl       Minimum incoming TTL / Hop Limit
                -N              Shutdown the network socket after EOF on stdin
                -n              Suppress name/port resolutions
                -O length       TCP send buffer length
                -P proxyuser    Username for proxy authentication
                -p port         Specify local port for remote connects
                -q secs         quit after EOF on stdin and delay of secs
                -r              Randomize remote ports
                -S              Enable the TCP MD5 signature option
                -s sourceaddr   Local source address
                -T keyword      TOS value
                -t              Answer TELNET negotiation
                -U              Use UNIX domain socket
                -u              UDP mode
                -V rtable       Specify alternate routing table
                -v              Verbose
                -W recvlimit    Terminate after receiving a number of packets
                -w timeout      Timeout for connects and final net reads
                -X proto        Proxy protocol: "4", "5" (SOCKS) or "connect"
                -x addr[:port]  Specify proxy address and port
                -Z              DCCP mode
                -z              Zero-I/O mode [used for scanning]
        Port numbers can be individual or ranges: lo-hi [inclusive]
Connection to 192.168.12.242 22 port [tcp/ssh] succeeded!

OpenBSD netcat is installed on .204 and SSH works, so the remaining issue is that libvirt’s SSH transport is calling nc with an option your nc doesn’t support (your earlier error: nc: invalid option -- 'U').

Fix: make sure libvirt uses a netcat implementation that supports what it expects.

On pop-os/.204, install netcat-traditional and try again:
bash

sudo apt update
sudo apt install -y netcat-traditional
sudo update-alternatives --config nc

Then re-run:
bash

virsh -c 'qemu+ssh://root@192.168.12.242/system' list --all

If update-alternatives doesn’t exist or it won’t switch, tell me the output of:
bash

dpkg -l | grep -E 'netcat|openbsd|traditional' || true

and I’ll give the exact next command.

i use nala

Got it—use nala to install another netcat implementation, then switch nc to it.

On .204, run:
bash

sudo nala update
sudo nala install -y netcat-traditional

Then check which nc you’re using now:
bash

which nc
nc -h | grep -E '(-U|Use UNIX|traditional|OpenBSD)' | head

If it still shows OpenBSD netcat, switch alternatives (if available):
bash

sudo update-alternatives --config nc

Now retry:
bash

virsh -c 'qemu+ssh://root@192.168.12.242/system' list --all

If update-alternatives isn’t present, paste the output of:
bash

ls -l /etc/alternatives/nc* 2>/dev/null || true
ls -l /usr/bin/nc /bin/nc 2>/dev/null || true

Finished Successfully
loufogle@pop-os ~> which nc
/usr/bin/nc

Great—now that nc/transport works, Cockpit should be able to connect too.

Next:

    In Cockpit on .204, open Virtual Machines.
    If it shows no connection / can’t list anything, make sure you have libvirt-clients installed on .204:

bash

sudo nala install -y libvirt-clients

    Then restart cockpit-machines:

bash

sudo systemctl restart cockpit-machines

If you still don’t see the Proxmox VMs, tell me the exact message shown on the Cockpit “Virtual Machines” page (or paste the error text).

loufogle@pop-os ~> virsh -c 'qemu+ssh://root@192.168.12.242/system' list --all

error: failed to connect to the hypervisor
error: End of file while reading data: nc: invalid option -- 'U'
nc -h for help: Input/output error

That means libvirt/virsh is still using a nc transport path that triggers the unsupported -U option—so simply installing netcat-traditional didn’t change what libvirt is calling.

Do this on .204:

    Confirm what nc is currently:

bash

/usr/bin/nc -h | grep -m1 'Use UNIX domain socket' || true

    Install a netcat that supports -U behavior, and switch via alternatives if available. With nala:

bash

sudo nala install -y netcat-openbsd netcat-traditional

    Force libvirt to use a specific nc by checking where libvirtd expects it:

bash

dpkg -L libvirt-clients 2>/dev/null | grep -E 'libvirt|nc' || true

Because this is getting distro/version-specific, I need one more thing: paste output of
bash

virsh -c 'qemu+ssh://root@192.168.12.242/system' --connect --debug 2>&1 | tail -n 80

(or if --debug isn’t supported, run virsh ... -d 4 list --all and paste the last ~60 lines).

loufogle@pop-os ~ [1]> /usr/bin/nc -h | grep -m1 'Use UNIX domain socket' || true

OpenBSD netcat (Debian patchlevel 1.218-4ubuntu1)
usage: nc [-46CDdFhklNnrStUuvZz] [-I length] [-i interval] [-M ttl]
          [-m minttl] [-O length] [-P proxy_username] [-p source_port]
          [-q seconds] [-s sourceaddr] [-T keyword] [-V rtable] [-W recvlimit]
          [-w timeout] [-X proxy_protocol] [-x proxy_address[:port]]
          [destination] [port]
        Command Summary:
                -4              Use IPv4
                -6              Use IPv6
                -b              Allow broadcast
                -C              Send CRLF as line-ending
                -D              Enable the debug socket option
                -d              Detach from stdin
                -F              Pass socket fd
                -h              This help text
                -I length       TCP receive buffer length
                -i interval     Delay interval for lines sent, ports scanned
                -k              Keep inbound sockets open for multiple connects
                -l              Listen mode, for inbound connects
                -M ttl          Outgoing TTL / Hop Limit
                -m minttl       Minimum incoming TTL / Hop Limit
                -N              Shutdown the network socket after EOF on stdin
                -n              Suppress name/port resolutions
                -O length       TCP send buffer length
                -P proxyuser    Username for proxy authentication
                -p port         Specify local port for remote connects
                -q secs         quit after EOF on stdin and delay of secs
                -r              Randomize remote ports
                -S              Enable the TCP MD5 signature option
                -s sourceaddr   Local source address
                -T keyword      TOS value
                -t              Answer TELNET negotiation
                -U              Use UNIX domain socket
                -u              UDP mode
                -V rtable       Specify alternate routing table
                -v              Verbose
                -W recvlimit    Terminate after receiving a number of packets
                -w timeout      Timeout for connects and final net reads
                -X proto        Proxy protocol: "4", "5" (SOCKS) or "connect"
                -x addr[:port]  Specify proxy address and port
                -Z              DCCP mode
                -z              Zero-I/O mode [used for scanning]
        Port numbers can be individual or ranges: lo-hi [inclusive]
loufogle@pop-os ~> 

So nc is still OpenBSD netcat, which supports -U (per your help), but libvirt is still erroring on -U—meaning libvirt is not using this nc binary (likely it’s calling a different nc from somewhere on PATH, or it’s invoking BusyBox/another netcat).

Run these on .204 and paste the output:

bash

command -v nc
readlink -f "$(command -v nc)"

    Check what libvirt/virsh is actually using for the ssh transport:

bash

virsh -c 'qemu+ssh://root@192.168.12.242/system' -d 4 list --all 2>&1 | tail -n 120

Also, quick workaround while we diagnose:

    Install a single netcat implementation and ensure /usr/bin/nc becomes that one. On Pop!_OS, do:

bash

sudo nala install -y netcat-traditional
sudo rm -f /usr/bin/nc
sudo ln -s "$(command -v nc.traditional)" /usr/bin/nc

If you paste the virsh -d 4 tail, I can tell you exactly which nc path libvirt is calling and the cleanest fix.

loufogle@pop-os ~> command -v nc
                   readlink -f "$(command -v nc)"

fish: $(...) is not supported. In fish, please use '(command)'.
readlink -f "$(command -v nc)"
             ^
loufogle@pop-os ~> virsh -c 'qemu+ssh://root@192.168.12.242/system' -d 4 list --all 2>&1 | tail -n 120

error: failed to connect to the hypervisor
error: End of file while reading data: nc: invalid option -- 'U'
nc -h for help: Input/output error
loufogle@pop-os ~> 

Run this with fish-friendly syntax (no $()):

fish

command -v nc

    Force the resolution to display the exact path:

fish

readlink -f (command -v nc)

    Then run a netcat “smoke test” with -U so we know the binary actually supports it:

fish

nc -U -h 2>&1 | head -n 5

    Now do the same libvirt listing but with stderr saved (so we can see what it’s calling). Use:

fish

virsh -c 'qemu+ssh://root@192.168.12.242/system' -d 4 list --all 2>&1 | tail -n 200

Paste outputs of (1), (2), (3), and the (4) tail.

Also, immediate likely fix on Pop!_OS: libvirt may be installed with a helper that calls nc via a different name. The quickest workaround is to install the OpenBSD nc and ensure libvirt uses it by putting a wrapper earlier in PATH:

    Create a wrapper script that forwards to OpenBSD nc but rejects/ignores -U if needed:

fish

sudo tee /usr/local/bin/nc >/dev/null <<'EOF'
#!/bin/sh
exec /usr/bin/nc "$@"
EOF
sudo chmod +x /usr/local/bin/nc

But I want the outputs first so we don’t guess the wrong libvirt helper.

loufogle@pop-os ~> command -v nc

/usr/bin/nc
loufogle@pop-os ~> readlink -f (command -v nc)

/usr/bin/nc.openbsd
loufogle@pop-os ~> nc -U -h 2>&1 | head -n 5

OpenBSD netcat (Debian patchlevel 1.218-4ubuntu1)
usage: nc [-46CDdFhklNnrStUuvZz] [-I length] [-i interval] [-M ttl]
          [-m minttl] [-O length] [-P proxy_username] [-p source_port]
          [-q seconds] [-s sourceaddr] [-T keyword] [-V rtable] [-W recvlimit]
          [-w timeout] [-X proxy_protocol] [-x proxy_address[:port]]
loufogle@pop-os ~> virsh -c 'qemu+ssh://root@192.168.12.242/system' -d 4 list --all 2>&1 | tail -n 200

error: failed to connect to the hypervisor
error: End of file while reading data: nc: invalid option -- 'U'
nc -h for help: Input/output error

Since /usr/bin/nc.openbsd is fine with -U, the virsh error means libvirt is running a different nc (or a wrapper) than /usr/bin/nc.

Do this on .204 to force libvirt’s ssh transport to use the OpenBSD nc path:

    Find what ssh transport helper libvirt uses (look for nc usage):

fish

sudo grep -R "nc" -n /usr/share/libvirt 2>/dev/null | head -n 50
sudo grep -R "netcat" -n /usr/share/libvirt 2>/dev/null | head -n 50

    Then check libvirt’s config for remote shell/ssh options (especially transport):

fish

sudo systemctl cat libvirtd 2>/dev/null | sed -n '1,200p'

    Most directly: set a temporary environment for virsh to use a specific nc binary (libvirt honors this on some distros; if it doesn’t, we’ll see from logs):

fish

env LIBVIRT_DEBUG=1 VIRSH_DEBUG=1 LIBVIRT_NC=/usr/bin/nc.openbsd virsh -c 'qemu+ssh://root@192.168.12.242/system' list --all

Also tell me your .204 distro version:
fish

cat /etc/os-release

Paste the outputs of (1) and (2) and I’ll give the exact edit (file + line) to make libvirt use /usr/bin/nc.openbsd for the remote SSH transport.


    PDF

Your debug log shows the exact problem: libvirt’s ssh helper is running nc without -U support. It’s specifically executing nc and trying nc ... -U /var/run/libvirt/libvirt-sock, but the nc that libvirt sees doesn’t accept -U.

Fix it by making sure /usr/bin/nc is the OpenBSD netcat for the libvirt helper process:

    Verify openbsd nc path exists:

fish

ls -l /usr/bin/nc.openbsd /bin/nc 2>/dev/null || true

    Force nc to be OpenBSD via alternatives (Pop uses update-alternatives if installed):

fish

sudo apt update
sudo apt install -y netcat-openbsd
sudo update-alternatives --config nc

    Confirm what the libvirt-launched ssh helper will use by running the same remote command locally (test transport):

fish

ssh -l root -T -e none 192.168.12.242 'nc -h 2>&1 | head -n 2'

    Retry:

fish

virsh -c 'qemu+ssh://root@192.168.12.242/system' list --all

If update-alternatives isn’t available on your system, tell me the output of:
fish

ls -l /usr/bin/nc*

and I’ll give the exact ln -sf / package fix for Pop!OS 22.04.

loufogle@pop-os ~> ls -l /usr/bin/nc.openbsd /bin/nc 2>/dev/null || true

lrwxrwxrwx 1 root root    20 Sep 18  2024 /bin/nc -> /etc/alternatives/nc
-rwxr-xr-x 1 root root 39560 Feb 23  2022 /usr/bin/nc.openbsd
loufogle@pop-os ~> sudo apt update
                   sudo apt install -y netcat-openbsd
                   sudo update-alternatives --config nc

Ign:1 chrisjbawden.github.io/cockpit-dockermanager stable InRelease
Get:2 cli.github.com/packages stable InRelease [3,917 B]                                                                           
Hit:3 chrisjbawden.github.io/cockpit-dockermanager stable Release                                                                  
Hit:4 repo.45drives.com/enterprise/ubuntu jammy InRelease                                                                          
Ign:5 chrisjbawden.github.io/cockpit-dockermanager stable Release.gpg                                                              
Hit:6 ppa.launchpadcontent.net/alex-p/tesseract-ocr5/ubuntu jammy InRelease                                                        
Hit:7 ppa.launchpadcontent.net/ar-lex/gamescope/ubuntu jammy InRelease                                                             
Hit:8 ppa.launchpadcontent.net/gerardpuig/ppa/ubuntu jammy InRelease                                                               
Hit:9 ppa.launchpadcontent.net/michel-slm/distrobox/ubuntu jammy InRelease                                                         
Hit:10 ppa.launchpadcontent.net/tomtomtom/woeusb/ubuntu jammy InRelease                                                            
Hit:11 ppa.launchpadcontent.net/ubuntu-toolchain-r/test/ubuntu jammy InRelease                                                     
Hit:12 ppa.launchpadcontent.net/zhangsongcui3371/fastfetch/ubuntu jammy InRelease                                                  
Hit:13 deb.opera.com/opera-gx-stable stable InRelease                                                                              
Hit:14 deb.nodesource.com/node_20.x nodistro InRelease                                                                             
Get:15 pkgs.tailscale.com/stable/ubuntu jammy InRelease                                                                            
Ign:16 https://releases.warp.dev/linux/deb stable InRelease                                                                                
Hit:17 apt.pop-os.org/proprietary jammy InRelease                                                                                   
Hit:18 apt.pop-os.org/release jammy InRelease                                                                                       
Hit:19 packages.microsoft.com/debian/13/prod trixie InRelease                                                                      
Hit:20 packages.microsoft.com/repos/code stable InRelease                                                                          
Hit:21 apt.pop-os.org/ubuntu jammy InRelease                                                                                   
Hit:22 apt.pop-os.org/ubuntu jammy-security InRelease                                                                          
Hit:23 apt.pop-os.org/ubuntu jammy-updates InRelease                                                                           
Hit:24 packages.mozilla.org/apt mozilla InRelease                                                                     
Hit:25 apt.pop-os.org/ubuntu jammy-backports InRelease                                                                              
Hit:26 https://repo.waydro.id jammy InRelease                                                                                              
Hit:27 https://releases.warp.dev/linux/deb stable Release                                                                                  
Hit:29 deb.tuxedocomputers.com/ubuntu jammy InRelease                                                                              
Fetched 10.5 kB in 6s (1,617 B/s)                                                                                                          
Reading package lists... Done
Building dependency tree... Done
Reading state information... Done
All packages are up to date.
Reading package lists... Done
Building dependency tree... Done
Reading state information... Done
netcat-openbsd is already the newest version (1.218-4ubuntu1).
0 upgraded, 0 newly installed, 0 to remove and 0 not upgraded.
There are 2 choices for the alternative nc (providing /bin/nc).

  Selection    Path                 Priority   Status
------------------------------------------------------------
* 0            /bin/nc.openbsd       50        auto mode
  1            /bin/nc.openbsd       50        manual mode
  2            /bin/nc.traditional   10        manual mode

Press <enter> to keep the current choice[*], or type selection number: 
loufogle@pop-os ~> sudo apt update
                   sudo apt install -y netcat-openbsd
                   sudo update-alternatives --config nc

Get:1 cli.github.com/packages stable InRelease [3,917 B]
Ign:2 chrisjbawden.github.io/cockpit-dockermanager stable InRelease                                                                
Hit:3 apt.pop-os.org/proprietary jammy InRelease                                                                                    
Hit:4 chrisjbawden.github.io/cockpit-dockermanager stable Release                                                                  
Hit:5 deb.nodesource.com/node_20.x nodistro InRelease                                                                              
Ign:6 https://releases.warp.dev/linux/deb stable InRelease                                                                                 
Get:7 pkgs.tailscale.com/stable/ubuntu jammy InRelease                                                                             
Hit:8 repo.45drives.com/enterprise/ubuntu jammy InRelease                                                                          
Ign:9 chrisjbawden.github.io/cockpit-dockermanager stable Release.gpg                                                              
Hit:10 deb.opera.com/opera-gx-stable stable InRelease                                                                              
Hit:11 apt.pop-os.org/release jammy InRelease                                                                                       
Hit:12 packages.microsoft.com/debian/13/prod trixie InRelease                                                                      
Hit:13 packages.microsoft.com/repos/code stable InRelease                                                                          
Hit:14 apt.pop-os.org/ubuntu jammy InRelease                                                                                        
Hit:15 apt.pop-os.org/ubuntu jammy-security InRelease                              
Hit:16 packages.mozilla.org/apt mozilla InRelease                                 
Hit:17 ppa.launchpadcontent.net/alex-p/tesseract-ocr5/ubuntu jammy InRelease      
Hit:18 apt.pop-os.org/ubuntu jammy-updates InRelease         
Hit:19 deb.tuxedocomputers.com/ubuntu jammy InRelease       
Hit:20 apt.pop-os.org/ubuntu jammy-backports InRelease       
Hit:21 ppa.launchpadcontent.net/ar-lex/gamescope/ubuntu jammy InRelease
Hit:22 ppa.launchpadcontent.net/gerardpuig/ppa/ubuntu jammy InRelease
Hit:23 ppa.launchpadcontent.net/michel-slm/distrobox/ubuntu jammy InRelease
Hit:24 ppa.launchpadcontent.net/tomtomtom/woeusb/ubuntu jammy InRelease
Hit:25 ppa.launchpadcontent.net/ubuntu-toolchain-r/test/ubuntu jammy InRelease
Hit:26 ppa.launchpadcontent.net/zhangsongcui3371/fastfetch/ubuntu jammy InRelease
Hit:27 https://repo.waydro.id jammy InRelease     
Hit:28 https://releases.warp.dev/linux/deb stable Release                                                                                  
Fetched 10.5 kB in 6s (1,672 B/s)                                                                                                          
Reading package lists... Done
Building dependency tree... Done
Reading state information... Done
All packages are up to date.
Reading package lists... Done
Building dependency tree... Done
Reading state information... Done
netcat-openbsd is already the newest version (1.218-4ubuntu1).
0 upgraded, 0 newly installed, 0 to remove and 0 not upgraded.
There are 2 choices for the alternative nc (providing /bin/nc).

  Selection    Path                 Priority   Status
------------------------------------------------------------
* 0            /bin/nc.openbsd       50        auto mode
  1            /bin/nc.openbsd       50        manual mode
  2            /bin/nc.traditional   10        manual mode

Press <enter> to keep the current choice[*], or type selection number: 1
loufogle@pop-os ~> sudo apt update
                   sudo apt install -y netcat-openbsd
                   sudo update-alternatives --config nc

Ign:1 chrisjbawden.github.io/cockpit-dockermanager stable InRelease
Hit:2 chrisjbawden.github.io/cockpit-dockermanager stable Release                                                                  
Hit:3 deb.nodesource.com/node_20.x nodistro InRelease                                                                              
Ign:4 https://releases.warp.dev/linux/deb stable InRelease                                                                                 
Hit:5 deb.opera.com/opera-gx-stable stable InRelease                                                                               
Get:6 cli.github.com/packages stable InRelease [3,917 B]                                                                           
Hit:7 apt.pop-os.org/proprietary jammy InRelease                                                                                    
Ign:8 chrisjbawden.github.io/cockpit-dockermanager stable Release.gpg                                                              
Hit:9 repo.45drives.com/enterprise/ubuntu jammy InRelease                                                                          
Get:10 pkgs.tailscale.com/stable/ubuntu jammy InRelease                                                                            
Hit:11 apt.pop-os.org/release jammy InRelease                                                                                       
Hit:12 apt.pop-os.org/ubuntu jammy InRelease                                                                                        
Hit:13 apt.pop-os.org/ubuntu jammy-security InRelease                                                                               
Hit:14 apt.pop-os.org/ubuntu jammy-updates InRelease                                                                                
Hit:15 apt.pop-os.org/ubuntu jammy-backports InRelease                                                                              
Hit:16 packages.microsoft.com/debian/13/prod trixie InRelease                                                                      
Hit:17 packages.microsoft.com/repos/code stable InRelease                                                                          
Hit:18 ppa.launchpadcontent.net/alex-p/tesseract-ocr5/ubuntu jammy InRelease                                                       
Hit:19 ppa.launchpadcontent.net/ar-lex/gamescope/ubuntu jammy InRelease                                                       
Hit:20 packages.mozilla.org/apt mozilla InRelease                                                                     
Hit:21 ppa.launchpadcontent.net/gerardpuig/ppa/ubuntu jammy InRelease                                                 
Hit:22 https://repo.waydro.id jammy InRelease                                                          
Hit:23 ppa.launchpadcontent.net/michel-slm/distrobox/ubuntu jammy InRelease                                                        
Hit:24 ppa.launchpadcontent.net/tomtomtom/woeusb/ubuntu jammy InRelease                                                            
Hit:25 https://releases.warp.dev/linux/deb stable Release                                                                                  
Hit:26 deb.tuxedocomputers.com/ubuntu jammy InRelease                                                                              
Hit:28 ppa.launchpadcontent.net/ubuntu-toolchain-r/test/ubuntu jammy InRelease                                                     
Hit:29 ppa.launchpadcontent.net/zhangsongcui3371/fastfetch/ubuntu jammy InRelease                                                  
Fetched 10.5 kB in 7s (1,595 B/s)                                                                                                          
Reading package lists... Done
Building dependency tree... Done
Reading state information... Done
All packages are up to date.
Reading package lists... Done
Building dependency tree... Done
Reading state information... Done
netcat-openbsd is already the newest version (1.218-4ubuntu1).
0 upgraded, 0 newly installed, 0 to remove and 0 not upgraded.
There are 2 choices for the alternative nc (providing /bin/nc).

  Selection    Path                 Priority   Status
------------------------------------------------------------
  0            /bin/nc.openbsd       50        auto mode
* 1            /bin/nc.openbsd       50        manual mode
  2            /bin/nc.traditional   10        manual mode

Press <enter> to keep the current choice[*], or type selection number: 2
update-alternatives: using /bin/nc.traditional to provide /bin/nc (nc) in manual mode
loufogle@pop-os ~> 

If you switched nc to traditional and libvirt still errors with nc: invalid option -- 'U', then libvirt is still invoking the wrong nc (or a different helper binary), or your libvirt transport is configured to use a netcat mode that’s incompatible with the transport.

Do this last check on .204:

    Confirm what virsh sees for nc:

fish

which -a nc

    Confirm traditional nc behavior with -U:

fish

nc -U -h 2>&1 | head -n 3

    Re-run virsh:

fish

virsh -c 'qemu+ssh://root@192.168.12.242/system' list --all

If step (2) shows “invalid option -- 'U'”, then you should switch nc back to openbsd (option 0/1). Run:
fish

sudo update-alternatives --config nc

pick /bin/nc.openbsd, then re-run virsh.

loufogle@pop-os ~> which -a nc

/usr/bin/nc
/bin/nc

Switch nc back to OpenBSD (since libvirt is choking on -U under traditional).

Run on .204:
fish

sudo update-alternatives --config nc

Select /bin/nc.openbsd (option 0 or 1 as shown earlier).

Then retry:
fish

virsh -c 'qemu+ssh://root@192.168.

loufogle@pop-os ~/n/p/nexus-terminal (feat/v0.6.0-ring3-shell)> sudo update-alternatives --config nc

[sudo] password for loufogle: 
There are 2 choices for the alternative nc (providing /bin/nc).

  Selection    Path                 Priority   Status
------------------------------------------------------------
  0            /bin/nc.openbsd       50        auto mode
  1            /bin/nc.openbsd       50        manual mode
* 2            /bin/nc.traditional   10        manual mode

Press <enter> to keep the current choice[*], or type selection number: 0
update-alternatives: using /bin/nc.openbsd to provide /bin/nc (nc) in auto mode
loufogle@pop-os ~/n/p/nexus-terminal (feat/v0.6.0-ring3-shell)> virsh -c 'qemu+ssh://root@192.168.12.242/system' list --all

error: failed to connect to the hypervisor
error: End of file while reading data: nc: invalid option -- 'U'
nc -h for help: Input/output error

Ok—so even with nc set back to OpenBSD, libvirt is still executing a different nc for the transport.

Next: reinstall/realign libvirt clients so they don’t ship their own netcat transport helper.

On .204 run:
fish

sudo nala install -y --reinstall libvirt-clients libvirt-daemon-system
sudo systemctl restart libvirtd 2>/dev/null || true

Then run:
fish

virsh -c 'qemu+ssh://root@192.168.12.242/system' list --all

If it still fails, I need the exact stack of what libvirt runs—run this on .204 and paste the last ~80 lines:
fish

LIBVIRT_LOG_OUTPUTS=1 LIBVIRT_LOG_FILTERS=1 LIBVIRT_LOG_LEVEL=1 virsh -c 'qemu+ssh://root@192.168.12.242/system' list --all 2>&1 | tail -n 120

loufogle@pop-os ~/n/p/nexus-terminal (feat/v0.6.0-ring3-shell)> sudo nala install -y --reinstall libvirt-clients libvirt-daemon-system
                                                                sudo systemctl restart libvirtd 2>/dev/null || true

Usage: nala install [OPTIONS] PKGS ...

Error: No such option: --reinstall Did you mean --install-suggests?

Pop!\_OS (apt) doesn’t use --reinstall there—use reinstall with apt.

On .204 run:

fish

sudo nala install -y --force-yes libvirt-clients libvirt-daemon-system

If that complains, do this instead:

fish

sudo apt-get update
sudo apt-get install --reinstall -y libvirt-clients libvirt-daemon-system

Then restart:
fish

sudo systemctl restart libvirtd 2>/dev/null || true

Finally retry:
fish

virsh -c 'qemu+ssh://root@192.168.12.242/system' list --all

If it still fails, paste the output of:
fish

virsh -c 'qemu+ssh://root@192.168.12.242/system' -d 4 list --all 2>&1 | tail -n 120

loufogle@pop-os ~/n/p/nexus-terminal (feat/v0.6.0-ring3-shell)> sudo systemctl restart libvirtd 2>/dev/null || true

loufogle@pop-os ~/n/p/nexus-terminal (feat/v0.6.0-ring3-shell)> virsh -c 'qemu+ssh://root@192.168.12.242/system' list --all

error: failed to connect to the hypervisor
error: End of file while reading data: nc: invalid option -- 'U'
nc -h for help: Input/output error
loufogle@pop-os ~/n/p/nexus-terminal (feat/v0.6.0-ring3-shell) [1]> virsh -c 'qemu+ssh://root@192.168.12.242/system' -d 4 list --all 2>&1 | tail -n 120

error: failed to connect to the hypervisor
error: End of file while reading data: nc: invalid option -- 'U'
nc -h for help: Input/output error

Then libvirt on .204 is using an nc that still doesn’t accept -U (even if /usr/bin/nc.openbsd does). Let’s find exactly which nc binary virsh is invoking.

Run this on .204:

fish

virsh -c 'qemu+ssh://root@192.168.12.242/system' -d 4 list --all 2>&1 | grep -E 'nc|netcat|transport' | tail -n 50

    If that prints nothing, run strace for the failing virsh to catch the execve of nc:

fish

sudo strace -f -e execve -s 200 virsh -c 'qemu+ssh://root@192.168.12.242/system' list --all 2>&1 | grep -E 'nc' | tail -n 50

Paste outputs from (1) and/or (2). Then we’ll adjust PATH or the specific libvirt transport setting so it uses /usr/bin/nc.openbsd.

loufogle@pop-os ~/n/p/nexus-terminal (feat/v0.6.0-ring3-shell)> virsh -c 'qemu+ssh://root@192.168.12.242/system' -d 4 list --all 2>&1 | grep -E 'nc|netcat|transport' | tail -n 50

error: End of file while reading data: nc: invalid option -- 'U'
nc -h for help: Input/output error

We can fix this by avoiding the libvirt “ssh+nc” transport entirely: use libvirt over SSH without nc by switching the remote URI to the direct SSH transport libvirt supports (qemu+ssh already uses it, but we’ll make sure libvirt doesn’t try to use Unix-socket forwarding).

First, confirm the exact libvirt version/transport support on .204:
fish

virsh --version
libvirtd --version

Then try connecting using qemu+ssh but explicitly telling virsh to use the system URI scheme that doesn’t rely on libvirt’s “-U” helper by switching to the TCP transport (libvirt listens TCP on tiamat). We’ll enable that on tiamat if needed.

So do this on tiamat (.242):

    Check if libvirt TCP is already enabled:

bash

sudo ss -ltnp | grep 16509 || true
sudo ss -ltnp | grep 5900 || true

    If nothing on 16509 (default libvirt TCP), enable listening:

bash

sudo sed -i 's/^#\?listen_tls = .*/listen_tls = 0/' /etc/libvirt/libvirtd.conf
sudo sed -i 's/^#\?listen_tcp = .*/listen_tcp = 1/' /etc/libvirt/libvirtd.conf
sudo systemctl restart libvirtd

    Ensure firewall allows 16509/tcp (from .204):

bash

sudo ufw allow from 192.168.12.0/24 to any port 16509 proto tcp

Then on .204, use TCP URI:
fish

virsh -c 'qemu+tcp://root@192.168.12.242/system' list --all

If you’d rather not enable TCP, tell me your OS on tiamat and whether you already have firewalld/ufw—I’ll tailor the exact commands.

loufogle@pop-os ~/n/p/nexus-terminal (feat/v0.6.0-ring3-shell)> virsh --version
                                                                libvirtd --version

8.0.0
libvirtd (libvirt) 8.0.0
loufogle@pop-os ~/n/p/nexus-terminal (feat/v0.6.0-ring3-shell)> sudo ss -ltnp | grep 16509 || true
                                                                sudo ss -ltnp | grep 5900 || true

loufogle@pop-os ~/n/p/nexus-terminal (feat/v0.6.0-ring3-shell)> sudo sed -i 's/^#\?listen_tls = .*/listen_tls = 0/' /etc/libvirt/libvirtd.conf
                                                                sudo sed -i 's/^#\?listen_tcp = .*/listen_tcp = 1/' /etc/libvirt/libvirtd.conf
                                                                sudo systemctl restart libvirtd

loufogle@pop-os ~/n/p/nexus-terminal (feat/v0.6.0-ring3-shell)> sudo ufw allow from 192.168.12.0/24 to any port 16509 proto tcp

Rule added
loufogle@pop-os ~/n/p/nexus-terminal (feat/v0.6.0-ring3-shell)> virsh -c 'qemu+tcp://root@192.168.12.242/system' list --all

error: failed to connect to the hypervisor
error: unable to connect to server at '192.168.12.242:16509': Connection refused

You enabled TCP, but libvirt is still not listening on 16509, so connection is refused.

On tiamat (.242) run these:

bash

sudo ss -ltnp | grep -E '16509|libvirt' || true

    Check what libvirtd is listening/configured:

bash

sudo grep -nE 'listen_(tls|tcp)|auth_|tcp_' /etc/libvirt/libvirtd.conf

    See if libvirtd failed to restart:

bash

sudo systemctl status libvirtd --no-pager | tail -n 80

Paste outputs of (1)-(3). Then I’ll tell you exactly what to change so it listens on 16509 and accepts connections from .204.

loufogle@pop-os ~/n/p/nexus-terminal (feat/v0.6.0-ring3-shell)> sudo ss -ltnp | grep -E '16509|libvirt' || true

loufogle@pop-os ~/n/p/nexus-terminal (feat/v0.6.0-ring3-shell)> sudo grep -nE 'listen_(tls|tcp)|auth_|tcp_' /etc/libvirt/libvirtd.conf

20:listen_tls = 0
34:listen_tcp = 1
52:#tcp_port = "16509"
163:auth_unix_ro = "none"
169:# thus only allowing root user to connect, and 'auth_unix_rw'
174:# allows any user to connect and 'auth_unix_rw' will default
179:auth_unix_rw = "none"
187:#auth_tcp = "sasl"
196:#auth_tls = "none"
204:#tcp_min_ssf = 112
loufogle@pop-os ~/n/p/nexus-terminal (feat/v0.6.0-ring3-shell)> sudo systemctl status libvirtd --no-pager | tail -n 80

● libvirtd.service - Virtualization daemon
     Loaded: loaded (/lib/systemd/system/libvirtd.service; enabled; vendor preset: enabled)
     Active: active (running) since Sun 2026-06-28 16:20:14 EDT; 2min 1s ago
TriggeredBy: ● libvirtd-ro.socket
             ● libvirtd-admin.socket
             ● libvirtd.socket
       Docs: man:libvirtd(8)
             libvirt.org
   Main PID: 1445283 (libvirtd)
      Tasks: 21 (limit: 32768)
     Memory: 57.4M
        CPU: 373ms
     CGroup: /system.slice/libvirtd.service
             ├─  14146 /usr/sbin/dnsmasq --conf-file=/var/lib/libvirt/dnsmasq/default.conf --leasefile-ro --dhcp-script=/usr/lib/libvirt/libvirt_leaseshelper
             ├─  14147 /usr/sbin/dnsmasq --conf-file=/var/lib/libvirt/dnsmasq/default.conf --leasefile-ro --dhcp-script=/usr/lib/libvirt/libvirt_leaseshelper
             └─1445283 /usr/sbin/libvirtd

Jun 28 16:20:15 pop-os libvirtd[1445283]: ignoring socket '/tmp/steam_chrome_overlay_uid1000_spid385394'
Jun 28 16:20:15 pop-os libvirtd[1445283]: ignoring FIFO '/tmp/ad_gevt_qipcgwevt_13924_13924_6'
Jun 28 16:20:15 pop-os libvirtd[1445283]: ignoring FIFO '/tmp/ad_gevt_qipcgwevt_13924_13924_17'
Jun 28 16:20:15 pop-os libvirtd[1445283]: ignoring socket '/tmp/kdsingleapp-loufogle-3-org.flameshot.Flameshot'
Jun 28 16:20:15 pop-os libvirtd[1445283]: ignoring FIFO '/tmp/ad_gevt_quiver_anynet_quit_event'
Jun 28 16:20:15 pop-os libvirtd[1445283]: ignoring FIFO '/tmp/ad_gevt_qipcgwevt_13924_13924_16'
Jun 28 16:20:15 pop-os libvirtd[1445283]: ignoring FIFO '/tmp/ad_gevt_qipcgwevt_13924_13924_15'
Jun 28 16:20:15 pop-os libvirtd[1445283]: ignoring FIFO '/tmp/ad_gevt_qipcgwevt_quiver_anynet_service_client_20397_20444_0'
Jun 28 16:20:15 pop-os libvirtd[1445283]: cannot stat file '/tmp/.mount_SunshiAApAjI': Permission denied
Jun 28 16:20:15 pop-os libvirtd[1445283]: internal error: Failed to restart storage pool 'tmp-1': cannot stat file '/tmp/.mount_SunshiAApAjI': Permission denied

listen_tcp = 1 isn’t enough—libvirtd is actively failing to restart a storage pool due to /tmp/.mount_* permission errors, and that can prevent libvirtd from successfully bringing up its TCP listener.

On tiamat (.242) do this:

    See the real libvirtd error and whether the TCP socket is bound:

bash

sudo journalctl -u libvirtd --since "10 min ago" | tail -n 200
sudo ss -ltnp | grep 16509 || true

    Stop libvirt and remove the broken tmp mount pool, then restart:

bash

sudo systemctl stop libvirtd
sudo virsh pool-list --all || true
sudo virsh pool-destroy tmp-1 2>/dev/null || true
sudo rm -f /tmp/.mount_* 2>/dev/null || true
sudo systemctl start libvirtd
sudo systemctl status libvirtd --no-pager | tail -n 80

    Retry from .204:

fish

virsh -c 'qemu+tcp://root@192.168.12.242/system' list --all

If you paste the output of step (1) (journalctl ... tail and ss -ltnp | grep 16509), I’ll tell you the exact next command for your specific failing pool (tmp-1 vs something else).


    PDF

Your libvirtd isn’t actually serving TCP because it’s failing to restart the tmp-1 storage pool: cannot stat file '/tmp/.mount_...': Permission denied. Fix that first, then re-test 16509.

On tiamat (.242) do:

bash

sudo ls -ld /tmp
sudo ls -l /tmp/.mount_* 2>/dev/null || true
sudo stat /tmp/.mount_SunshiAApAjI 2>/dev/null || true

    Reset the broken /tmp mount pool files (since you’re already root on that host):

bash

sudo systemctl stop libvirtd
sudo rm -f /tmp/.mount_* 2>/dev/null || true
sudo systemctl start libvirtd
sudo systemctl status libvirtd --no-pager | tail -n 50

    Confirm TCP listener:

bash

sudo ss -ltnp | grep -E '16509|libvirt' || true

If you still don’t see 16509 listening, paste the output of:
bash

sudo journalctl -u libvirtd --since "10 min ago" | tail -n 120

Then re-try from .204:
fish

virsh -c 'qemu+tcp://root@192.168.12.242/system

loufogle@pop-os ~/n/p/nexus-terminal (feat/v0.6.0-ring3-shell)> sudo ls -ld /tmp
                                                                sudo ls -l /tmp/.mount_* 2>/dev/null || true
                                                                sudo stat /tmp/.mount_SunshiAApAjI 2>/dev/null || true

drwxrwxrwt 31 root root 36864 Jun 28 16:28 /tmp
loufogle@pop-os ~/n/p/nexus-terminal (feat/v0.6.0-ring3-shell)> sudo systemctl stop libvirtd
                                                                sudo rm -f /tmp/.mount_* 2>/dev/null || true
                                                                sudo systemctl start libvirtd
                                                                sudo systemctl status libvirtd --no-pager | tail -n 50

Warning: Stopping libvirtd.service, but it can still be activated by:
  libvirtd-ro.socket
  libvirtd-admin.socket
  libvirtd.socket
● libvirtd.service - Virtualization daemon
     Loaded: loaded (/lib/systemd/system/libvirtd.service; enabled; vendor preset: enabled)
     Active: active (running) since Sun 2026-06-28 16:28:44 EDT; 10ms ago
TriggeredBy: ● libvirtd-ro.socket
             ● libvirtd-admin.socket
             ● libvirtd.socket
       Docs: man:libvirtd(8)
             libvirt.org
   Main PID: 1454120 (libvirtd)
      Tasks: 22 (limit: 32768)
     Memory: 52.0M
        CPU: 22ms
     CGroup: /system.slice/libvirtd.service
             ├─  14146 /usr/sbin/dnsmasq --conf-file=/var/lib/libvirt/dnsmasq/default.conf --leasefile-ro --dhcp-script=/usr/lib/libvirt/libvirt_leaseshelper
             ├─  14147 /usr/sbin/dnsmasq --conf-file=/var/lib/libvirt/dnsmasq/default.conf --leasefile-ro --dhcp-script=/usr/lib/libvirt/libvirt_leaseshelper
             ├─1454120 /usr/sbin/libvirtd
             └─1454145 /usr/sbin/iptables -w --table filter --list-rules

Jun 28 16:28:44 pop-os systemd[1]: Starting Virtualization daemon...
Jun 28 16:28:44 pop-os systemd[1]: Started Virtualization daemon.
loufogle@pop-os ~/n/p/nexus-terminal (feat/v0.6.0-ring3-shell)> sudo ss -ltnp | grep -E '16509|libvirt' || true

loufogle@pop-os ~/n/p/nexus-terminal (feat/v0.6.0-ring3-shell)> sudo journalctl -u libvirtd --since "10 min ago" | tail -n 120

Jun 28 16:20:14 pop-os systemd[1]: Stopping Virtualization daemon...
Jun 28 16:20:14 pop-os systemd[1]: libvirtd.service: Deactivated successfully.
Jun 28 16:20:14 pop-os systemd[1]: libvirtd.service: Unit process 14146 (dnsmasq) remains running after unit stopped.
Jun 28 16:20:14 pop-os systemd[1]: libvirtd.service: Unit process 14147 (dnsmasq) remains running after unit stopped.
Jun 28 16:20:14 pop-os systemd[1]: Stopped Virtualization daemon.
Jun 28 16:20:14 pop-os systemd[1]: libvirtd.service: Found left-over process 14146 (dnsmasq) in control group while starting unit. Ignoring.
Jun 28 16:20:14 pop-os systemd[1]: This usually indicates unclean termination of a previous run, or service implementation deficiencies.
Jun 28 16:20:14 pop-os systemd[1]: libvirtd.service: Found left-over process 14147 (dnsmasq) in control group while starting unit. Ignoring.
Jun 28 16:20:14 pop-os systemd[1]: This usually indicates unclean termination of a previous run, or service implementation deficiencies.
Jun 28 16:20:14 pop-os systemd[1]: Starting Virtualization daemon...
Jun 28 16:20:14 pop-os systemd[1]: Started Virtualization daemon.
Jun 28 16:20:15 pop-os dnsmasq[14146]: read /etc/hosts - 3 names
Jun 28 16:20:15 pop-os dnsmasq[14146]: read /var/lib/libvirt/dnsmasq/default.addnhosts - 0 names
Jun 28 16:20:15 pop-os dnsmasq-dhcp[14146]: read /var/lib/libvirt/dnsmasq/default.hostsfile
Jun 28 16:20:15 pop-os libvirtd[1445283]: libvirt version: 8.0.0, package: 1ubuntu7.17 (Seyeong Kim <seyeong.kim@canonical.com> Thu, 21 May 2026 03:50:35 +0000)
Jun 28 16:20:15 pop-os libvirtd[1445283]: hostname: pop-os
Jun 28 16:20:15 pop-os libvirtd[1445283]: ignoring dangling symlink '/home/loufogle/.lr-atari800.cfg'
Jun 28 16:20:15 pop-os libvirtd[1445283]: ignoring dangling symlink '/home/loufogle/.steampath'
Jun 28 16:20:15 pop-os libvirtd[1445283]: ignoring FIFO '/tmp/ad_gevt_qipcgwevt_13924_13924_14'
Jun 28 16:20:15 pop-os libvirtd[1445283]: ignoring FIFO '/tmp/ad_gevt_quiver_anynet_kill_event'
Jun 28 16:20:15 pop-os libvirtd[1445283]: ignoring FIFO '/tmp/ad_gevt_qipcgwevt_13924_13924_5'
Jun 28 16:20:15 pop-os libvirtd[1445283]: ignoring FIFO '/tmp/ad_gevt_qipcgwevt_20397_20445_0'
Jun 28 16:20:15 pop-os libvirtd[1445283]: ignoring socket '/tmp/steam_chrome_overlay_uid1000_spid385394'
Jun 28 16:20:15 pop-os libvirtd[1445283]: ignoring FIFO '/tmp/ad_gevt_qipcgwevt_13924_13924_6'
Jun 28 16:20:15 pop-os libvirtd[1445283]: ignoring FIFO '/tmp/ad_gevt_qipcgwevt_13924_13924_17'
Jun 28 16:20:15 pop-os libvirtd[1445283]: ignoring socket '/tmp/kdsingleapp-loufogle-3-org.flameshot.Flameshot'
Jun 28 16:20:15 pop-os libvirtd[1445283]: ignoring FIFO '/tmp/ad_gevt_quiver_anynet_quit_event'
Jun 28 16:20:15 pop-os libvirtd[1445283]: ignoring FIFO '/tmp/ad_gevt_qipcgwevt_13924_13924_16'
Jun 28 16:20:15 pop-os libvirtd[1445283]: ignoring FIFO '/tmp/ad_gevt_qipcgwevt_13924_13924_15'
Jun 28 16:20:15 pop-os libvirtd[1445283]: ignoring FIFO '/tmp/ad_gevt_qipcgwevt_quiver_anynet_service_client_20397_20444_0'
Jun 28 16:20:15 pop-os libvirtd[1445283]: cannot stat file '/tmp/.mount_SunshiAApAjI': Permission denied
Jun 28 16:20:15 pop-os libvirtd[1445283]: internal error: Failed to restart storage pool 'tmp-1': cannot stat file '/tmp/.mount_SunshiAApAjI': Permission denied
Jun 28 16:23:57 pop-os systemd[1]: Stopping Virtualization daemon...
Jun 28 16:23:57 pop-os systemd[1]: libvirtd.service: Deactivated successfully.
Jun 28 16:23:57 pop-os systemd[1]: libvirtd.service: Unit process 14146 (dnsmasq) remains running after unit stopped.
Jun 28 16:23:57 pop-os systemd[1]: libvirtd.service: Unit process 14147 (dnsmasq) remains running after unit stopped.
Jun 28 16:23:57 pop-os systemd[1]: Stopped Virtualization daemon.
Jun 28 16:23:57 pop-os systemd[1]: libvirtd.service: Found left-over process 14146 (dnsmasq) in control group while starting unit. Ignoring.
Jun 28 16:23:57 pop-os systemd[1]: This usually indicates unclean termination of a previous run, or service implementation deficiencies.
Jun 28 16:23:57 pop-os systemd[1]: libvirtd.service: Found left-over process 14147 (dnsmasq) in control group while starting unit. Ignoring.
Jun 28 16:23:57 pop-os systemd[1]: This usually indicates unclean termination of a previous run, or service implementation deficiencies.
Jun 28 16:23:57 pop-os systemd[1]: Starting Virtualization daemon...
Jun 28 16:23:58 pop-os systemd[1]: Started Virtualization daemon.
Jun 28 16:23:58 pop-os dnsmasq[14146]: read /etc/hosts - 3 names
Jun 28 16:23:58 pop-os dnsmasq[14146]: read /var/lib/libvirt/dnsmasq/default.addnhosts - 0 names
Jun 28 16:23:58 pop-os dnsmasq-dhcp[14146]: read /var/lib/libvirt/dnsmasq/default.hostsfile
Jun 28 16:23:58 pop-os libvirtd[1449059]: libvirt version: 8.0.0, package: 1ubuntu7.17 (Seyeong Kim <seyeong.kim@canonical.com> Thu, 21 May 2026 03:50:35 +0000)
Jun 28 16:23:58 pop-os libvirtd[1449059]: hostname: pop-os
Jun 28 16:23:58 pop-os libvirtd[1449059]: ignoring dangling symlink '/home/loufogle/.lr-atari800.cfg'
Jun 28 16:23:58 pop-os libvirtd[1449059]: ignoring dangling symlink '/home/loufogle/.steampath'
Jun 28 16:23:58 pop-os libvirtd[1449059]: ignoring FIFO '/tmp/ad_gevt_qipcgwevt_13924_13924_14'
Jun 28 16:23:58 pop-os libvirtd[1449059]: ignoring FIFO '/tmp/ad_gevt_quiver_anynet_kill_event'
Jun 28 16:23:58 pop-os libvirtd[1449059]: ignoring FIFO '/tmp/ad_gevt_qipcgwevt_13924_13924_5'
Jun 28 16:23:58 pop-os libvirtd[1449059]: ignoring FIFO '/tmp/ad_gevt_qipcgwevt_20397_20445_0'
Jun 28 16:23:58 pop-os libvirtd[1449059]: ignoring socket '/tmp/steam_chrome_overlay_uid1000_spid385394'
Jun 28 16:23:58 pop-os libvirtd[1449059]: ignoring FIFO '/tmp/ad_gevt_qipcgwevt_13924_13924_6'
Jun 28 16:23:58 pop-os libvirtd[1449059]: ignoring FIFO '/tmp/ad_gevt_qipcgwevt_13924_13924_17'
Jun 28 16:23:58 pop-os libvirtd[1449059]: ignoring socket '/tmp/kdsingleapp-loufogle-3-org.flameshot.Flameshot'
Jun 28 16:23:58 pop-os libvirtd[1449059]: ignoring FIFO '/tmp/ad_gevt_quiver_anynet_quit_event'
Jun 28 16:23:58 pop-os libvirtd[1449059]: ignoring FIFO '/tmp/ad_gevt_qipcgwevt_13924_13924_16'
Jun 28 16:23:58 pop-os libvirtd[1449059]: ignoring FIFO '/tmp/ad_gevt_qipcgwevt_13924_13924_15'
Jun 28 16:23:58 pop-os libvirtd[1449059]: ignoring FIFO '/tmp/ad_gevt_qipcgwevt_quiver_anynet_service_client_20397_20444_0'
Jun 28 16:23:58 pop-os libvirtd[1449059]: cannot stat file '/tmp/.mount_SunshiAApAjI': Permission denied
Jun 28 16:23:58 pop-os libvirtd[1449059]: internal error: Failed to restart storage pool 'tmp-1': cannot stat file '/tmp/.mount_SunshiAApAjI': Permission denied
Jun 28 16:28:44 pop-os systemd[1]: Stopping Virtualization daemon...
Jun 28 16:28:44 pop-os systemd[1]: libvirtd.service: Deactivated successfully.
Jun 28 16:28:44 pop-os systemd[1]: libvirtd.service: Unit process 14146 (dnsmasq) remains running after unit stopped.
Jun 28 16:28:44 pop-os systemd[1]: libvirtd.service: Unit process 14147 (dnsmasq) remains running after unit stopped.
Jun 28 16:28:44 pop-os systemd[1]: Stopped Virtualization daemon.
Jun 28 16:28:44 pop-os systemd[1]: libvirtd.service: Found left-over process 14146 (dnsmasq) in control group while starting unit. Ignoring.
Jun 28 16:28:44 pop-os systemd[1]: This usually indicates unclean termination of a previous run, or service implementation deficiencies.
Jun 28 16:28:44 pop-os systemd[1]: libvirtd.service: Found left-over process 14147 (dnsmasq) in control group while starting unit. Ignoring.
Jun 28 16:28:44 pop-os systemd[1]: This usually indicates unclean termination of a previous run, or service implementation deficiencies.
Jun 28 16:28:44 pop-os systemd[1]: Starting Virtualization daemon...
Jun 28 16:28:44 pop-os systemd[1]: Started Virtualization daemon.
Jun 28 16:28:44 pop-os dnsmasq[14146]: read /etc/hosts - 3 names
Jun 28 16:28:44 pop-os dnsmasq[14146]: read /var/lib/libvirt/dnsmasq/default.addnhosts - 0 names
Jun 28 16:28:44 pop-os dnsmasq-dhcp[14146]: read /var/lib/libvirt/dnsmasq/default.hostsfile
Jun 28 16:28:44 pop-os libvirtd[1454120]: libvirt version: 8.0.0, package: 1ubuntu7.17 (Seyeong Kim <seyeong.kim@canonical.com> Thu, 21 May 2026 03:50:35 +0000)
Jun 28 16:28:44 pop-os libvirtd[1454120]: hostname: pop-os
Jun 28 16:28:44 pop-os libvirtd[1454120]: ignoring dangling symlink '/home/loufogle/.lr-atari800.cfg'
Jun 28 16:28:44 pop-os libvirtd[1454120]: ignoring dangling symlink '/home/loufogle/.steampath'
Jun 28 16:28:44 pop-os libvirtd[1454120]: ignoring FIFO '/tmp/ad_gevt_qipcgwevt_13924_13924_14'
Jun 28 16:28:44 pop-os libvirtd[1454120]: ignoring FIFO '/tmp/ad_gevt_quiver_anynet_kill_event'
Jun 28 16:28:44 pop-os libvirtd[1454120]: ignoring FIFO '/tmp/ad_gevt_qipcgwevt_13924_13924_5'
Jun 28 16:28:44 pop-os libvirtd[1454120]: ignoring FIFO '/tmp/ad_gevt_qipcgwevt_20397_20445_0'
Jun 28 16:28:44 pop-os libvirtd[1454120]: ignoring socket '/tmp/steam_chrome_overlay_uid1000_spid385394'
Jun 28 16:28:44 pop-os libvirtd[1454120]: ignoring FIFO '/tmp/ad_gevt_qipcgwevt_13924_13924_6'
Jun 28 16:28:44 pop-os libvirtd[1454120]: ignoring FIFO '/tmp/ad_gevt_qipcgwevt_13924_13924_17'
Jun 28 16:28:44 pop-os libvirtd[1454120]: ignoring socket '/tmp/kdsingleapp-loufogle-3-org.flameshot.Flameshot'
Jun 28 16:28:44 pop-os libvirtd[1454120]: ignoring FIFO '/tmp/ad_gevt_quiver_anynet_quit_event'
Jun 28 16:28:44 pop-os libvirtd[1454120]: ignoring FIFO '/tmp/ad_gevt_qipcgwevt_13924_13924_16'
Jun 28 16:28:44 pop-os libvirtd[1454120]: ignoring FIFO '/tmp/ad_gevt_qipcgwevt_13924_13924_15'
Jun 28 16:28:44 pop-os libvirtd[1454120]: ignoring FIFO '/tmp/ad_gevt_qipcgwevt_quiver_anynet_service_client_20397_20444_0'
Jun 28 16:28:44 pop-os libvirtd[1454120]: cannot stat file '/tmp/.mount_markte0jGGKN': Permission denied
Jun 28 16:28:44 pop-os libvirtd[1454120]: internal error: Failed to restart storage pool 'tmp-1': cannot stat file '/tmp/.mount_markte0jGGKN': Permission denied
