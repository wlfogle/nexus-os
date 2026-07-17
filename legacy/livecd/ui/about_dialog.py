"""
About dialog for MobaLiveCD Linux
"""

import gi
gi.require_version('Gtk', '4.0')
gi.require_version('Adw', '1')

from gi.repository import Gtk, Adw
from core.qemu_runner import QEMURunner

class AboutDialog:
    """About dialog showing application information"""
    
    def __init__(self, parent):
        self.parent = parent
        
    def present(self):
        """Show the about dialog"""
        # Adw.AboutWindow requires libadwaita 1.2+, use Adw.Window fallback
        try:
            dialog = Adw.AboutWindow()
        except (AttributeError, TypeError):
            self._present_fallback()
            return
        dialog.set_transient_for(self.parent)
        
        # Application details
        dialog.set_application_name("MobaLiveCD Linux")
        dialog.set_application_icon("application-x-cd-image")
        dialog.set_version("1.0.0")
        dialog.set_comments("A QEMU-based LiveCD/ISO testing tool for Linux")
        
        # Copyright and license
        dialog.set_copyright("© 2024 Linux Port")
        dialog.set_license_type(Gtk.License.GPL_2_0)
        
        # Description
        dialog.set_website("https://github.com/mobatek/mobalivecd")
        dialog.set_issue_url("https://github.com/mobatek/mobalivecd/issues")
        
        # Credits
        dialog.set_developers([
            "Original MobaLiveCD by Mobatek",
            "Linux port adaptation"
        ])
        
        # Additional details
        try:
            qemu_runner = QEMURunner()
            system_info = qemu_runner.get_system_info()
            
            details = f"""System Information:
• QEMU Binary: {system_info.get('qemu_binary', 'Not found')}
• QEMU Version: {system_info.get('qemu_version', 'Unknown')}
• KVM Acceleration: {'Available' if system_info.get('kvm_available') else 'Not available'}
• Default Memory: {system_info.get('memory', '512M')}

This application allows you to test bootable CD-ROM images (LiveCDs) using QEMU virtualization. It provides an easy-to-use graphical interface for running ISO files without burning them to physical media.

Features:
• Simple GUI for ISO file selection
• QEMU integration with optimal settings
• KVM acceleration when available
• File association support
• Multi-language support (planned)

Based on the original MobaLiveCD for Windows, this Linux port brings the same functionality to Linux desktop environments using modern GTK4/Libadwaita interface."""
            
            dialog.set_debug_info(details)
            
        except Exception as e:
            dialog.set_debug_info(f"System info unavailable: {str(e)}")
        
        # Legal information
        dialog.set_license("""
This program is free software; you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation; either version 2 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License along
with this program; if not, write to the Free Software Foundation, Inc.,
51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
""")
        
        dialog.present()
    
    def _present_fallback(self):
        """Fallback about dialog for libadwaita < 1.2"""
        dialog = Adw.Window()
        dialog.set_transient_for(self.parent)
        dialog.set_modal(True)
        dialog.set_title("About MobaLiveCD Linux")
        dialog.set_default_size(400, 350)
        
        main_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
        header = Adw.HeaderBar()
        main_box.append(header)
        
        content = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
        content.set_margin_top(24)
        content.set_margin_bottom(24)
        content.set_margin_start(24)
        content.set_margin_end(24)
        content.set_halign(Gtk.Align.CENTER)
        
        title = Gtk.Label()
        title.set_markup("<big><b>MobaLiveCD Linux</b></big>")
        content.append(title)
        
        version = Gtk.Label(label="Version 1.0.0")
        content.append(version)
        
        desc = Gtk.Label(label="A QEMU-based LiveCD/ISO testing tool for Linux")
        desc.set_wrap(True)
        content.append(desc)
        
        copyright_label = Gtk.Label(label="GPL v2+ License")
        copyright_label.add_css_class("dim-label")
        content.append(copyright_label)
        
        main_box.append(content)
        dialog.set_content(main_box)
        dialog.present()
