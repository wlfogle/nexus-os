#!/bin/bash

# Color codes
YELLOW='\033[0;33m'
GREEN='\033[0;32m'
BLUE="\e[34m"
NC='\033[0m' # No Color
RED='\033[0;31m'

function f_print_header {
    echo -e "${YELLOW}##############>>>> ${GREEN}WELCOME ${NC}$(whoami)${GREEN} to POP OS CONFIGURATOR ${NC}v1 ${YELLOW}<<<<################"
    echo ""
    f_play_alert
}

function f_play_alert {
    paplay /usr/share/sounds/Pop/stereo/alert/alarm-clock-elapsed.oga || echo "Failed to play sound"
}

function f_change_net_name {
    interface=$(ip route | grep default | awk '{print $5}')
    ip_address=$(ip -o -f inet addr show "$interface" | awk '{print $4}')
    mac_address=$(ip link show "$interface" | awk '/ether/ {print $2}')
    echo -e "Your Default Physcial Network Interface name is: \n${BLUE}${interface}${NC}"
    echo -e "IP Address: ${BLUE}${ip_address}${NC}"
    # Check if the MAC address was found
    if [[ -z "$mac_address" ]]; then
        echo "No MAC address found for interface ${BLUE}$interface${NC}"
    else
        echo "The MAC address of ${BLUE}$interface${NC} is:${BLUE} $mac_address${NC}"
    fi
    echo ""
    f_play_alert
    echo -e "${GREEN}Change ${NC}$interface ${GREEN}network interface name to ${NC}eth0? ${GREEN}[y/N]${NC}"
    read -r -p "" response
    if [[ "$response" =~ ^([yY][eE][sS]|[yY])$ ]]; then
        sudo cat << EOF >> /etc/udev/rules.d/70-persistent-net.rules
Modify ATTR{address} with MAC of the network interface and NAME parameter with the new desired name.
SUBSYSTEM=="net", ACTION=="add", DRIVERS=="?*", ATTR{address}=="$mac_address", NAME="eth0"
EOF
        echo -e "${RED}ABOUT TO REBOOT${YELLOW} - re-run this script after reboot${NC}"
        sudo touch ~/net_changed.txt
        f_play_alert
        read
        sudo systemctl reboot
        exit
    else
        sudo touch ~/net_changed.txt
    fi
}

function f_purge_crap {
    echo -e "${GREEN}Resize dock icon size (settings > desktop > dock - change to 44), and increase view to 200%, disable HiDPI enabled toggle, change desktop background${NC}"
    echo -e "${YELLOW}Remove show active hint, Border and Gaps both to ZERO"
    echo ""
    echo -e "${BLUE}Remove workspaces and applications buttons, enable top left workspace shortcut"
    read
    sudo apt purge firefox libreoffice-* geary -y
    echo -e "${GREEN}done${NC}"
    sudo apt autoremove -y && sudo apt update && sudo apt upgrade -y && sudo apt install -f
    echo -e "${GREEN}done${NC}"
    f_play_alert
}

function f_net_share {
    clear
    sudo apt install cifs-utils -y
    sudo mkdir -p /mnt/nas_share
    read -sp "Enter network share SERVER IP Address: " srvip
    read -sp "Enter network share  name: " srvshare
    sudo cat << EOF >> /etc/fstab
//$srvip/$srvshare /mnt/net_share cifs credentials=/etc/samba/credentials,iocharset=utf8,uid=1000,gid=1000 0 0
EOF
    f_play_alert
    read -sp "Enter your username: " ursname
    echo
    read -sp "Enter your password: " password
    echo
    echo -e "username="$ursname"\npassword="$password | sudo tee /etc/samba/credentials > /dev/null
    sudo chmod 600 /etc/samba/credentials
    sudo mount -a
    # Path to your SMB share's mount point
    MOUNT_POINT="/mnt/nas_share"
    BOOKMARK_NAME="NAS SMB Share"

    # Ensure the bookmarks file exists - this is for Nautilus file manager
    if [ ! -f ~/.config/gtk-3.0/bookmarks ]; then
        mkdir -p ~/.config/gtk-3.0
        touch ~/.config/gtk-3.0/bookmarks
    fi

    # Add the bookmark in Nautilus if it's not already there
    if ! grep -q "$MOUNT_POINT" ~/.config/gtk-3.0/bookmarks; then
        echo "file://$MOUNT_POINT $BOOKMARK_NAME" >> ~/.config/gtk-3.0/bookmarks
        echo "Bookmark added for $MOUNT_POINT"
    else
        echo "Bookmark for $MOUNT_POINT already exists"
    fi
    f_play_alert
    echo -e "${GREEN}done${NC}"
    read
}

