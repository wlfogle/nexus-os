#!/usr/bin/env python3
# -*- coding: utf-8 -*-
#
# SPDX-FileCopyrightText: 2024
# SPDX-License-Identifier: GPL-3.0+
#
# ZFS post-installation configuration module for Calamares
# This module handles ZFS-specific system configuration after installation

import subprocess
import libcalamares
import os
import sys

def pretty_name():
    return "ZFS Post-Installation Configuration"

def status_name():
    return "Configuring ZFS system services..."

def run():
    """
    Configure ZFS services and system settings post-installation
    """
    
    # Get configuration
    config = libcalamares.job.configuration
    gs = libcalamares.globalstorage
    
    # Check if ZFS was used - check both possible global storage keys
    use_zfs = gs.value("useZfs") or gs.value("zfsDatasets") or gs.value("zfsPoolInfo")
    if not use_zfs:
        libcalamares.utils.debug("ZFS not used, skipping ZFS post-configuration")
        return None
    
    root_mount_point = gs.value("rootMountPoint")
    if not root_mount_point:
        root_mount_point = "/mnt"
    
    # Get ZFS pool information from various possible sources
    zfs_pool_name = gs.value("zfsPoolName") 
    if not zfs_pool_name:
        # Try to get from zfsPoolInfo which is used by the existing C++ ZFS module
        zfs_pool_info = gs.value("zfsPoolInfo")
        if zfs_pool_info and len(zfs_pool_info) > 0:
            zfs_pool_name = zfs_pool_info[0].get("poolName", "rpool")
        else:
            zfs_pool_name = config.get("poolName", "rpool")
    
    try:
        # Copy ZFS hostid file (similar to existing zfshostid module)
        result = copy_zfs_hostid(root_mount_point)
        if result:
            return result
            
        # Enable ZFS services in the installed system
        result = enable_zfs_services(root_mount_point)
        if result:
            return result
            
        # Configure ZFS-specific system settings
        result = configure_zfs_system(root_mount_point, zfs_pool_name)
        if result:
            return result
            
        # Set up ZFS cache file
        result = setup_zfs_cache(root_mount_point, zfs_pool_name)
        if result:
            return result
            
        # Configure mkinitcpio for ZFS
        result = configure_initramfs(root_mount_point)
        if result:
            return result
            
        # Reset ZFS mountpoints to proper values for the installed system
        result = reset_zfs_mountpoints(zfs_pool_name)
        if result:
            return result
            
        libcalamares.utils.debug("ZFS post-installation configuration completed successfully")
        return None
        
    except Exception as e:
        return f"ZFS post-installation configuration failed: {str(e)}"

def copy_zfs_hostid(root_mount_point):
    """
    Copy ZFS hostid file (from existing zfshostid module functionality)
    """
    
    try:
        hostid_source = '/etc/hostid'
        hostid_destination = f'{root_mount_point}/etc/hostid'
        
        if os.path.exists(hostid_source):
            import shutil
            shutil.copy2(hostid_source, hostid_destination)
            libcalamares.utils.debug("Copied ZFS hostid file")
        else:
            libcalamares.utils.warning("ZFS hostid file not found at /etc/hostid")
            
        return None
        
    except Exception as e:
        return f"Failed to copy ZFS hostid: {str(e)}"

def enable_zfs_services(root_mount_point):
    """
    Enable ZFS services in the installed system
    """
    
    services = [
        "zfs-import-cache.service",
        "zfs-import.target", 
        "zfs-mount.service",
        "zfs.target"
    ]
    
    try:
        for service in services:
            cmd = ["chroot", root_mount_point, "systemctl", "enable", service]
            libcalamares.utils.debug(f"Enabling service: {service}")
            result = subprocess.run(cmd, check=True, capture_output=True, text=True)
            
        return None
        
    except subprocess.CalledProcessError as e:
        return f"Failed to enable ZFS services: {e.stderr}"

