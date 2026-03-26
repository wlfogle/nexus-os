"""
Main window implementation for MobaLiveCD Linux
"""

import gi
gi.require_version('Gtk', '4.0')
gi.require_version('Adw', '1')

from gi.repository import Gtk, Adw, Gio, GLib
import os
import threading
from core.enhanced_qemu_runner import AIEnhancedQEMURunner
from ui.help_dialog import HelpDialog
from ui.about_dialog import AboutDialog
from ui.usb_dialog import USBCreationDialog
from ui.usb_selector_dialog import USBSelectorDialog
from ui.nvme_selector_dialog import NVMePartitionSelectorDialog

class MobaLiveCDWindow(Adw.ApplicationWindow):
    """Main application window"""
    
    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        
        # Initialize enhanced QEMU runner with ZFS/UEFI support
        self.qemu_runner = AIEnhancedQEMURunner()
        self.current_boot_source = None
        self.boot_source_type = None  # 'iso', 'usb', or 'nvme'
        
        # Set up window properties
        self.set_title("MobaLiveCD")
        self.set_default_size(600, 500)
        
        # Create main content (includes header)
        self.setup_main_content()
        
        # Add actions
        self.create_actions()
        
        # Load translations
        self.setup_translations()
    
    def create_actions(self):
        """Create window actions"""
        # Help action
        help_action = Gio.SimpleAction.new("help", None)
        help_action.connect("activate", self.on_help)
        self.add_action(help_action)
        
        # About action
        about_action = Gio.SimpleAction.new("about", None)
        about_action.connect("activate", self.on_about)
        self.add_action(about_action)
    
    def setup_main_content(self):
        """Setup the main window content"""
        # Main container
        main_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        
        # Add header bar to main content
        header = Adw.HeaderBar()
        
        # Menu button
        menu_button = Gtk.MenuButton()
        menu_button.set_icon_name("open-menu-symbolic")
        menu_button.set_tooltip_text("Application menu")
        
        # Create menu model
        menu = Gio.Menu()
        menu.append("Help", "win.help")
        menu.append("About", "win.about")
        menu_button.set_menu_model(menu)
        
        header.pack_end(menu_button)
        main_box.append(header)
        
        self.set_content(main_box)
        
        # Welcome section
        self.create_welcome_section(main_box)
        
        # File association section
        self.create_association_section(main_box)
        
        # ISO selection section  
        self.create_iso_section(main_box)
        
        # Action buttons
        self.create_action_buttons(main_box)
    
    def create_welcome_section(self, parent):
        """Create welcome section"""
        welcome_group = Adw.PreferencesGroup()
        welcome_group.set_title("Welcome to MobaLiveCD")
        welcome_group.set_description(
            "This program allows you to test your bootable CD-ROM (LiveCD) "
            "using QEMU virtualization. You can either set up file associations "
            "for ISO images or directly run a LiveCD ISO file.")
        
        parent.append(welcome_group)
    
    def create_association_section(self, parent):
        """Create file association section"""
        assoc_group = Adw.PreferencesGroup()
        assoc_group.set_title("File Association")
        
        # Association row
        assoc_row = Adw.ActionRow()
        assoc_row.set_title("Install right-click menu association")
        assoc_row.set_subtitle("Add 'Open with MobaLiveCD' to ISO file context menu")
        
        # Association button
        assoc_button = Gtk.Button()
        assoc_button.set_label("Install Association")
        assoc_button.set_valign(Gtk.Align.CENTER)
        assoc_button.connect("clicked", self.on_install_association)
        assoc_row.add_suffix(assoc_button)
        
        # Uninstall button
        uninstall_button = Gtk.Button()
        uninstall_button.set_label("Remove")
        uninstall_button.set_valign(Gtk.Align.CENTER)
        uninstall_button.connect("clicked", self.on_remove_association)
        assoc_row.add_suffix(uninstall_button)
        
        assoc_group.add(assoc_row)
        parent.append(assoc_group)
    
    def create_iso_section(self, parent):
        """Create boot source selection section"""
        boot_group = Adw.PreferencesGroup()
        boot_group.set_title("Boot Source")
        
        # Current selection row
        self.boot_source_row = Adw.ActionRow()
        self.boot_source_row.set_title("No boot source selected")
        self.boot_source_row.set_subtitle("Use the buttons below to select an ISO, USB, or NVMe device")
        boot_group.add(self.boot_source_row)
        
        # Browse ISO row
        iso_row = Adw.ActionRow()
        iso_row.set_title("ISO Image")
        iso_row.set_subtitle("Boot from an ISO file")
        browse_button = Gtk.Button(label="Browse...")
        browse_button.set_valign(Gtk.Align.CENTER)
        browse_button.connect("clicked", self.on_browse_iso)
        iso_row.add_suffix(browse_button)
        iso_row.set_activatable_widget(browse_button)
        boot_group.add(iso_row)
        
        # USB row
        usb_row = Adw.ActionRow()
        usb_row.set_title("USB Device")
        usb_row.set_subtitle("Boot from a USB drive")
        usb_button = Gtk.Button(label="Select...")
        usb_button.set_valign(Gtk.Align.CENTER)
        usb_button.connect("clicked", self.on_select_usb)
        usb_row.add_suffix(usb_button)
        usb_row.set_activatable_widget(usb_button)
        boot_group.add(usb_row)
        
        # NVMe row
        nvme_row = Adw.ActionRow()
        nvme_row.set_title("NVMe Partition")
        nvme_row.set_subtitle("Boot from an NVMe partition")
        nvme_button = Gtk.Button(label="Select...")
        nvme_button.set_valign(Gtk.Align.CENTER)
        nvme_button.connect("clicked", self.on_select_nvme)
        nvme_row.add_suffix(nvme_button)
        nvme_row.set_activatable_widget(nvme_button)
        boot_group.add(nvme_row)
        
        parent.append(boot_group)
        
        # Run section
        run_group = Adw.PreferencesGroup()
        
        self.run_button = Gtk.Button(label="Boot in QEMU")
        self.run_button.add_css_class("suggested-action")
        self.run_button.set_sensitive(False)
        self.run_button.connect("clicked", self.on_run_boot_source)
        self.run_button.set_margin_top(6)
        self.run_button.set_margin_bottom(6)
        run_group.add(self.run_button)
        
        # USB creation button (visible only when ISO selected)
        self.usb_button = Gtk.Button(label="Create Bootable USB from ISO")
        self.usb_button.add_css_class("destructive-action")
        self.usb_button.set_sensitive(False)
        self.usb_button.set_visible(False)
        self.usb_button.connect("clicked", self.on_create_usb)
        run_group.add(self.usb_button)
        
        parent.append(run_group)
        
        # Keep compat reference
        self.boot_source_label = None
        self.usb_creation_group = None
    
    def create_action_buttons(self, parent):
        """Create bottom action buttons"""
        # Add some spacing
        parent.append(Gtk.Separator(orientation=Gtk.Orientation.HORIZONTAL))
        
        # Button box
        button_box = Gtk.Box(spacing=12)
        button_box.set_halign(Gtk.Align.CENTER)
        button_box.set_margin_top(20)
        button_box.set_margin_bottom(20)
        
        # Exit button
        exit_button = Gtk.Button()
        exit_button.set_label("Exit")
        exit_button.connect("clicked", lambda x: self.close())
        button_box.append(exit_button)
        
        parent.append(button_box)
    
    def setup_translations(self):
        """Setup language support - placeholder for now"""
        # TODO: Implement i18n support
        pass
    
    # Event handlers
    def on_browse_iso(self, button):
        """Handle ISO file browsing"""
        dialog = Gtk.FileChooserDialog(
            title="Select ISO file",
            transient_for=self,
            action=Gtk.FileChooserAction.OPEN
        )
        
        dialog.add_buttons(
            "_Cancel", Gtk.ResponseType.CANCEL,
            "_Open", Gtk.ResponseType.ACCEPT
        )
        
        # Add ISO filter
        filter_iso = Gtk.FileFilter()
        filter_iso.set_name("ISO Image Files")
        filter_iso.add_pattern("*.iso")
        filter_iso.add_pattern("*.ISO")
        dialog.add_filter(filter_iso)
        
        filter_all = Gtk.FileFilter()
        filter_all.set_name("All Files")
        filter_all.add_pattern("*")
        dialog.add_filter(filter_all)
        
        dialog.connect("response", self.on_file_dialog_response)
        dialog.show()
    
    def on_file_dialog_response(self, dialog, response):
        """Handle file dialog response"""
        if response == Gtk.ResponseType.ACCEPT:
            file = dialog.get_file()
            if file:
                self.load_iso_file(file.get_path())
        
        dialog.destroy()
    
    def load_boot_source(self, source_path, source_type):
        """Load a boot source (ISO file or USB device)"""
        is_valid, message = self.qemu_runner.validate_boot_source(source_path)
        
        if is_valid:
            self.current_boot_source = source_path
            self.boot_source_type = source_type
            
            if source_type == 'iso':
                self.boot_source_row.set_title(f"ISO: {os.path.basename(source_path)}")
                self.boot_source_row.set_subtitle(source_path)
                self.usb_button.set_visible(True)
                self.usb_button.set_sensitive(True)
            elif source_type == 'usb':
                self.boot_source_row.set_title(f"USB: {source_path}")
                self.boot_source_row.set_subtitle("USB device selected")
                self.usb_button.set_visible(False)
            else:
                self.boot_source_row.set_title(f"NVMe: {source_path}")
                self.boot_source_row.set_subtitle("NVMe partition selected")
                self.usb_button.set_visible(False)
            
            self.run_button.set_sensitive(True)
            print(f"Loaded {source_type.upper()}: {source_path}")
        else:
            self.show_error(f"Invalid {source_type}: {message}")
    
    def load_iso_file(self, iso_path):
        """Load an ISO file (backward compatibility)"""
        self.load_boot_source(iso_path, 'iso')
    
    def on_select_usb(self, button):
        """Handle USB device selection"""
        dialog = USBSelectorDialog(self)
        dialog.present()
        
        def on_dialog_response(dialog, response_id):
            if response_id == 'select':
                device_path = dialog.get_selected_device()
                if device_path:
                    self.load_boot_source(device_path, 'usb')
            dialog.destroy()
        
        dialog.connect('response', on_dialog_response)
    
    def on_select_nvme(self, button):
        """Handle NVMe partition selection"""
        dialog = NVMePartitionSelectorDialog(self)
        dialog.present()
        
        def on_dialog_response(dialog, response_id):
            if response_id == 'select':
                partition_path = dialog.get_selected_partition()
                if partition_path:
                    self.load_boot_source(partition_path, 'nvme')
            dialog.destroy()
        
        dialog.connect('response', on_dialog_response)
    
    def on_run_boot_source(self, button):
        """Handle running the boot source (ISO or USB)"""
        if not self.current_boot_source:
            return
        
        # Disable button during execution
        button.set_sensitive(False)
        button.set_label("Starting QEMU...")
        
        # Run QEMU in a separate thread
        def run_qemu():
            try:
                self.qemu_runner.run_boot_source(self.current_boot_source)
            except Exception as e:
                GLib.idle_add(self.show_error, f"Failed to start QEMU: {str(e)}")
            finally:
                GLib.idle_add(self.reset_run_button)
        
        thread = threading.Thread(target=run_qemu, daemon=True)
        thread.start()
    
    def on_run_iso(self, button):
        """Handle running the ISO (backward compatibility)"""
        self.on_run_boot_source(button)
    
    def reset_run_button(self):
        """Reset run button state"""
        self.run_button.set_sensitive(True)
        self.run_button.set_label("Boot in QEMU")
    
    def on_create_usb(self, button):
        """Handle USB creation"""
        if not self.current_boot_source or self.boot_source_type != 'iso':
            self.show_error("Please select an ISO file first to create a USB drive")
            return
        
        # Show USB creation dialog
        dialog = USBCreationDialog(self, self.current_boot_source)
        dialog.present()
    
    def on_install_association(self, button):
        """Handle installing file association"""
        try:
            self.install_desktop_association()
            self.show_success("File association installed successfully!")
        except Exception as e:
            self.show_error(f"Failed to install association: {str(e)}")
    
    def on_remove_association(self, button):
        """Handle removing file association"""
        try:
            self.remove_desktop_association()
            self.show_success("File association removed successfully!")
        except Exception as e:
            self.show_error(f"Failed to remove association: {str(e)}")
    
    def install_desktop_association(self):
        """Install desktop file association"""
        exec_path = os.path.abspath(os.path.join(
            os.path.dirname(os.path.abspath(__file__)), '..', 'mobalivecd.py'
        ))
        desktop_entry = """[Desktop Entry]
Version=1.0
Type=Application
Name=MobaLiveCD
Comment=Test LiveCD ISO files with QEMU
GenericName=LiveCD Tester
Exec={exec_path} %f
Icon=application-x-cd-image
Terminal=false
StartupNotify=true
MimeType=application/x-cd-image;application/x-iso9660-image;application/x-raw-disk-image;
Categories=System;Emulator;
Keywords=ISO;LiveCD;QEMU;Virtualization;CD;DVD;
""".format(exec_path=exec_path)
        
        # Create desktop file
        desktop_dir = os.path.expanduser("~/.local/share/applications")
        os.makedirs(desktop_dir, exist_ok=True)
        
        desktop_file = os.path.join(desktop_dir, "mobalivecd.desktop")
        with open(desktop_file, 'w') as f:
            f.write(desktop_entry)
        os.chmod(desktop_file, 0o755)
        
        # Update desktop & MIME databases
        os.system("update-desktop-database ~/.local/share/applications 2>/dev/null || true")
        os.system("update-mime-database ~/.local/share/mime 2>/dev/null || true")
    
    def remove_desktop_association(self):
        """Remove desktop file association"""
        desktop_file = os.path.expanduser("~/.local/share/applications/mobalivecd.desktop")
        if os.path.exists(desktop_file):
            os.remove(desktop_file)
            os.system("update-desktop-database ~/.local/share/applications 2>/dev/null || true")
    
    def on_help(self, action, param):
        """Show help dialog"""
        dialog = HelpDialog(self)
        dialog.present()
    
    def on_about(self, action, param):
        """Show about dialog"""
        dialog = AboutDialog(self)
        dialog.present()
    
    def show_error(self, message):
        """Show error message"""
        toast = Adw.Toast()
        toast.set_title(f"Error: {message}")
        toast.set_timeout(5)
        
        if not hasattr(self, 'toast_overlay'):
            self.toast_overlay = Adw.ToastOverlay()
            content = self.get_content()
            self.set_content(self.toast_overlay)
            self.toast_overlay.set_child(content)
        
        self.toast_overlay.add_toast(toast)
    
    def show_success(self, message):
        """Show success message"""
        toast = Adw.Toast()
        toast.set_title(message)
        toast.set_timeout(3)
        
        if not hasattr(self, 'toast_overlay'):
            self.toast_overlay = Adw.ToastOverlay()
            content = self.get_content()
            self.set_content(self.toast_overlay)
            self.toast_overlay.set_child(content)
        
        self.toast_overlay.add_toast(toast)
