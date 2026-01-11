#!/usr/bin/env python3
"""
Ruty System Tray Application for Ubuntu/GNOME
Uses GTK and AppIndicator for native GNOME support.
"""

import os
import sys
import threading
import subprocess
from pathlib import Path

import gi
gi.require_version('Gtk', '3.0')
gi.require_version('AyatanaAppIndicator3', '0.1')
from gi.repository import Gtk, GLib, AyatanaAppIndicator3


# Get the script directory for relative paths
SCRIPT_DIR = Path(__file__).parent
MAIN_SCRIPT = SCRIPT_DIR / "main.py"


def notify(title: str, message: str):
    """Send desktop notification"""
    try:
        subprocess.run(["notify-send", title, message], check=False)
    except FileNotFoundError:
        pass


def open_chat(widget=None):
    """Open interactive chat in a new terminal"""
    terminals = [
        ["gnome-terminal", "--", "uv", "run", "python", str(MAIN_SCRIPT), "chat"],
        ["konsole", "-e", "uv", "run", "python", str(MAIN_SCRIPT), "chat"],
        ["xterm", "-e", "uv", "run", "python", str(MAIN_SCRIPT), "chat"],
    ]
    
    for cmd in terminals:
        try:
            subprocess.Popen(cmd, cwd=str(SCRIPT_DIR))
            return
        except FileNotFoundError:
            continue
    
    notify("Ruty", "No terminal found. Run manually.")


def sync_data(widget=None):
    """Sync data folder to Supermemory"""
    notify("Ruty", "🔄 Syncing...")
    
    def run_sync():
        result = subprocess.run(
            ["uv", "run", "python", str(MAIN_SCRIPT), "sync"],
            cwd=str(SCRIPT_DIR),
            capture_output=True,
            text=True
        )
        GLib.idle_add(
            notify, "Ruty",
            "✓ Sync done!" if result.returncode == 0 else "✗ Sync failed"
        )
    
    threading.Thread(target=run_sync, daemon=True).start()


def list_memories(widget=None):
    """Show memory count"""
    def run_list():
        result = subprocess.run(
            ["uv", "run", "python", str(MAIN_SCRIPT), "list"],
            cwd=str(SCRIPT_DIR),
            capture_output=True,
            text=True
        )
        first_line = result.stdout.strip().split('\n')[0] if result.stdout else "No memories"
        GLib.idle_add(notify, "Ruty", first_line)
    
    threading.Thread(target=run_list, daemon=True).start()


def quit_app(widget=None):
    """Exit application"""
    Gtk.main_quit()


def create_menu():
    """Create GTK menu for the indicator"""
    menu = Gtk.Menu()
    
    # Chat item
    item_chat = Gtk.MenuItem(label="💬 Open Chat")
    item_chat.connect("activate", open_chat)
    menu.append(item_chat)
    
    # Sync item
    item_sync = Gtk.MenuItem(label="🔄 Sync Data")
    item_sync.connect("activate", sync_data)
    menu.append(item_sync)
    
    # List item
    item_list = Gtk.MenuItem(label="📋 List Memories")
    item_list.connect("activate", list_memories)
    menu.append(item_list)
    
    # Separator
    menu.append(Gtk.SeparatorMenuItem())
    
    # Quit item
    item_quit = Gtk.MenuItem(label="❌ Quit")
    item_quit.connect("activate", quit_app)
    menu.append(item_quit)
    
    menu.show_all()
    return menu


def main():
    """Run the tray application"""
    # Create AppIndicator
    indicator = AyatanaAppIndicator3.Indicator.new(
        "ruty",
        "brain",  # Use a system icon, or provide path to custom icon
        AyatanaAppIndicator3.IndicatorCategory.APPLICATION_STATUS
    )
    
    indicator.set_status(AyatanaAppIndicator3.IndicatorStatus.ACTIVE)
    indicator.set_title("Ruty")
    indicator.set_menu(create_menu())
    
    # Set a secondary action for middle-click
    indicator.set_secondary_activate_target(None)
    
    print("🧠 Ruty is running in the system tray")
    print("   Click the brain icon for options")
    
    Gtk.main()


if __name__ == "__main__":
    main()
