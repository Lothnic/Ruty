
import sys
import os

# Add current directory to path so we can import ruty
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

try:
    from ruty.server import run_server
except ImportError:
    # Fallback if run directly or packaging issue
    try:
        from server import run_server
    except ImportError:
        print("Could not import ruty.server or server")
        sys.exit(1)

if __name__ == "__main__":
    run_server()
