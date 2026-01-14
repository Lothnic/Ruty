"""System tools for managing documents and system operations.

Provides tools for:
- Document management (list, delete)
- URL opening
- Shell command execution (sandboxed)
"""
import os
import subprocess
import webbrowser
from langchain_core.tools import tool
from ..memory import list_docs, delete_document as delete_doc_api


@tool
def list_documents() -> str:
    """List all documents in your knowledge base.
    
    Returns a list of all saved documents and memories.
    
    Returns:
        List of document titles/IDs
    """
    docs = list_docs()
    
    if not docs:
        return "No documents found in your knowledge base."
    
    memories = docs.get("memories", [])
    
    if not memories:
        return "No documents found in your knowledge base."
    
    lines = [f"Found {len(memories)} documents:"]
    for doc in memories[:30]:  # Limit to 30 for readability
        title = doc.get("title") or doc.get("customId") or doc.get("id", "Unknown")
        lines.append(f"  • {title}")
    
    if len(memories) > 30:
        lines.append(f"  ... and {len(memories) - 30} more")
    
    return "\n".join(lines)


@tool
def delete_document(doc_id: str) -> str:
    """Delete a document from your knowledge base.
    
    Use this to remove a specific document by its ID.
    
    Args:
        doc_id: ID of the document to delete
    
    Returns:
        Confirmation message
    """
    result = delete_doc_api(doc_id)
    
    if result:
        return f"✓ Deleted document: {doc_id}"
    else:
        return f"✗ Failed to delete document: {doc_id}"


@tool
def open_url(url: str) -> str:
    """Open a URL in the user's default web browser.
    
    Use this to help users access websites, documentation,
    or any web content they mention.
    
    Args:
        url: The URL to open (must start with http:// or https://)
    
    Returns:
        Confirmation message
    """
    # Validate URL
    if not url.startswith(("http://", "https://")):
        # Try adding https://
        if "." in url and not url.startswith("/"):
            url = f"https://{url}"
        else:
            return f"✗ Invalid URL: {url}. Must be a valid web address."
    
    try:
        webbrowser.open(url)
        return f"✓ Opened {url} in your browser"
    except Exception as e:
        return f"✗ Failed to open URL: {e}"


# Commands that are safe to run without user confirmation
SAFE_COMMANDS = {
    "ls", "pwd", "whoami", "date", "uptime", "hostname",
    "cat", "head", "tail", "wc", "grep", "find", "which",
    "echo", "printf", "df", "free", "uname",
}

# Commands that are NEVER allowed
BLOCKED_COMMANDS = {
    "rm", "rmdir", "dd", "mkfs", "fdisk", "mount", "umount",
    "shutdown", "reboot", "poweroff", "halt", "init",
    "passwd", "useradd", "userdel", "usermod", "groupadd",
    "chmod", "chown", "chgrp",
    "curl", "wget",  # Network operations
    "sudo", "su", "doas",  # Privilege escalation
}


@tool
def run_shell(command: str) -> str:
    """Execute a shell command and return the output.
    
    Use this for system information, file operations, or automation tasks.
    Be careful with this tool - explain to the user what you're doing.
    
    Some commands are blocked for safety (rm, sudo, etc.)
    
    Args:
        command: The shell command to execute
    
    Returns:
        Command output or error message
    """
    # Parse the first word (the actual command)
    parts = command.strip().split()
    if not parts:
        return "✗ Empty command"
    
    cmd_name = parts[0]
    
    # Check blocklist
    if cmd_name in BLOCKED_COMMANDS:
        return f"✗ Command '{cmd_name}' is blocked for safety reasons"
    
    # Check for pipe to dangerous commands
    if "|" in command:
        for blocked in BLOCKED_COMMANDS:
            if f"| {blocked}" in command or f"|{blocked}" in command:
                return f"✗ Piping to '{blocked}' is not allowed"
    
    # Check for shell operators that could be dangerous
    dangerous_ops = ["&&", "||", ";", ">", ">>", "<"]
    for op in dangerous_ops:
        if op in command:
            # Allow > only for safe commands
            if op in [">", ">>"] and cmd_name not in SAFE_COMMANDS:
                return f"✗ Redirects with '{cmd_name}' are not allowed"
    
    try:
        result = subprocess.run(
            command,
            shell=True,
            capture_output=True,
            text=True,
            timeout=30,  # 30 second timeout
            cwd=os.path.expanduser("~"),  # Run from home directory
        )
        
        output = result.stdout
        if result.stderr:
            output += f"\n[stderr]: {result.stderr}"
        
        if result.returncode != 0:
            output += f"\n[exit code: {result.returncode}]"
        
        # Truncate very long output
        if len(output) > 2000:
            output = output[:2000] + "\n... (truncated)"
        
        return output if output.strip() else "✓ Command completed (no output)"
        
    except subprocess.TimeoutExpired:
        return "✗ Command timed out after 30 seconds"
    except Exception as e:
        return f"✗ Command failed: {e}"


@tool 
def get_system_info() -> str:
    """Get basic system information.
    
    Returns:
        System information including OS, hostname, and uptime
    """
    info = []
    
    try:
        # OS info
        import platform
        info.append(f"OS: {platform.system()} {platform.release()}")
        info.append(f"Machine: {platform.machine()}")
        info.append(f"Hostname: {platform.node()}")
        
        # Uptime (Linux)
        if os.path.exists("/proc/uptime"):
            with open("/proc/uptime", "r") as f:
                uptime_seconds = float(f.readline().split()[0])
                hours = int(uptime_seconds // 3600)
                minutes = int((uptime_seconds % 3600) // 60)
                info.append(f"Uptime: {hours}h {minutes}m")
        
    except Exception as e:
        info.append(f"(Error getting some info: {e})")
    
    return "\n".join(info)