function f_5_apps_inst {
        sudo apt update -y && sudo apt upgrade -y
        sudo apt install btop vlc nmap ufw code mkvtoolnix-gui -y
        sudo ufw enable
        code --install-extension formulahendry.docker-explorer
        code --install-extension ms-azuretools.vscode-docker
        code --install-extension ms-vscode.remote-explorer
        code --install-extension ms-python.python
        code --install-extension ms-vscode-remote.remote-ssh
        code --install-extension ms-vscode-remote.remote-containers
        code --install-extension esbenp.prettier-vscode
        code --install-extension foxundermoon.shell-format
        code --install-extension ms-python.debugpy
        code --install-extension ms-python.vscode-pylance
        code --install-extension ms-vscode-remote.remote-ssh-edit
        code --install-extension timonwong.shellcheck
        code --install-extension visualstudioexptteam.intellicode-api-usage-examples
        code --install-extension visualstudioexptteam.vscodeintellicode
        xdg-mime default vlc.desktop video/mp4
        xdg-mime default vlc.desktop video/x-matroska
        xdg-mime default vlc.desktop video/x-msvideo
        xdg-mime default vlc.desktop video/x-ms-wmv
        echo -e "${GREEN}done${NC}"
        f_play_alert
        read
}

function f_time_shift {
        clear
        echo -e "${RED}Install Timeshift? [N/Y?] ${NC}"
        f_play_alert
        read -r -p "" response
        if [[ "$response" =~ ^([yY][eE][sS]|[yY])$ ]]; then
            sudo add-apt-repository -y ppa:teejee2008/ppa -y
            echo -e "${GREEN}repo added${NC}"
            sudo apt update -y
            sudo apt install timeshift -y
            sudo timeshift-gtk
            echo -e "${GREEN}done${NC}"
            f_play_alert
            read
        fi
}

function f_warp_fish {
    clear
    echo -e "${RED}Download Warp deb install file from https://www.warp.dev/ before continuing${NC}"
    read
    sudo chmod +x ~/Downloads/warp-terminal_*_amd64.deb
    sudo dpkg -i ~/Downloads/warp-terminal_*_amd64.deb
    rm ~/Downloads/warp-terminal_*_amd64.deb
    echo ""
    echo -e "${GREEN}command completed - Run Warp, sign in as ${YELLOW}officeca ${GREEN}and click link in email authorisation, add to dock${NC}"
    read
    sudo apt-add-repository ppa:fish-shell/release-3 -y
    sudo apt update -y
    sudo apt install fish -y
    echo ""
    thus="$"
    sudo cat << EOF >> /usr/share/fish/config.fish
printf '\eP${thus}f{"hook": "SourcedRcFileForWarp", "value": { "shell": "fish" }}\x9c'
EOF
    echo ""
    echo -e "${GREEN}in WARP > Settings > Features : Startup Shell : change to > fish & SAVE"
    echo -e "${GREEN}Change startup directory to Home, and disable show changelog after update"
    echo -e "${RED}DO NOT Config terminal prompt in Warp but -"
    echo -e "${GREEN}run fish_config only if needed LATER${NC}"
    f_play_alert
    read
}