def configure_zfs_system(root_mount_point, pool_name):
    """
    Configure ZFS-specific system settings
    """
    
    try:
        # Create ZFS configuration directory
        zfs_conf_dir = os.path.join(root_mount_point, "etc", "zfs")
        os.makedirs(zfs_conf_dir, exist_ok=True)
        
        # Create zpool.cache symlink if it doesn't exist
        cache_file = os.path.join(zfs_conf_dir, "zpool.cache")
        if not os.path.exists(cache_file):
            # Copy the current cache file
            current_cache = "/etc/zfs/zpool.cache"
            if os.path.exists(current_cache):
                subprocess.run(["cp", current_cache, cache_file], check=True)
        
        # Set ZFS module parameters
        modprobe_conf = os.path.join(root_mount_point, "etc", "modprobe.d", "zfs.conf")
        with open(modprobe_conf, "w") as f:
            f.write("# ZFS module parameters\\n")
            f.write("# Limit ARC to half of available RAM\\n")
            f.write("options zfs zfs_arc_max=2147483648\\n")
            f.write("# Enable prefetch\\n")
            f.write("options zfs zfs_prefetch_disable=0\\n")
            
        return None
        
    except Exception as e:
        return f"Failed to configure ZFS system settings: {str(e)}"

def setup_zfs_cache(root_mount_point, pool_name):
    """
    Set up ZFS cache file for faster boot
    """
    
    try:
        # Generate zpool.cache
        cache_file = os.path.join(root_mount_point, "etc", "zfs", "zpool.cache")
        cmd = ["zpool", "set", "cachefile=" + cache_file, pool_name]
        subprocess.run(cmd, check=True, capture_output=True)
        
        return None
        
    except subprocess.CalledProcessError as e:
        return f"Failed to set up ZFS cache: {e.stderr}"

def configure_initramfs(root_mount_point):
    """
    Configure mkinitcpio for ZFS support
    """
    
    try:
        mkinitcpio_conf = os.path.join(root_mount_point, "etc", "mkinitcpio.conf")
        
        # Check if file exists
        if not os.path.exists(mkinitcpio_conf):
            libcalamares.utils.warning("mkinitcpio.conf not found, skipping initramfs configuration")
            return None
        
        # Read current configuration
        with open(mkinitcpio_conf, "r") as f:
            lines = f.readlines()
        
        # Modify hooks and modules
        new_lines = []
        for line in lines:
            if line.startswith("HOOKS="):
                # Add zfs hook if not present
                if "zfs" not in line:
                    # Add zfs before filesystems
                    line = line.replace("filesystems", "zfs filesystems")
                new_lines.append(line)
            elif line.startswith("MODULES="):
                # Add zfs module if not present
                if "zfs" not in line:
                    line = line.replace("MODULES=(", "MODULES=(zfs ")
                new_lines.append(line)
            else:
                new_lines.append(line)
        
        # Write back the configuration
        with open(mkinitcpio_conf, "w") as f:
            f.writelines(new_lines)
        
        # Regenerate initramfs
        cmd = ["chroot", root_mount_point, "mkinitcpio", "-P"]
        subprocess.run(cmd, check=True, capture_output=True)
        
        return None
        
    except Exception as e:
        return f"Failed to configure initramfs: {str(e)}"

def reset_zfs_mountpoints(pool_name):
    """
    Reset ZFS dataset mountpoints to their final values
    """
    
    # Get datasets from global storage if available
    gs = libcalamares.globalstorage
    zfs_datasets = gs.value("zfsDatasets")
    
    if zfs_datasets:
        # Use datasets from global storage (from existing C++ module)
        datasets = []
        for dataset in zfs_datasets:
            if isinstance(dataset, dict):
                ds_name = dataset.get("dsName", "")
                mountpoint = dataset.get("mountpoint", "")
                if ds_name and mountpoint:
                    datasets.append((f"{pool_name}/{ds_name}", mountpoint))
    else:
        # Use default dataset layout
        datasets = [
            (f"{pool_name}/ROOT/default", "/"),
            (f"{pool_name}/home", "/home"),
            (f"{pool_name}/var", "/var"),
            (f"{pool_name}/var/log", "/var/log")
        ]
    
    try:
        for dataset, mountpoint in datasets:
            # Check if dataset exists
            result = subprocess.run(["zfs", "list", "-H", "-o", "name", dataset], 
                                  capture_output=True, text=True)
            if result.returncode == 0:
                # Set the proper mountpoint
                subprocess.run(["zfs", "set", f"mountpoint={mountpoint}", dataset],
                             check=True, capture_output=True)
                libcalamares.utils.debug(f"Set mountpoint for {dataset} to {mountpoint}")
        
        return None
        
    except subprocess.CalledProcessError as e:
        return f"Failed to reset ZFS mountpoints: {e.stderr}"