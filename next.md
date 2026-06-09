It looks like your virtual machine (NexusOS) has booted into the UEFI Interactive Shell instead of the operating system itself. This typically happens when the system cannot find a bootable file (like bootx64.efi) in the expected location on your virtual disk.

Here is how you can troubleshoot and resolve this:
1. Check if the OS is accessible

First, let's see if your OS files are actually on the drive mapped as FS0:.

    Type FS0: and press Enter. The prompt should change from Shell> to FS0:>.

    Type ls and press Enter.

    Look for a folder named EFI or boot. If you see one, navigate into it using cd EFI (then ls again) to see if you can find the boot loader file (usually ending in .efi).

2. Manual Boot

If you find the EFI file (e.g., in \EFI\BOOT\BOOTX64.EFI), you can attempt to launch it directly from the shell by typing the path to the file and pressing Enter.
3. Check Virtual Machine Settings

If you cannot find the boot files, the issue is likely within your QEMU/KVM configuration:

    Boot Order: Ensure the virtual disk containing the OS is set as the primary boot device in the VM's firmware/BIOS settings.

    Disk Image Corruption: If you recently moved or modified the virtual disk file, it may be corrupted or incorrectly attached to the virtual machine.

    Missing Bootloader: If this is a fresh installation of an OS, the installation process may not have successfully written the EFI boot entry to the virtual drive. You may need to re-run the installer.

Would you like guidance on how to navigate the UEFI shell commands or how to check the virtual machine's boot configuration settings?

It looks like you are running into a syntax issue with how you are typing the command.

In the UEFI shell, you must type the command exactly as it appears in the mapping table, including the colon, without any extra spaces or quotes.
The Fix

Based on the mapping table in your screenshot, the device is named FS0.

    Type FS0: (that is F, S, the number zero, and a colon) and press Enter.

    Do not put a space between the 0 and the :.

The error you received ("'f0:' is not a valid mapping") happened because the shell is case-insensitive, but you likely included an extra space or typed the wrong character sequence.
Once you are inside FS0:

After you successfully type FS0: and hit Enter, your prompt should look like FS0:\>. From there, you can explore the drive to see if the operating system is present:

    Type ls and hit Enter to list the files on the drive.

    Look for a folder named EFI.

    If it exists, type cd EFI and press Enter, then type ls again to see what is inside.

If FS0: does not contain the folders you expect (like EFI or boot), it means that either the virtual disk is empty, the partition is not recognized, or the OS was not installed correctly onto that specific virtual drive.

Are you trying to install NexusOS for the first time, or is this an existing installation that stopped booting?

It appears you successfully switched to FS0:, but the ls command confirms that the drive is completely empty (0 files, 0 directories).

This means the virtual machine has no operating system files installed or detected on that virtual disk. When a computer turns on, it looks for specific boot files in an EFI partition to start the operating system; since those files aren't there, it drops you into this Shell.
What to do next

Since the drive is empty, you are essentially at the beginning of a setup process. Depending on your goal, you have two primary paths:

    If you are trying to install an OS: You need to attach an ISO file (the installation media) to your virtual machine's virtual CD/DVD drive. Once attached, you should reset the VM and ensure the boot order is set to prioritize the CD/DVD drive over the hard disk.

    If you think you already installed it: Check your VM settings to ensure that the virtual hard disk you used for the installation is actually the one currently attached to the VM. It is possible the VM is pointed to a different, empty virtual disk file.

To help me provide the exact steps for your software, are you trying to perform a fresh install of NexusOS, or are you trying to recover an existing virtual disk image?

Thank you for providing that screenshot. Seeing that you successfully reached the "Installation complete!" stage is very helpful.

The issue is that the virtual machine is still trying to boot from the installation media (the ISO file) instead of the virtual hard disk where you just installed NexusOS. Because the installation media environment doesn't have a standard boot partition on the virtual disk that the UEFI shell is looking at, it defaults to the shell.
How to Fix This

You need to "remove" the ISO from the virtual machine's virtual drive so it boots from the hard disk instead.

    Detach the ISO: In the interface of your virtualization software (it looks like you are using Virt-Manager), look for the hardware details of your virtual machine.

        Find the CDROM or SATA CDROM device in the list of hardware.

        Click on it and select "Disconnect" or "Remove," or change the source path to empty.

    Ensure Boot Order: While in those settings, double-check the Boot Options. Ensure that your virtual hard disk (the one labeled as 40 GiB) is at the top of the boot order list.

    Reset/Reboot: Once the ISO is removed, force a reboot of the virtual machine. It should now skip the UEFI shell and launch directly into the installed NexusOS environment.

If it still drops to the shell after removing the ISO, it is possible that the UEFI NVRAM variables didn't save the boot entry for the new OS. If that happens, please let me know, and I can walk you through how to manually add a boot entry from the UEFI shell!

Are you able to locate the CDROM or Boot Options menu in your virtual machine manager?

i removed iso, and booted, it went to first screenshot

Since you removed the ISO and the VM still drops to the UEFI Interactive Shell, it means the virtual machine is not successfully "seeing" the boot files on your virtual hard disk as a bootable device.