function f_dock_port_comp {
    echo -e "${RED}Install Docker, Portainer AND Compose? [y/N] ${NC}"
    f_play_alert
    read -r -p "" response
    if [[ "$response" =~ ^([yY][eE][sS]|[yY])$ ]]; then
        sudo apt install docker.io -y
        sudo docker volume create portainer_stuff
        sudo docker run -d -p 9443:9443 -p 8000:8000 --name portainer --restart=always -v /var/run/docker.sock:/var/run/docker.sock -v portainer_stuff:/data portainer/portainer-ce:latest
        sudo docker ps
        echo -e "${YELLOW}command completed - https://localhost:9443 - apply portainer different admin username password.  Save login details in Brave (hit enter rather than click save)${NC}"
        read
        sudo sysctl -w vm.max_map_count=262144
        sudo curl -L "https://github.com/docker/compose/releases/download/v2.12.2/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
        echo -e "${GREEN}command completed"
        echo -e "${NC}"
        sudo chmod +x /usr/local/bin/docker-compose
        docker-compose --version
        echo -e "${GREEN}command completed"
        f_play_alert
        read
    fi
}   

function f_6_apps {
    sudo apt update -y && sudo apt upgrade -y
    clear
    sudo apt install gparted gsmartcontrol neofetch grsync ttf-mscorefonts-installer -y
    sudo add-apt-repository ppa:wireshark-dev/stable -y
    sudo apt update -y
    echo -e "${RED}NEXT - Select YES to non-super users"
    read
    sudo apt install wireshark -y
    echo -e "${GREEN}done${NC}"
    sudo usermod -aG wireshark $(whoami)
        # download profiles from here https://github.com/amwalding/wireshark_profiles
    echo -e "${RED}Wireshark will ONLY work after reboot"
    echo -e "${GREEN}Add Wireshark icon to dock"
    f_play_alert
    read
}

function f_pwsafe_brave {
    sudo apt install passwordsafe -y
        #sudo curl -fsSLo /usr/share/keyrings/brave-browser-beta-archive-keyring.gpg https://brave-browser-apt-beta.s3.brave.com/brave-browser-beta-archive-keyring.gpg
        #echo "deb [signed-by=/usr/share/keyrings/brave-browser-beta-archive-keyring.gpg] https://brave-browser-apt-beta.s3.brave.com/ stable main"|sudo tee /etc/apt/sources.list.d/brave-browser-beta.list
        #sudo apt update -y
        #sudo apt install brave-browser-beta -y
    echo -e "${GREEN}command completed"
    f_play_alert
    read
}

function f_instructs {
        clear
        echo -e "${GREEN}Install Brave, WPS Office, Thunderbird, Bitwarden, Eddy (or gdebi), ${YELLOW}AND Startup Disk Creator AND Dropbox LIGHT VER from pop store"
        echo ""
        echo -e "${BLUE}Sign into dropbox when prompted with website - confirm in email 2fa."
        echo ""
        echo -e "${YELLOW}Drag Dropbox folder to sidebar on left in nautilus${NC}"
        echo ""
        echo -e "${GREEN}Pin Brave to favourites, and change search to Google, hide leo, hide wallet"
        echo -e "${GREEN}remove showing rewards in appearance"
        echo -e "${GREEN}add AddBlock plus and AdGuard Adblocker"
        echo -e "${GREEN}pin extensions to toolbar, install SPEED dial FVD and import SD2.txt in dropbox folder, and Bookmarks, and passwords in nas"
        echo ""
        hosty=$(hostname)
        f_play_alert
        echo -e "${YELLOW}Change computer name from ${RED}$hosty ${YELLOW}[Y/N]?"
        read -r -p "" response
        if [[ "$response" =~ ^([yY][eE][sS]|[yY])$ ]]; then
            read -sp "Enter new computer name: " compname
            sudo hostnamectl set-hostname $compname
        fi
        clear
        echo -e "${GREEN}https://www.virtualbox.org/wiki/Linux_Downloads select Ubuntu ver and download, install with Eddy${NC}"
        f_play_alert
        read
}

