#!/bin/bash
# Run the tray application (Robust version for Shortcuts)

# 1. Ensure we are in the script directory
cd "$(dirname "$(readlink -f "$0")")"

# 2. Log execution attempt (for debugging)
echo "$(date): Run triggered with args: $@" >> /tmp/ruty_debug.log

# 3. Add user bin to PATH (where uv lives)
export PATH="$HOME/.local/bin:$PATH"

# 4. Run with absolute path to uv
if command -v uv >/dev/null; then
    exec uv run python tray.py "$@"
else
    echo "$(date): Error - uv not found in PATH" >> /tmp/ruty_debug.log
    # Fallback to absolute path if command -v fails
    exec "$HOME/.local/bin/uv" run python tray.py "$@"
fi
