"""
NVMe partition selector dialog for MobaLiveCD Linux
"""

import gi
gi.require_version('Gtk', '4.0')
gi.require_version('Adw', '1')

from gi.repository import Gtk, Adw, GObject
from core.nvme_handler import NVMeHandler


class NVMePartitionSelectorDialog(Adw.Window):
    """Dialog for selecting NVMe partitions to boot"""
    
    __gsignals__ = {
        'response': (GObject.SignalFlags.RUN_FIRST, None, (str,))
    }
    
    def __init__(self, parent_window):
        super().__init__()
        
        self.parent_window = parent_window
        self.nvme_handler = NVMeHandler()
        self.selected_partition = None
        
        # Window properties
        self.set_title("Select NVMe Partition")
        self.set_default_size(600, 500)
        self.set_transient_for(parent_window)
        self.set_modal(True)
        
        # Setup UI
        self.setup_ui()
        
        # Load NVMe partitions
        self.refresh_partitions()
    
    def setup_ui(self):
        """Setup the dialog UI"""
        # Main container
        main_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self.set_content(main_box)
        
        # Header
        header = Adw.HeaderBar()
        header.set_show_start_title_buttons(False)
        header.set_show_end_title_buttons(False)
        
        # Cancel button
        cancel_button = Gtk.Button(label="Cancel")
        cancel_button.connect('clicked', self.on_cancel)
        header.pack_start(cancel_button)
        
        # Select button
        self.select_button = Gtk.Button(label="Select")
        self.select_button.add_css_class("suggested-action")
        self.select_button.set_sensitive(False)
        self.select_button.connect('clicked', self.on_select)
        header.pack_end(self.select_button)
        
        main_box.append(header)
        
        # Content area
        content = Adw.PreferencesPage()
        content.set_vexpand(True)
        
        # Warning section
        warning_group = Adw.PreferencesGroup()
        warning_group.set_title("⚠️ Important Warning")
        warning_group.set_description(
            "You are about to boot directly from an NVMe partition. This will:\n\n"
            "• Grant QEMU direct access to your partition data\n"
            "• Potentially modify the partition if the OS writes to it\n"
            "• Risk data corruption if not handled carefully\n\n"
            "Make sure you have backups of important data before proceeding!"
        )
        
        warning_row = Adw.ActionRow()
        warning_row.set_title("Read and understand the risks above")
        warning_row.add_css_class("warning")
        warning_group.add(warning_row)
        
        content.add(warning_group)
        
        # Partitions section
        self.partitions_group = Adw.PreferencesGroup()
        self.partitions_group.set_title("Available NVMe Partitions")
        content.add(self.partitions_group)
        
        # Refresh button section
        refresh_group = Adw.PreferencesGroup()
        refresh_row = Adw.ActionRow()
        refresh_row.set_title("Refresh partition list")
        refresh_row.set_subtitle("Scan for NVMe partitions again")
        
        refresh_button = Gtk.Button(label="Refresh")
        refresh_button.connect('clicked', self.on_refresh)
        refresh_row.add_suffix(refresh_button)
        refresh_group.add(refresh_row)
        content.add(refresh_group)
        
        # Scrolled window for content
        scrolled = Gtk.ScrolledWindow()
        scrolled.set_policy(Gtk.PolicyType.NEVER, Gtk.PolicyType.AUTOMATIC)
        scrolled.set_child(content)
        main_box.append(scrolled)
    
    def refresh_partitions(self):
        """Refresh the list of available NVMe partitions"""
        # Clear existing partition rows by creating a new preferences group
        # This avoids the GTK widget removal issues
        try:
            # Get parent container
            parent = self.partitions_group.get_parent()
            if parent:
                # Remove old group
                parent.remove(self.partitions_group)
                
                # Create new group
                self.partitions_group = Adw.PreferencesGroup()
                self.partitions_group.set_title("Available NVMe Partitions")
                
                # Add back to parent (after refresh_group and before content end)
                # Find the refresh group to insert before it
                refresh_child = None
                child = parent.get_first_child()
                while child is not None:
                    if hasattr(child, 'get_title') and child.get_title() and "refresh" in child.get_title().lower():
                        refresh_child = child
                        break
                    child = child.get_next_sibling()
                
                if refresh_child:
                    parent.insert_child_before(self.partitions_group, refresh_child)
                else:
                    parent.append(self.partitions_group)
            else:
                # Fallback: try to clear children individually but more safely
                while True:
                    child = self.partitions_group.get_first_child()
                    if child is None:
                        break
                    try:
                        self.partitions_group.remove(child)
                    except:
                        break  # Exit if removal fails
        except Exception as e:
            print(f"Warning: Error during partition refresh: {e}")
        
        # Reset selection
        self.selected_partition = None
        self.select_button.set_sensitive(False)
        
        # Detect NVMe devices and partitions
        try:
            nvme_devices = self.nvme_handler.detect_nvme_devices()
            
            if not nvme_devices:
                # No NVMe devices found
                no_devices_row = Adw.ActionRow()
                no_devices_row.set_title("No NVMe devices found")
                no_devices_row.set_subtitle("Make sure you have NVMe storage devices installed")
                no_devices_row.add_css_class("dim-label")
                self.partitions_group.add(no_devices_row)
                return
            
            # Group partitions by device
            for device in nvme_devices:
                # Add device header
                device_row = Adw.ActionRow()
                device_row.set_title(f"🔧 {device['name']} ({device['size']})")
                device_row.set_subtitle(f"Device: {device['device']}")
                device_row.add_css_class("accent")
                self.partitions_group.add(device_row)
                
                partitions = device.get('partitions', [])
                if not partitions:
                    # No partitions on this device
                    no_parts_row = Adw.ActionRow()
                    no_parts_row.set_title("  └─ No partitions found")
                    no_parts_row.add_css_class("dim-label")
                    self.partitions_group.add(no_parts_row)
                    continue
                
                # Add partitions
                for i, partition in enumerate(partitions):
                    is_last = i == len(partitions) - 1
                    prefix = "  └─ " if is_last else "  ├─ "
                    
                    # Create partition row
                    part_row = Adw.ActionRow()
                    
                    # Title with partition info
                    title = f"{prefix}{partition['name']} ({partition['size']})"
                    part_row.set_title(title)
                    
                    # Subtitle with filesystem and mount info
                    subtitle_parts = []
                    if partition.get('fstype'):
                        subtitle_parts.append(f"FS: {partition['fstype']}")
                    if partition.get('label'):
                        subtitle_parts.append(f"Label: {partition['label']}")
                    if partition.get('mountpoint'):
                        subtitle_parts.append(f"Mounted: {partition['mountpoint']}")
                    
                    if subtitle_parts:
                        part_row.set_subtitle(" • ".join(subtitle_parts))
                    else:
                        part_row.set_subtitle(partition['device'])
                    
                    # Add bootable indicator
                    if partition.get('bootable', False):
                        bootable_icon = Gtk.Image()
                        bootable_icon.set_from_icon_name("emblem-ok-symbolic")
                        bootable_icon.set_tooltip_text("Potentially bootable")
                        part_row.add_prefix(bootable_icon)
                    
                    # Add radio button
                    radio_button = Gtk.CheckButton()
                    radio_button.set_group(getattr(self, '_partition_group', None))
                    if not hasattr(self, '_partition_group'):
                        self._partition_group = radio_button
                    
                    radio_button.connect('toggled', self.on_partition_selected, partition)
                    part_row.add_suffix(radio_button)
                    part_row.set_activatable(True)
                    part_row.connect('activated', lambda row, rb=radio_button: rb.set_active(True))
                    
                    # Style based on bootable status
                    if partition.get('bootable', False):
                        part_row.add_css_class("success")
                    elif partition.get('mountpoint'):
                        part_row.add_css_class("warning")
                    
                    self.partitions_group.add(part_row)
            
            # Add info about bootable partitions
            info_row = Adw.ActionRow()
            info_row.set_title("ℹ️ Partition Selection Tips")
            info_row.set_subtitle(
                "• Green partitions (✓) are detected as potentially bootable\n"
                "• Yellow partitions are currently mounted - they will be unmounted before booting\n"
                "• You can select any partition, but bootable ones are more likely to work"
            )
            info_row.add_css_class("dim-label")
            self.partitions_group.add(info_row)
            
        except Exception as e:
            # Error loading partitions
            error_row = Adw.ActionRow()
            error_row.set_title("Error loading NVMe partitions")
            error_row.set_subtitle(f"Error: {str(e)}")
            error_row.add_css_class("error")
            self.partitions_group.add(error_row)
    
    def on_partition_selected(self, radio_button, partition):
        """Handle partition selection"""
        if radio_button.get_active():
            self.selected_partition = partition
            self.select_button.set_sensitive(True)
        elif self.selected_partition == partition:
            self.selected_partition = None
            self.select_button.set_sensitive(False)
    
    def on_refresh(self, button):
        """Handle refresh button click"""
        button.set_sensitive(False)
        button.set_label("Refreshing...")
        
        # Refresh in next idle cycle
        def do_refresh():
            self.refresh_partitions()
            button.set_sensitive(True)
            button.set_label("Refresh")
            return False
        
        from gi.repository import GLib
        GLib.idle_add(do_refresh)
    
    def on_select(self, button):
        """Handle select button click"""
        if self.selected_partition:
            self.emit('response', 'select')
    
    def on_cancel(self, button):
        """Handle cancel button click"""
        self.emit('response', 'cancel')
    
    def get_selected_partition(self):
        """Get the currently selected partition device path"""
        if self.selected_partition:
            return self.selected_partition.get('device')
        return None