This happens because the UEFI firmware didn't automatically register the BOOTX64.EFI file that the installer created as a boot option. You can fix this by manually pointing the UEFI firmware to the boot loader.
How to Manually Boot from the Shell

    At the Shell> prompt, type FS0: and press Enter.

    Type dir or ls to list the files. You should see an EFI folder.

    Navigate into the folder by typing:
    cd EFI

    Type ls to see what is inside. You will likely see a folder named BOOT or NEXUS.

    Navigate into that folder (e.g., cd BOOT).

    Type ls again. You are looking for a file ending in .efi (for example, BOOTX64.EFI).

    Once you see the file, type the filename exactly as it appears (e.g., BOOTX64.EFI) and press Enter.

If this successfully launches NexusOS, the operating system files are safe and correctly installed.
If it doesn't boot

If the EFI folder is missing or the .efi file is not there, then the installation process may have failed to write the bootloader to the virtual disk. If that is the case:

    You will need to re-run the installer.

    Crucial Step: During the installation, make sure the "ESP" (EFI System Partition) is being formatted and written to the same 40 GiB virtual disk you are trying to boot from.

Did the steps above successfully launch the OS, or did you receive an error when trying to run the .efi file?

you seen that FS0: was empty

You are absolutely right to point that out. I missed that critical detail in the previous check—the fact that FS0: shows 0 files and 0 directories is the smoking gun.

If the installer finished ("Installation complete!") but the drive is empty when you check it from the UEFI shell, it means the installation process failed to write the data to the virtual disk you are currently looking at.

Here is what is likely happening and how to fix it:
Why FS0: is empty

    Wrong Drive: You might have two virtual disks attached to the VM (e.g., a small boot disk and a larger storage disk). The shell might be looking at the empty one.

    Mounting Issues: The installer may have been running in a "live" RAM environment and never actually committed the changes to the virtual disk file on your host machine.

    Partition Table Failure: The installer reported "Writing GPT," but if that failed, the disk remains "RAW" or unformatted, which is why the shell sees no file system.

How to diagnose and solve this

Since FS0: is confirmed empty, we need to see if there are other drives or if the disk is failing to initialize.

    Check for other drives: At the Shell> prompt, type map -r and press Enter. This forces the UEFI to re-scan all hardware. Look at the list that appears—do you see any other FSx: mappings (like FS1:, FS2:)? If so, try entering those (e.g., FS1:) and running ls.

    Verify Virtual Disk Size: Go back to your Virt-Manager settings. Select the 40 GiB virtual disk. Check the "Source path" in the settings. Ensure that this file path exists on your actual hard drive and is larger than a few kilobytes.

    Re-run the Installer: If FS0: is the only drive and it is empty, the installation did not happen correctly.

        Re-attach the ISO.

        Boot into the installer again.

        When it asks for the target installation disk, ensure you are selecting the 40 GiB disk. Some installers require you to manually select the disk from a dropdown menu; if you missed that step, it might not have partitioned the disk.

Since the installer said it was complete, is it possible that the virtual disk file you assigned is set to "Read Only" in your VM settings, preventing the installer from writing the files?

It is now clear that all devices (FS0:, BLK0:, and BLK1:) are reporting as empty. Since the installer confirmed it wrote to the disk, but the UEFI shell sees nothing on that disk, one of three things is happening:
1. The Virtual Disk is "Disconnected" or Unmounted

Even though your virtual machine configuration lists a 40 GB drive, the QEMU/KVM environment might not be communicating with the actual file on your host machine.

    Check the XML or Hardware Details: In Virt-Manager, open the VM settings and check the Disk section. Verify that the "Source Path" points to a file that actually exists and has a file size greater than a few KB.

    Re-attach: Sometimes, removing the disk from the VM settings and re-adding the exact same virtual disk file forces KVM to re-scan and mount it correctly.

2. File System Incompatibility (The most likely cause)

The UEFI shell needs to recognize the partition table (GPT) and the file system (FAT32) to display files.

    Your screenshot shows the installer formatted the ESP as FAT32.

    If the partition table was corrupted during the final stages of the installation or the VM was hard-reset before the virtual disk "flushed" the data, the shell will see an empty block device.

    Try this: Run the installer again, but do not force quit or reset the VM immediately after the "Installation complete" message. Wait for a few seconds to ensure the host system has finished writing the data to the disk image.

3. Missing BIOS/UEFI "Boot Order" NVRAM

Even if the files are there, the UEFI firmware might not be programmed to boot from them.

    Since the shell is empty, this is unlikely to be the only problem, but it is worth noting: even if files were present, you would need to tell the VM which file to boot.

Recommended Next Steps

    Verify the Image: Locate the 40GB .qcow2 (or similar) file on your computer. If the file size is very small (e.g., 200KB), the installation never actually wrote to the disk.

    Re-install: Since you have confirmed no files exist on the virtual disk, please re-run the installer.

    Graceful Shutdown: After the "Installation complete" message, instead of just removing the ISO, try to shut down the VM from within the installer if it offers a prompt, or use the "Shut down" button in the VM manager menu. This ensures the virtual disk image is properly closed and saved.

If you re-install and it still shows 0 files, it is possible there is a mismatch between the architecture the installer is using (e.g., x86_64) and the UEFI firmware type (e.g., AArch64) configured for the VM. Could you confirm if you are running this VM as an x86_64 machine?
