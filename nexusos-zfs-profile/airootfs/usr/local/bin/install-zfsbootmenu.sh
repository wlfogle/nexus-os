#!/bin/bash
set -e

# Install zfsbootmenu from AUR
echo "Installing zfsbootmenu..."

# Create temporary user for building AUR packages
useradd -m -G wheel -s /bin/bash aur_builder
echo 'aur_builder ALL=(ALL) NOPASSWD: ALL' >> /etc/sudoers

# Install base-devel and git if not already installed
pacman -S --noconfirm --needed base-devel git

# Switch to aur_builder and install zfsbootmenu
sudo -u aur_builder bash -c '
cd /tmp
git clone https://aur.archlinux.org/zfsbootmenu.git
cd zfsbootmenu
makepkg -si --noconfirm
'

# Clean up
userdel -r aur_builder
sed -i '/aur_builder/d' /etc/sudoers

echo "ZFSBootMenu installed successfully"