function f_wazuh {
    clear
    echo -e "${YELLOW}Install Wazuh Management Centre in Docker on this Computer? [Y/n?]"
    echo ""
    echo -e "${RED}ONLY INSTALL IF DOCKER IS INSTALLED"
    echo -e "${NC}"
    read -r -p "" response
    if [[ "$response" =~ ^([yY][eE][sS]|[yY])$ ]];
    then
        git clone https://github.com/wazuh/wazuh-docker.git -b v4.7.5
        echo -e "${GREEN}done"
        echo -e "${NC}"
        cd wazuh-docker
        cd single-node
        echo -e "${GREEN}change username and password for the Wazuh dashboard in the docker-compose.yml file"
        echo -e "${NC}"
        read
        sudo docker-compose -f generate-indexer-certs.yml run --rm generator
        sudo docker-compose up -d

        echo -e "${GREEN}will need 'sudo systemctl restart wazuh-agent' later, when deploying local agent"
        echo -e "${GREEN}paste xml config (in dropbox) into file within this folder /home/captain/wazuh-docker/single-node/config/wazuh_cluster/wazuh_manager.conf"
        echo -e "${NC}"
        read
        sudo systemctl status wazuh-agent
        echo "use this link to access Wazuh https:/localhost/"
        echo -e "${NC}"
        cd ~
        f_play_alert
        read
    fi
}

