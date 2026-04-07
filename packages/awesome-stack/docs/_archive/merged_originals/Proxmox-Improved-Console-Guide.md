# Improving Proxmox Console Experience: SPICE & xterm.js

## SPICE Console Setup
1. Edit the VM’s hardware in Proxmox (web UI).
2. Change the **Display** type to **SPICE**.
3. Start the VM.
4. Click on the **SPICE** console in the Proxmox GUI, then download/open the `.vv` file using the `remote-viewer` or `virt-viewer` client.
   - Arch-based install: `sudo pacman -S virt-viewer`

- SPICE features: Best desktop feel, clipboard sync, dynamic resizing, smooth mouse.
- Recommended: Set **Graphic Card** to `qxl`.

### Example `/etc/pve/qemu-server/611.conf`:
```
ostype: l26
vga: qxl
agent: 1
```
- Add (if available):
```
spice_enhancements: folders=on,clipboard=on
```

## xterm.js (Browser Console)
- Use the "Console" tab in Proxmox GUI. This is xterm.js for serial Linux consoles, not graphical desktops.
- Set display to "Default" or "std" for best results with graphical output.

## Notes
- For SPICE, client-side software required.
- For xterm.js, just use your browser.

---
*Documented by Agent Mode, July 30, 2025.*

