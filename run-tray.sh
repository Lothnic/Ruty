#!/bin/bash
# Run the tray with system Python (for GTK access) + venv packages
cd "$(dirname "$0")"
PYTHONPATH="$(pwd)/.venv/lib/python3.10/site-packages:$PYTHONPATH" python3 tray.py
