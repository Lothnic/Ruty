#!/usr/bin/env python3
"""
Ruty - Personal AI Assistant with LangGraph

A tool-orchestrated agent with memory powered by Supermemory.

Usage:
    python main.py          # Interactive chat
    python main.py --help   # Show help
"""
import sys
from dotenv import load_dotenv

load_dotenv()

def main():
    """Main entry point for Ruty"""
    if len(sys.argv) > 1:
        arg = sys.argv[1]
        
        if arg in ["--help", "-h"]:
            print(__doc__)
            print("The agent can:")
            print("  • Search your knowledge base")
            print("  • Save new memories")
            print("  • Sync folders to memory")
            print("  • Upload individual files")
            print("  • List and delete documents")
            print()
            print("Just chat naturally - the agent decides which tools to use!")
            return
        
        if arg == "--version":
            print("Ruty v0.2.0 (LangGraph)")
            return
    
    # Run interactive chat
    from ruty.cli import interactive_chat
    interactive_chat()


if __name__ == "__main__":
    main()