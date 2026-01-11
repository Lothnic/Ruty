#!/usr/bin/env python3
"""
Ruty System Tray Application with Qt5
Global keyboard shortcut: Super+Space
"""

import sys
import uuid
import os
import socket

from pathlib import Path
from datetime import datetime

from PyQt5.QtWidgets import QApplication, QSystemTrayIcon, QMenu, QAction
from PyQt5.QtGui import QIcon
from PyQt5.QtCore import QTimer, QThread, pyqtSignal
from pynput import keyboard
from langgraph.checkpoint.sqlite import SqliteSaver

# IPC Configuration
SOCKET_PATH = f"/tmp/ruty_{os.getuid()}.sock"


class IPCServer(QThread):
    """Background thread to listen for IPC commands"""
    toggle_signal = pyqtSignal()
    
    def run(self):
        # Remove stale socket
        if os.path.exists(SOCKET_PATH):
            try:
                os.unlink(SOCKET_PATH)
            except OSError:
                pass
                
        server = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        try:
            server.bind(SOCKET_PATH)
            server.listen(1)
            
            while True:
                try:
                    conn, _ = server.accept()
                    data = conn.recv(1024)
                    if data == b"toggle":
                        self.toggle_signal.emit()
                    conn.close()
                except Exception:
                    break
        except Exception as e:
            print(f"⚠️  Could not start IPC server: {e}")


def run_ipc_server():
    """Start listening for toggle commands"""
    # Create thread properly attached to main app logic
    # We assign it to 'app' so it doesn't get garbage collected
    app.ipc_thread = IPCServer()
    app.ipc_thread.toggle_signal.connect(toggle_chat)
    app.ipc_thread.start()


def send_toggle_command():
    """Send toggle command to running instance"""
    try:
        client = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        client.connect(SOCKET_PATH)
        client.send(b"toggle")
        client.close()
        print("✓ Sent toggle command to Ruty")
        return True
    except (FileNotFoundError, ConnectionRefusedError):
        return False

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent))

from ruty.agent import create_agent
from ruty.gui import create_chat_window


# Paths
SCRIPT_DIR = Path(__file__).parent
DB_PATH = str((SCRIPT_DIR / "ruty.sqlite").resolve())

# Global instances
app = None
chat_window = None
tray_icon = None


def toggle_chat():
    """Show/hide the chat window"""
    global chat_window
    if chat_window:
        chat_window.toggle_visibility()


def quit_app():
    """Exit application"""
    global app
    if app:
        app.quit()


def main():
    """Run the tray application"""
    global app, chat_window, tray_icon
    
    # Check for CLI arguments
    if len(sys.argv) > 1 and sys.argv[1] == "--toggle":
        if send_toggle_command():
            sys.exit(0)
        else:
            print("❌ Ruty is not running. Start it first.")
            sys.exit(1)
    
    # Check if already running (primitive check via socket)
    if send_toggle_command():
        print("⚡ Ruty is already running. Toggled window.")
        sys.exit(0)
        
    print("🧠 Ruty is starting...")
    
    # Create Qt application FIRST
    app = QApplication(sys.argv)
    app.setQuitOnLastWindowClosed(False)  # Keep running when window closes
    
    # Start IPC server (needs app to exist)
    run_ipc_server()
    
    # Initialize agent with SQLite persistence
    print(f"⚡ Connecting to database: {DB_PATH}")
    # Store context manager and enter it
    app.checkpointer_cm = SqliteSaver.from_conn_string(DB_PATH)
    checkpointer = app.checkpointer_cm.__enter__()
    agent = create_agent(checkpointer=checkpointer)
    
    # Create persistent session config
    session_id = f"tray_session_{datetime.now().strftime('%Y%m%d')}"
    config = {"configurable": {"thread_id": session_id}}
    
    # Create chat window
    print("🪟 Creating chat window...")
    chat_window = create_chat_window(agent, config)
    
    # Create system tray icon with fallback
    # Try to find a suitable icon
    icon = QIcon.fromTheme("brain")
    if icon.isNull():
        icon = QIcon.fromTheme("applications-accessories")  # Fallback
    if icon.isNull():
        icon = QIcon.fromTheme("system-search")  # Another fallback
    
    tray_icon = QSystemTrayIcon(icon, app)
    
    # Create tray menu
    menu = QMenu()
    
    # Actions
    toggle_action = QAction("💬 Toggle Chat", menu)
    toggle_action.triggered.connect(toggle_chat)
    menu.addAction(toggle_action)
    
    menu.addSeparator()
    
    quit_action = QAction("❌ Quit", menu)
    quit_action.triggered.connect(quit_app)
    menu.addAction(quit_action)
    
    tray_icon.setContextMenu(menu)
    tray_icon.setToolTip("Ruty - AI Assistant")
    tray_icon.show()
    
    # Double-click tray icon to open chat
    tray_icon.activated.connect(
        lambda reason: toggle_chat() if reason == QSystemTrayIcon.DoubleClick else None
    )
    
    # Register global hotkey
    print("⌨️  Registering hotkey: Ctrl+Alt+Space")
    try:
        from pynput.keyboard import Key, Listener
        
        # Track pressed keys
        current_keys = set()
        
        def on_press(key):
            current_keys.add(key)
            # Check if Ctrl+Alt+Space is pressed
            if (Key.ctrl_l in current_keys or Key.ctrl_r in current_keys) and \
               (Key.alt_l in current_keys or Key.alt_r in current_keys) and \
               Key.space in current_keys:
                QTimer.singleShot(0, toggle_chat)
        
        def on_release(key):
            try:
                current_keys.remove(key)
            except KeyError:
                pass
        
        # Start listener
        listener = Listener(on_press=on_press, on_release=on_release)
        listener.start()
        print("   ✓ Hotkey registered: Ctrl+Alt+Space")
    except Exception as e:
        print(f"⚠️  Could not register hotkey: {e}")
        tray_icon.showMessage(
            "Ruty",
            "Hotkey registration failed. Use tray menu.",
            QSystemTrayIcon.Warning,
            3000
        )
    
    print("✅ Ruty is running!")
    print("   • System Tray: Click icon")
    print("   • Wayland Users: Bind 'Super+Space' to:")
    print(f"     {os.path.abspath('run-tray.sh')} --toggle")
    
    # Show startup notification
    if QSystemTrayIcon.isSystemTrayAvailable():
        tray_icon.showMessage(
            "Ruty", 
            "Running! Bind Super+Space to 'run-tray.sh --toggle'",
            QSystemTrayIcon.Information, 3000
        )
    
    # Run Qt event loop
    sys.exit(app.exec_())


if __name__ == "__main__":
    main()