function f_nordvpn {
    clear
    sudo groupadd nordvpn
    sudo usermod -aG nordvpn $(whoami)
    sh <(curl -sSf https://downloads.nordcdn.com/apps/linux/install.sh)
    echo ""
    echo -e "${YELLOW}Login via app (top right)"
    echo ""
    echo -e "${GREEN}then copy (right click continue button and save url), EDIT (just have token) and paste & run below in a SEPARATE Terminal window${NC}"
    echo ""
    echo -e "sudo nordvpn login --callback '${RED}paste full link in here${NC}' ${GREEN}<<<< and then run this command in separate window before continuing"
    f_play_alert
    read
}

function f_connect_nord {
    echo "<<<< Attempting to connect to UK NordVPN >>>"
    nordvpn connect uk
    echo -e "${YELLOW}Successful? (remedy in separate term window before continuing)"
    read
    echo ""
    sudo nordvpn allowlist add subnet 192.168.0.0/16
    echo ""
    echo -e "${RED}nordvpn now allows local 192.168.0.0 traffic to split tunnel and not go through VPN"
    f_play_alert
    read
}

function f_startup {
    sudo cp ~/Documents/*.desktop ~/.config/autostart/
}

function f_starship {
    curl -sS https://starship.rs/install.sh | sh
    echo ""
    echo -e "${GREEN}Download a Font(s) available from https://nerdfonts.com"
    echo -e "${GREEN}About to copy fonts to local machine"
    read
    sudo cp ~/Documents/Fonts/*.ttf ~/.local/share/fonts/
    fc-cache -f -v > /dev/null
    echo -e "${GREEN}Go to Warp settings > appearance  - change font to Hack Nerd, Font size 16. Change theme to Dracula"
    read
    cp ~/Documents/starship.toml ~/.config/starship.toml
            #echo ""
            # echo -e "${GREEN}Add the following to the end of (file will open once you press enter) ''"
            #read
            #sudo nano ~/.config/fish/config.fish
    sudo cat << EOF >> ~/.config/fish/config.fish
starship init fish | source
alias man='tldr'
EOF
    starship init fish | source
    echo ""
    echo -e "${GREEN}Done"
    sudo apt install tldr -y
    sudo tldr --update
    #### Add alias to work outside of fish
    awk '/# some more ls aliases/ { print; print "alias man='\''tldr'\''"; next }1' ~/.bashrc > temp && mv temp ~/.bashrc
    echo -e "${GREEN}Done"
    f_play_alert
}

function f_monitor_apps {
    # following section allows login screen to appear on screen 2 (external display only) solely
    echo -e "${RED}ensure screen is setup to show solely on screen 2 before continuing"
    echo -e "${NC}"
    read
    sudo cp ~/.config/monitors.xml /var/lib/gdm/.config/monitors.xml
    sudo chown gdm:gdm /var/lib/gdm/.config/monitors.xml
    echo -e "${GREEN}command completed${NC}"
    f_play_alert
}

function f_jlb_speaker {
    sudo cp ~/Documents/hosts /etc/
    sudo cp ~/Documents/pre-suspend.sh /usr/lib/systemd/system-sleep/
    sudo chmod +x /usr/lib/systemd/system-sleep/pre-suspend.sh
    sudo cp ~/Documents/recon_bt.sh ~/Documents/
    sudo chmod +x ~/Documents/recon_bt.sh
    f_play_alert
    echo -e "${RED}Press Bluetooth button on speaker before continuing"
    read
    ~/Documents/recon_bt.sh
    # echo -e "${RED}create copy of 'pre-suspend.sh' from Dropbox into  folder, and CHMOD +x it. This is for bluetooth speaker"
    echo -e "${GREEN}Speaker changes implemented"
    f_play_alert
    read
}

function f_dual_boot {
    echo ""
    echo -e "${RED}Is WINDOWS installed for dual boot?  Add the dual boot menu? [Y/N?]${NC}"
    echo ""
    read -r -p "" response
    if [[ "$response" =~ ^([yY][eE][sS]|[yY])$ ]];
        then
            sudo cat << EOF >> /boot/efi/loader/loader.conf
timeout 30
entries 1
EOF
    fi
    f_play_alert
}

function f_the_end {
    echo -e "${YELLOW}Open Wireshark - add profiles by Directory > under pop os"
    echo ""
    echo -e "${GREEN}and finally..."
    echo ""
    echo -e "${GREEN}Create containers - Firefox (jlesage/firefox), Pi-Hole (create with vlanMac), portainer/portainer-ce, haugene/transmission-openvpn:latest"
    echo -e "${GREEN}jokobaki/netalertx:latest, ntop/ntopng_amd64.dev:latest, nextcloud + mariadb:10.5 (use vscode to access the config.php file), cloudflared/cloudflared:latest"
    echo -e ""
    echo -e "${GREEN}All is done!${NC}"
    echo ""
    sudo rm net_changed.txt nord_installed.txt nord_connect.txt
    echo ""
    echo ""
    echo -e "${YELLOW}PRESS enter to ${NC}REBOOT${RED} - DO NOT RE RUN THIS SCRIPT"
    paplay /usr/share/sounds/Pop/stereo/notification/message-new-instant.oga
    read
    echo -e "${RED}***** Rebooting! *****${NC}"
    sudo systemctl reboot
    exit
}

function f_conduct {
    ### Stage 1
    f_print_header
    sudo apt update -y && sudo apt upgrade -y

    if [ ! -f ~/net_changed.txt ]; then
        echo -e "${RED}******* > Stage 1 of 4 > *******"
        echo ""
        f_change_net_name
    fi
    
    ### Stage 2

        if [ ! -f ~/nord_installed.txt ]; then
            f_play_alert
            echo -e "${RED}******* >> Stage 2 of 4 >> *******"
            f_purge_crap            
            f_5_apps_inst
            f_net_share
            f_dock_port_comp
            f_time_shift
            f_6_apps  # inc wireshark
            sudo apt autoremove -y
            
            f_pwsafe_brave
            f_instructs
            f_jlb_speaker
            f_warp_fish
            f_wazuh
            f_nordvpn
            f_starship
            
            echo -e "${RED}ABOUT TO REBOOT!"
            sudo touch ~/nord_installed.txt
            f_play_alert
            read
            sudo systemctl reboot
            exit
        else
            if [ ! -f ~/nord_connect.txt ]; then
                sudo apt autoremove && sudo apt clean -y
                clear
                echo -e "${RED}******* >>> Stage 3 of 4 >>> *******"
                f_connect_nord
                f_monitor_apps
                echo -e "${RED}ABOUT TO REBOOT - ${YELLOW}Re-run this script a 3rd time after reboot!${NC}"
                sudo touch ~/nord_connect.txt
                f_play_alert
                read
                sudo systemctl reboot
                exit
            fi
        fi

    sudo apt update && sudo apt upgrade -y && sudo apt install -f
    clear
    echo -e "${RED}******* >>> Stage 4 of 4 >>>> *******"
    f_dual_boot
    f_the_end
}
# OfficeCakes (c)2024
f_conduct
exit
