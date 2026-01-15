#!/bin/bash
# Toggle Ruty launcher window
# Set up as custom shortcut with Super+Space in GNOME Settings

RUTY_BIN="/home/lothnic/Desktop/Projects/ruty/target/debug/ruty"

if pgrep -x ruty > /dev/null; then
    pkill -USR1 ruty  # Closes the window
else
    cd /home/lothnic/Desktop/Projects/ruty
    $RUTY_BIN &
fi
